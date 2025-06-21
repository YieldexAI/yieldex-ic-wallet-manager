/// Common ERC-20 token addresses on Sepolia Testnet
/// These constants can be used with the universal ERC-20 service methods

/// USDC on Sepolia
pub const USDC_SEPOLIA: &str = "0x1c7d4B196Cb0C7B01d743Fbc6116a902379C7238";

/// LINK on Sepolia  
pub const LINK_SEPOLIA: &str = "0xf8Fb3713D459D7C1018BD0A49d19b4C44290EbE5";

/// WETH on Sepolia
pub const WETH_SEPOLIA: &str = "0x7b79995e5f793A07Bc00c21412e50Ecae098E7f9";

/// AAVE Pool on Sepolia (spender for approvals)
pub const AAVE_POOL_SEPOLIA: &str = "0x6Ae43d3271ff6888e7Fc43Fd7321a503ff738951";

/// Helper struct for token information
pub struct TokenConstant {
    pub symbol: &'static str,
    pub name: &'static str,
    pub address: &'static str,
    pub decimals: u8,
}

/// All supported tokens on Sepolia
pub const SEPOLIA_TOKENS: &[TokenConstant] = &[
    TokenConstant {
        symbol: "USDC",
        name: "USD Coin",
        address: USDC_SEPOLIA,
        decimals: 6,
    },
    TokenConstant {
        symbol: "LINK",
        name: "Chainlink Token",
        address: LINK_SEPOLIA,
        decimals: 18,
    },
    TokenConstant {
        symbol: "WETH",
        name: "Wrapped Ether",
        address: WETH_SEPOLIA,
        decimals: 18,
    },
];

/// Get token address by symbol
pub fn get_token_address_by_symbol(symbol: &str) -> Option<&'static str> {
    SEPOLIA_TOKENS
        .iter()
        .find(|token| token.symbol.eq_ignore_ascii_case(symbol))
        .map(|token| token.address)
}

/// Get token info by address
pub fn get_token_info_by_address(address: &str) -> Option<&'static TokenConstant> {
    SEPOLIA_TOKENS
        .iter()
        .find(|token| token.address.eq_ignore_ascii_case(address))
} 