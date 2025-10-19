use candid::CandidType;
use serde::{Deserialize, Serialize};

// --- Recommendation Types ---

/// Тип рекомендации для rebalancing операций
#[derive(Clone, Debug, CandidType, Serialize, Deserialize)]
pub enum RecommendationType {
    StandardTransfer,  // Обычный перевод между протоколами на одной сети
}

/// Детали swap операции (опционально, для будущих версий)
#[derive(Clone, Debug, CandidType, Serialize, Deserialize)]
pub struct SwapDetails {
    pub from_token: String,
    pub to_token: String,
    pub from_market: Option<String>,
    pub to_market: Option<String>,
    pub swap_protocol: Option<String>,
}

/// Структура рекомендации для rebalancing
#[derive(Clone, Debug, CandidType, Serialize, Deserialize)]
pub struct Recommendation {
    pub asset: String,                        // "USDC"
    pub to_asset: String,                     // "USDC"
    pub from_chain: String,                   // "Arbitrum"
    pub to_chain: Option<String>,             // Optional для cross-chain (пока не поддерживается)
    pub from_protocol: String,                // "aave-v3" | "compound-v3"
    pub to_protocol: String,                  // "aave-v3" | "compound-v3"
    pub current_apy: f64,                     // Текущая доходность
    pub target_apy: f64,                      // Целевая доходность
    pub estimated_profit: f64,                // Расчетная прибыль
    pub gas_cost: f64,                        // Стоимость газа
    pub position_size: String,                // Количество в читаемом формате "1000"
    pub pool_id: Option<String>,              // Идентификатор пула
    pub recommendation_type: RecommendationType,
    pub swap_details: Option<SwapDetails>,    // Для будущих swap операций
}

/// Результат исполнения рекомендации
#[derive(Clone, Debug, CandidType, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub status: String,                       // "success" | "failed" | "partial"
    pub withdraw_tx: Option<String>,          // Хеш транзакции withdraw
    pub swap_tx: Option<String>,              // Хеш swap (для будущего)
    pub supply_tx: Option<String>,            // Хеш транзакции supply
    pub amount_transferred: String,           // Фактически переведено
    pub actual_gas_cost: Option<f64>,         // Фактическая стоимость газа
    pub error_details: Option<String>,        // Детали ошибки
}
