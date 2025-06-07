use std::cell::RefCell;

use alloy::{
    network::EthereumWallet,
    primitives::{address, U256, U160, Bytes, Uint},
    providers::{Provider, ProviderBuilder},
    signers::Signer,
    sol,
    transports::icp::IcpConfig,
};

use crate::{create_icp_signer, get_rpc_service_sepolia};
use crate::services::approve_weth;

thread_local! {
    static NONCE: RefCell<Option<u64>> = const { RefCell::new(None) };
}

// Universal Router contract (Uniswap V3)
sol!(
    #[allow(missing_docs, clippy::too_many_arguments)]
    #[sol(rpc)]
    UniversalRouter,
    "src/abi/UniversalRouter.json"
);

// QuoterV2 contract for getting quotes
sol!(
    #[allow(missing_docs, clippy::too_many_arguments)]
    #[sol(rpc)]
    QuoterV2,
    "src/abi/QuoterV2.json"
);

// Sepolia testnet addresses (from Perplexity research)
const UNIVERSAL_ROUTER: &str = "0x3fc91a3afd70395cd496c647d5a6cc9d4b2b7fad";
const QUOTER_V2: &str = "0x61fFE014bA17989E743c5F6cB21bF9697530B21e"; // Using a known working address
const WETH_SEPOLIA: &str = "0x7b79995e5f793A07Bc00c21412e50Ecae098E7f9";
const USDC_SEPOLIA: &str = "0x1c7d4b196cb0c7b01d743fbc6116a902379c7238";

// Uniswap V3 fee tiers (in basis points)
const FEE_LOW: u32 = 500;     // 0.05%
const FEE_MEDIUM: u32 = 3000; // 0.3%
const FEE_HIGH: u32 = 10000;  // 1%

/// Get quote for WETH → USDC swap using Uniswap V3.
/// 
/// This function shows how much USDC you would receive for a given amount of WETH
/// using the most liquid pool (usually 0.3% fee tier).
#[ic_cdk::update]
pub async fn get_weth_usdc_quote_v3(weth_amount: String) -> Result<String, String> {
    // Parse the WETH amount (18 decimals)
    let weth_amount = weth_amount.parse::<U256>()
        .map_err(|e| format!("Invalid WETH amount: {}", e))?;

    if weth_amount == U256::ZERO {
        return Err("Amount must be greater than 0".to_string());
    }

    // Setup provider (read-only, no wallet needed)
    let rpc_service = get_rpc_service_sepolia();
    let config = IcpConfig::new(rpc_service);
    let provider = ProviderBuilder::new().on_icp(config);

    // Create QuoterV2 contract instance
    let quoter = QuoterV2::new(
        address!("61fFE014bA17989E743c5F6cB21bF9697530B21e"),
        provider,
    );

    // Try different fee tiers to find the best price
    let mut best_quote = U256::ZERO;
    let mut best_fee = FEE_MEDIUM;

    for fee in [FEE_LOW, FEE_MEDIUM, FEE_HIGH] {
        match quoter.quoteExactInputSingle(
            address!("7b79995e5f793A07Bc00c21412e50Ecae098E7f9"),    // tokenIn (WETH)
            address!("1c7d4b196cb0c7b01d743fbc6116a902379c7238"),    // tokenOut (USDC)
            fee,                       // fee
            weth_amount,               // amountIn
            U256::ZERO,                // sqrtPriceLimitX96 (no limit)
        ).call().await {
            Ok(quote) => {
                let amount_out = quote._0;
                if amount_out > best_quote {
                    best_quote = amount_out;
                    best_fee = fee;
                }
                ic_cdk::println!("Fee {}bp: {} USDC", fee, amount_out.to_string().parse::<f64>().unwrap_or(0.0) / 1_000_000.0);
            },
            Err(e) => {
                ic_cdk::println!("Failed to get quote for fee {}bp: {}", fee, e);
            }
        }
    }

    if best_quote == U256::ZERO {
        return Err("Could not get quote from any pool".to_string());
    }

    let weth_human = weth_amount.to_string().parse::<f64>().unwrap_or(0.0) / 1e18;
    let usdc_human = best_quote.to_string().parse::<f64>().unwrap_or(0.0) / 1_000_000.0;

    Ok(format!(
        "Quote: {} WETH → {} USDC (rate: 1 WETH = {} USDC, best fee: {}bp)",
        weth_human,
        usdc_human,
        usdc_human / weth_human,
        best_fee
    ))
}

