use std::cell::RefCell;

use alloy::{
    network::EthereumWallet,
    primitives::{address, Address, U256},
    providers::{Provider, ProviderBuilder},
    signers::Signer,
    sol,
    transports::icp::IcpConfig,
};

use crate::create_icp_signer;
use crate::services::rpc_service::{get_rpc_service_by_chain_id, SEPOLIA_CHAIN_ID};

thread_local! {
    static NONCE: RefCell<Option<u64>> = const { RefCell::new(None) };
}

// WETH ABI - Wrapped ETH follows ERC-20 standard with additional deposit/withdraw
sol!(
    #[allow(missing_docs, clippy::too_many_arguments)]
    #[sol(rpc)]
    WETH,
    r#"[
        {
            "constant": false,
            "inputs": [
                {"name": "spender", "type": "address"},
                {"name": "amount", "type": "uint256"}
            ],
            "name": "approve",
            "outputs": [{"name": "", "type": "bool"}],
            "type": "function"
        },
        {
            "constant": true,
            "inputs": [
                {"name": "owner", "type": "address"},
                {"name": "spender", "type": "address"}
            ],
            "name": "allowance",
            "outputs": [{"name": "", "type": "uint256"}],
            "type": "function"
        },
        {
            "constant": true,
            "inputs": [{"name": "account", "type": "address"}],
            "name": "balanceOf",
            "outputs": [{"name": "", "type": "uint256"}],
            "type": "function"
        },
        {
            "constant": false,
            "inputs": [],
            "name": "deposit",
            "outputs": [],
            "payable": true,
            "type": "function"
        },
        {
            "constant": false,
            "inputs": [{"name": "amount", "type": "uint256"}],
            "name": "withdraw",
            "outputs": [],
            "type": "function"
        }
    ]"#
);

// Sepolia testnet addresses
const UNISWAP_V2_ROUTER: &str = "0xeE567Fe1712Faf6149d80dA1E6934E354124CfE3";

/// Approve WETH spending for Uniswap V2 Router on Sepolia.
/// 
/// This function allows Uniswap to spend your WETH tokens for trading.
/// Common workflow:
/// 1. Wrap ETH → WETH (deposit)
/// 2. Approve WETH for Uniswap (this function)
/// 3. Trade on Uniswap
#[ic_cdk::update]
pub async fn approve_weth_for_uniswap(amount: String) -> Result<String, String> {
    approve_weth(UNISWAP_V2_ROUTER.to_string(), amount).await
}

/// Approve WETH spending for any spender address.
#[ic_cdk::update]
pub async fn approve_weth(spender_address: String, amount: String) -> Result<String, String> {
    // Parse the spender address
    let spender_address = spender_address.parse::<Address>()
        .map_err(|e| format!("Invalid spender address: {}", e))?;
    
    // Parse the amount (WETH has 18 decimals like ETH)
    let amount = amount.parse::<U256>()
        .map_err(|e| format!("Invalid amount: {}", e))?;

    // Setup signer
    let signer = create_icp_signer().await?;
    let address = signer.address();

    // Setup provider
    let wallet = EthereumWallet::from(signer);
    let rpc_service = get_rpc_service_by_chain_id(SEPOLIA_CHAIN_ID)?;
    let config = IcpConfig::new(rpc_service);
    let provider = ProviderBuilder::new()
        .with_gas_estimation()
        .wallet(wallet)
        .on_icp(config);

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

    // Create WETH contract instance (Sepolia WETH address)
    let contract = WETH::new(
        address!("0x7b79995e5f793A07Bc00c21412e50Ecae098E7f9"),
        provider.clone(),
    );

    // Check current allowance
    let current_allowance = contract.allowance(address, spender_address).call().await
        .map_err(|e| format!("Failed to get current allowance: {}", e))?;

    ic_cdk::println!("Current WETH allowance for {}: {}", spender_address, current_allowance._0);

    // Execute the approve transaction
    match contract
        .approve(spender_address, amount)
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
                    
                    // Log the approval
                    let amount_eth = amount.to_string().parse::<f64>().unwrap_or(0.0) / 1e18;
                    ic_cdk::println!(
                        "WETH approval successful: {} approved {} WETH (raw: {}) for spender {}",
                        address, 
                        amount_eth,
                        amount,
                        spender_address
                    );

                    Ok(format!(
                        "WETH approval successful! Tx: {:?}. Spender {} can now spend up to {} WETH", 
                        tx_hash,
                        spender_address,
                        amount_eth
                    ))
                }
                None => Err("Transaction not found after sending".to_string()),
            }
        }
        Err(e) => Err(format!("WETH approve transaction failed: {:?}", e)),
    }
}

