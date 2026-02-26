use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct Ticker {
    pub exchange: String,
    pub symbol: String,
    pub symbol_name: String,
    pub taker_fee_rate: Option<String>,
    pub maker_fee_rate: Option<String>,
    pub taker_coefficient: Option<String>,
    pub maker_coefficient: Option<String>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}
#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct ActiveOrder {
    pub exchange: String,
    pub order_id: String,
    pub symbol: String,
    pub side: String,
    pub price: String,
    pub origin_size: String,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}
#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct EventOrder {
    pub exchange: String,
    pub status: String,
    pub type_: String,
    pub symbol: String,
    pub side: String,
    pub order_type: String,
    pub fee_type: Option<String>,
    pub liquidity: Option<String>,
    pub price: Option<String>,
    pub order_id: String,
    pub client_oid: Option<String>,
    pub trade_id: Option<String>,
    pub origin_size: Option<String>,
    pub size: Option<String>,
    pub filled_size: Option<String>,
    pub match_size: Option<String>,
    pub match_price: Option<String>,
    pub canceled_size: Option<String>,
    pub old_size: Option<String>,
    pub remain_size: Option<String>,
    pub remain_funds: Option<String>,
    pub order_time: i64,
    pub ts: i64,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct TradeSymbol {
    pub exchange: String,
    pub symbol: String,
    pub size: String,
    pub enable: bool,
}
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Symbol {
    pub exchange: String,
    pub symbol: String,
    pub symbol_name: String,
    pub base_currency: String,
    pub quote_currency: String,
    pub fee_currency: String,
    pub market: String,
    pub base_min_size: String,
    pub quote_min_size: String,
    pub base_max_size: String,
    pub quote_max_size: String,
    pub base_increment: String,
    pub quote_increment: String,
    pub price_increment: String,
    pub price_limit_rate: String,
    pub min_funds: Option<String>,
    pub is_margin_enabled: bool,
    pub enable_trading: bool,
    pub fee_category: i16,
    pub maker_fee_coefficient: String,
    pub taker_fee_coefficient: String,
    pub st: bool,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Currency {
    pub exchange: String,
    pub currency: String,
    pub currency_name: String,
    pub full_name: String,
    pub is_margin_enabled: bool,
    pub is_debit_enabled: bool,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct PgConnection {
    pub total_connections: i64,
    pub active_connections: i64,
}
#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct PgTableInfo {
    pub schemaname: String,
    pub relname: String,
    pub seq_scan: Option<i64>,
    pub seq_tup_read: Option<i64>,
    pub idx_scan: Option<i64>,
    pub idx_tup_fetch: Option<i64>,
    pub n_tup_ins: Option<i64>,
    pub n_tup_upd: Option<i64>,
    pub n_tup_del: Option<i64>,
    pub n_live_tup: Option<i64>,
    pub n_dead_tup: Option<i64>,
}
#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct PgTableIndex {
    pub schemaname: String,
    pub relname: String,
    pub idx_scan: Option<i64>,
    pub idx_tup_read: Option<i64>,
    pub idx_tup_fetch: Option<i64>,
}
#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct PgStatStatements {
    pub query: String,
    pub calls: Option<i64>,
    pub total_exec_time: Option<f64>,
    pub mean_exec_time: Option<f64>,
    pub rows: Option<i64>,
}
#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct PgStatTableSize {
    pub schemaname: String,
    pub relname: String,
    pub total_size: String,
    pub table_size: String,
    pub indexes_size: String,
}
#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct Event {
    pub exchange: String,
    pub msg: String,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}
#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct Error {
    pub exchange: String,
    pub msg: String,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}
#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct PositionRatio {
    pub exchange: String,
    pub debt_ratio: f64,
    pub total_asset: f64,
    pub margin_coefficient_total_asset: String,
    pub total_debt: String,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}
#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct PositionDebt {
    pub exchange: String,
    pub debt_symbol: String,
    pub debt_value: String,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}
#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct PositionAsset {
    pub exchange: String,
    pub asset_symbol: String,
    pub asset_total: String,
    pub asset_available: String,
    pub asset_hold: String,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct Balance {
    pub exchange: String,
    pub account_id: String,
    pub available: String,
    pub available_change: String,
    pub currency: String,
    pub hold_value: String,
    pub hold_change: String,
    pub relation_event: String,
    pub relation_event_id: String,
    pub event_time: String,
    pub total: String,
    pub symbol: Option<String>,
    pub order_id: Option<String>,
    pub trade_id: Option<String>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}
#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct MsgSend {
    pub exchange: String,
    pub args_symbol: Option<String>,
    pub args_side: Option<String>,
    pub args_size: Option<String>,
    pub args_funds: Option<String>,
    pub args_price: Option<String>,
    pub args_time_in_force: Option<String>,
    pub args_type: Option<String>,
    pub args_auto_borrow: Option<bool>,
    pub args_auto_repay: Option<bool>,
    pub args_client_oid: Option<String>,
    pub args_order_id: Option<String>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}
#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct Bots {
    pub exchange: Option<String>,
    pub entry_id: Option<String>,
    pub exit_tp_id: Option<String>,
    pub exit_sl_id: Option<String>,
    pub balance: Option<String>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}
#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct MsgEvent {
    pub exchange: String,
    pub msg: Option<String>,
    pub code: Option<String>,
    pub borrow_size: Option<String>,
    pub client_oid: Option<String>,
    pub order_id: Option<String>,
    pub loan_apply_id: Option<String>,
    pub limit_rate: Option<f64>,
    pub reset_rate: Option<f64>,
    pub remaining_rate: Option<f64>,
    pub in_time: f64,
    pub out_time: f64,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}