/// Get a simple quote estimate for WETH → USDC.
/// 
/// This is a simplified version that provides an estimated quote.
/// For a real implementation, this would query Uniswap V3 pools.
/// For now, it provides a reasonable estimate based on typical rates.
#[ic_cdk::update]
pub async fn get_weth_usdc_quote_v3_human(weth_amount_human: String) -> Result<String, String> {
    // Parse human-readable amount (e.g., "0.01" WETH)
    let amount_f64: f64 = weth_amount_human.parse()
        .map_err(|e| format!("Invalid amount format: {}", e))?;
    
    if amount_f64 <= 0.0 {
        return Err("Amount must be greater than 0".to_string());
    }
    
    // Simplified estimate: 1 WETH ≈ 3000 USDC (this would be fetched from Uniswap in real implementation)
    let estimated_rate = 3000.0;
    let estimated_usdc = amount_f64 * estimated_rate;
    
    Ok(format!(
        "Estimated quote: {} WETH → {} USDC (rate: 1 WETH ≈ {} USDC)\nNote: This is a simplified estimate. Use actual Uniswap interface for precise quotes.",
        amount_f64,
        estimated_usdc,
        estimated_rate
    ))
}

/// Swap WETH for USDC using Uniswap V3 Universal Router.
/// 
/// This function swaps a specified amount of WETH for USDC with slippage protection.
/// Prerequisites:
/// 1. You must have WETH tokens (use wrap_eth first if needed)
/// 2. You must approve the Universal Router to spend your WETH
/// 
/// The function will:
/// - Get the best quote from available pools
/// - Execute the swap via Universal Router
/// - Apply 1% slippage protection
#[ic_cdk::update]
pub async fn swap_weth_for_usdc_v3(weth_amount: String) -> Result<String, String> {
    // Parse the WETH amount (18 decimals)
    let weth_amount = weth_amount.parse::<U256>()
        .map_err(|e| format!("Invalid WETH amount: {}", e))?;

    if weth_amount == U256::ZERO {
        return Err("Amount must be greater than 0".to_string());
    }

    // Setup signer
    let signer = create_icp_signer().await?;
    let address = signer.address();

    // Setup provider
    let wallet = EthereumWallet::from(signer);
    let rpc_service = get_rpc_service_sepolia();
    let config = IcpConfig::new(rpc_service);
    let provider = ProviderBuilder::new()
        .with_gas_estimation()
        .wallet(wallet)
        .on_icp(config);

    // First get the best quote
    let quoter = QuoterV2::new(
        address!("61fFE014bA17989E743c5F6cB21bF9697530B21e"),
        provider.clone(),
    );

    // Find the best fee tier
    let mut best_quote = U256::ZERO;
    let mut best_fee = FEE_MEDIUM;

    for fee in [FEE_LOW, FEE_MEDIUM, FEE_HIGH] {
        if let Ok(quote) = quoter.quoteExactInputSingle(
            address!("7b79995e5f793A07Bc00c21412e50Ecae098E7f9"),  // WETH
            address!("1c7d4b196cb0c7b01d743fbc6116a902379c7238"),  // USDC
            fee,
            weth_amount,
            U256::ZERO,
        ).call().await {
            if quote._0 > best_quote {
                best_quote = quote._0;
                best_fee = fee;
            }
        }
    }

    if best_quote == U256::ZERO {
        return Err("Could not get quote for swap".to_string());
    }

    // Apply 1% slippage tolerance
    let min_usdc_out = best_quote * U256::from(99) / U256::from(100);

    ic_cdk::println!(
        "Expected USDC output: {} (using {}bp fee)",
        best_quote.to_string().parse::<f64>().unwrap_or(0.0) / 1_000_000.0,
        best_fee
    );
    ic_cdk::println!(
        "Minimum USDC output (1% slippage): {}",
        min_usdc_out.to_string().parse::<f64>().unwrap_or(0.0) / 1_000_000.0
    );

    // Create Universal Router contract instance
    let router = UniversalRouter::new(
        address!("3fc91a3afd70395cd496c647d5a6cc9d4b2b7fad"),
        provider.clone(),
    );

    // Handle nonce management
    let maybe_nonce = NONCE.with_borrow(|maybe_nonce| {
        maybe_nonce.map(|nonce| nonce + 1)
    });

    let nonce = if let Some(nonce) = maybe_nonce {
        nonce
    } else {
        provider.get_transaction_count(address).await
            .map_err(|e| format!("Failed to get nonce: {}", e))?
    };

    // Set deadline: 20 minutes from now
    let deadline = U256::from(ic_cdk::api::time() / 1_000_000_000 + 1200);

    // Encode V3 swap parameters
    // Command 0x00 = V3_SWAP_EXACT_IN
    let commands = Bytes::from([0x00]);
    
    // Encode the swap parameters for V3_SWAP_EXACT_IN
    // Parameters: recipient, amountIn, amountOutMin, path, payerIsUser
    let swap_params = alloy::sol_types::abi::encode(&(
        address,          // recipient
        weth_amount,      // amountIn  
        min_usdc_out,     // amountOutMin
        encode_v3_path(
            address!("7b79995e5f793A07Bc00c21412e50Ecae098E7f9"), // WETH
            best_fee, 
            address!("1c7d4b196cb0c7b01d743fbc6116a902379c7238")  // USDC
        ), // path
        true,             // payerIsUser
    ));

    let inputs = vec![Bytes::from(swap_params)];

    // Execute the swap
    match router
        .execute_0(commands, inputs, deadline)
        .nonce(nonce)
        .chain_id(11155111) // Sepolia chain ID
        .from(address)
        .send()
        .await
    {
        Ok(builder) => {
            let tx_hash = *builder.tx_hash();
            let tx_response = provider.get_transaction_by_hash(tx_hash).await
                .map_err(|e| format!("Failed to get transaction: {}", e))?;

            match tx_response {
                Some(tx) => {
                    // Update nonce cache
                    NONCE.with_borrow_mut(|nonce| {
                        *nonce = Some(tx.nonce);
                    });
                    
                    // Log the swap
                    ic_cdk::println!(
                        "WETH → USDC V3 swap successful: {} swapped {} WETH for USDC",
                        address, 
                        weth_amount.to_string().parse::<f64>().unwrap_or(0.0) / 1e18
                    );

                    Ok(format!(
                        "V3 Swap successful! Tx: {:?}. Swapped {} WETH for USDC (expected ~{} USDC)", 
                        tx_hash,
                        weth_amount.to_string().parse::<f64>().unwrap_or(0.0) / 1e18,
                        best_quote.to_string().parse::<f64>().unwrap_or(0.0) / 1_000_000.0
                    ))
                }
                None => Err("Transaction not found after sending".to_string()),
            }
        }
        Err(e) => Err(format!("V3 swap transaction failed: {:?}", e)),
    }
}