/// Approve WETH spending with human-readable amount.
/// 
/// This function accepts amounts like "1.5" (meaning 1.5 WETH/ETH) and automatically 
/// converts them to the proper 18-decimal format.
#[ic_cdk::update]
pub async fn approve_weth_human(spender_address: String, amount_human: String) -> Result<String, String> {
    // Parse human-readable amount (e.g., "1.5" WETH)
    let amount_f64: f64 = amount_human.parse()
        .map_err(|e| format!("Invalid amount format: {}", e))?;
    
    if amount_f64 < 0.0 {
        return Err("Amount cannot be negative".to_string());
    }
    
    // Convert to Wei (multiply by 10^18 for WETH's 18 decimals)
    let amount_wei = (amount_f64 * 1e18) as u128;
    let amount_u256 = U256::from(amount_wei);
    
    // Use the main approve function
    approve_weth(spender_address, amount_u256.to_string()).await
}

/// Get current WETH allowance for a spender
#[ic_cdk::update]
pub async fn get_weth_allowance(owner_address: Option<String>, spender_address: String) -> Result<String, String> {
    // Parse the spender address
    let spender_address = spender_address.parse::<Address>()
        .map_err(|e| format!("Invalid spender address: {}", e))?;
    
    // Determine owner address
    let owner_address = match owner_address {
        Some(addr) => addr.parse::<Address>()
            .map_err(|e| format!("Invalid owner address: {}", e))?,
        None => {
            let signer = create_icp_signer().await?;
            signer.address()
        }
    };

    // Setup provider (read-only, no wallet needed)
    let rpc_service = get_rpc_service_by_chain_id(SEPOLIA_CHAIN_ID)?;
    let config = IcpConfig::new(rpc_service);
    let provider = ProviderBuilder::new().on_icp(config);

    // Create WETH contract instance
    let contract = WETH::new(
        address!("0x7b79995e5f793A07Bc00c21412e50Ecae098E7f9"),
        provider,
    );

    // Get allowance
    let allowance = contract.allowance(owner_address, spender_address).call().await
        .map_err(|e| format!("Failed to get allowance: {}", e))?;

    let allowance_eth = allowance._0.to_string().parse::<f64>().unwrap_or(0.0) / 1e18;

    Ok(format!(
        "WETH Allowance: {} ETH (raw: {} wei) - Owner: {}, Spender: {}",
        allowance_eth,
        allowance._0,
        owner_address,
        spender_address
    ))
}

/// Get WETH balance
#[ic_cdk::update]
pub async fn get_weth_balance(address: Option<String>) -> Result<String, String> {
    let address = match address {
        Some(val) => val.parse::<Address>().map_err(|e| format!("Invalid address: {}", e))?,
        None => {
            let signer = create_icp_signer().await?;
            signer.address()
        }
    };

    let rpc_service = get_rpc_service_by_chain_id(SEPOLIA_CHAIN_ID)?;
    let config = IcpConfig::new(rpc_service);
    let provider = ProviderBuilder::new().on_icp(config);

    let contract = WETH::new(
        address!("0x7b79995e5f793A07Bc00c21412e50Ecae098E7f9"),
        provider,
    );

    let result = contract.balanceOf(address).call().await;
    match result {
        Ok(balance) => {
            let balance_eth = balance._0.to_string().parse::<f64>().unwrap_or(0.0) / 1e18;
            Ok(format!("WETH Balance: {} ETH (raw: {} wei)", balance_eth, balance._0))
        },
        Err(e) => Err(format!("Failed to get WETH balance: {}", e)),
    }
}

/// Revoke WETH approval (set allowance to 0)
#[ic_cdk::update]
pub async fn revoke_weth_approval(spender_address: String) -> Result<String, String> {
    approve_weth(spender_address, "0".to_string()).await
} 