/// Swap WETH for USDC with human-readable amount using V3.
#[ic_cdk::update]
pub async fn swap_weth_for_usdc_v3_human(weth_amount_human: String) -> Result<String, String> {
    // Parse human-readable amount (e.g., "0.01" WETH)
    let amount_f64: f64 = weth_amount_human.parse()
        .map_err(|e| format!("Invalid amount format: {}", e))?;
    
    if amount_f64 <= 0.0 {
        return Err("Amount must be greater than 0".to_string());
    }
    
    // Convert to Wei (multiply by 10^18 for WETH's 18 decimals)
    let amount_wei = (amount_f64 * 1e18) as u128;
    let amount_u256 = U256::from(amount_wei);
    
    // Use the main swap function
    swap_weth_for_usdc_v3(amount_u256.to_string()).await
}

/// Approve Universal Router to spend WETH tokens with human-readable amount.
/// 
/// This function approves the Universal Router to spend your WETH tokens
/// for Uniswap V3 swaps. You need to call this before performing any swaps.
#[ic_cdk::update]
pub async fn approve_weth_for_universal_router_human(amount_human: String) -> Result<String, String> {
    // Use the existing approve_weth_human function with Universal Router address
    approve_weth::approve_weth_human(UNIVERSAL_ROUTER.to_string(), amount_human).await
}

/// Approve Universal Router to spend WETH tokens with raw amount.
#[ic_cdk::update]
pub async fn approve_weth_for_universal_router(amount: String) -> Result<String, String> {
    // Use the existing approve_weth function with Universal Router address
    approve_weth::approve_weth(UNIVERSAL_ROUTER.to_string(), amount).await
}

// Helper function to encode V3 path (tokenA -> fee -> tokenB)
fn encode_v3_path(token_a: alloy::primitives::Address, fee: u32, token_b: alloy::primitives::Address) -> Bytes {
    let mut path = Vec::new();
    path.extend_from_slice(token_a.as_slice());
    path.extend_from_slice(&fee.to_be_bytes()[1..4]); // fee is 3 bytes
    path.extend_from_slice(token_b.as_slice());
    Bytes::from(path)
} 