//! JSON-ручки по свечам, которые собирает сервис kcs-monitor (таблица `candles`).
//!
//!   GET /api/candles                  — выборка с фильтрами и постраничностью
//!   GET /api/candles/latest           — последние N свечей каждой серии
//!   GET /api/candles/meta             — какие серии есть в БД и за какой период
//!   GET /api/candles/series/{symbol}  — серия(ы) по паре, по возрастанию времени
//!
//! Общие query-параметры:
//!   * `exchange`  — биржа, например `kucoin`;
//!   * `symbol`    — пары через запятую: `BTC-USDT,ETH-USDT`;
//!   * `timeframe` — таймфреймы через запятую: `1hour,1day`;
//!   * `from`, `to` — границы периода по времени начала свечи: unix-секунды,
//!     RFC3339 (`2026-09-10T12:00:00Z`) или `YYYY-MM-DD[ HH:MM:SS]` (UTC);
//!   * `order`     — `asc` или `desc` (по умолчанию: `desc`, у `/series` — `asc`);
//!   * `limit`     — сколько строк вернуть, `offset` — сдвиг.
//!
//! Лимиты: `/api/candles` и `/api/candles/series` — `limit` по умолчанию 500,
//! максимум 5000 (у `/series` он ограничивает ВСЮ выборку, а не каждую серию);
//! `/api/candles/latest` — свечей на серию: по умолчанию 1, максимум 100;
//! `/api/candles/meta` — `limit` не применяется, строк столько же, сколько серий.
//!
//! В ответе `count` — число свечей (только у `/api/candles`), `series_count` —
//! число серий (у сгруппированных ответов).

use crate::api::models::{Candle, CandleFilter, CandleMeta, CandleOrder, CandleSeries};
use crate::core::app_state::AppState;
use crate::services::candle_service::group_by_series;
use actix_web::{HttpResponse, Result as ActixResult, web};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::time::Instant;
use tracing::error;

/// Фильтры, общие для всех ручек свечей.
#[derive(Debug, Deserialize)]
pub struct CandleQuery {
    pub exchange: Option<String>,
    pub symbol: Option<String>,
    pub timeframe: Option<String>,
    pub from: Option<String>,
    pub to: Option<String>,
    pub order: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

impl CandleQuery {
    /// Разбирает query-параметры в фильтр; текст ошибки уходит клиенту в 400.
    fn filter(&self) -> Result<CandleFilter, String> {
        let from_ts = self.from.as_deref().map(parse_ts).transpose()?;
        let to_ts = self.to.as_deref().map(parse_ts).transpose()?;

        if let (Some(from), Some(to)) = (from_ts, to_ts)
            && from > to
        {
            return Err(format!("from ({from}) больше to ({to})"));
        }

        Ok(CandleFilter {
            exchange: single_value(self.exchange.as_deref()),
            symbols: split_list(self.symbol.as_deref()),
            timeframes: split_list(self.timeframe.as_deref()),
            from_ts,
            to_ts,
        })
    }

    /// Направление сортировки; `default` используется, если параметр не задан.
    fn order(&self, default: CandleOrder) -> Result<CandleOrder, String> {
        match self.order.as_deref().map(|v| v.trim().to_ascii_lowercase()) {
            None => Ok(default),
            Some(v) if v.is_empty() => Ok(default),
            Some(v) if v == "asc" => Ok(CandleOrder::Asc),
            Some(v) if v == "desc" => Ok(CandleOrder::Desc),
            Some(v) => Err(format!("неизвестный order '{v}': допустимо asc или desc")),
        }
    }
}

#[derive(Debug, Serialize)]
struct CandlesResponse {
    count: usize,
    candles: Vec<Candle>,
    elapsed_ms: u128,
}

#[derive(Debug, Serialize)]
struct LatestCandlesResponse {
    series_count: usize,
    series: Vec<CandleSeries>,
    elapsed_ms: u128,
}

#[derive(Debug, Serialize)]
struct CandleMetaResponse {
    series_count: usize,
    series: Vec<CandleMeta>,
    elapsed_ms: u128,
}

#[derive(Debug, Serialize)]
struct CandleSeriesResponse {
    symbol: String,
    series_count: usize,
    series: Vec<CandleSeries>,
    elapsed_ms: u128,
}

/// GET /api/candles — свечи по фильтру, свежие сверху (или `order=asc`).
pub async fn candles(
    state: web::Data<AppState>,
    query: web::Query<CandleQuery>,
) -> ActixResult<HttpResponse> {
    let started = Instant::now();

    let filter = match query.filter() {
        Ok(filter) => filter,
        Err(message) => return Ok(bad_request(message)),
    };
    let order = match query.order(CandleOrder::Desc) {
        Ok(order) => order,
        Err(message) => return Ok(bad_request(message)),
    };

    match state
        .candle_service
        .get_candles(filter, order, query.limit, query.offset)
        .await
    {
        Ok(candles) => Ok(HttpResponse::Ok().json(CandlesResponse {
            count: candles.len(),
            candles,
            elapsed_ms: started.elapsed().as_millis(),
        })),
        Err(err) => Ok(service_error(err)),
    }
}

/// GET /api/candles/latest — последние `limit` свечей каждой подходящей серии
/// (по умолчанию 1, максимум 100), сгруппированные по бирже/паре/таймфрейму.
pub async fn latest_candles(
    state: web::Data<AppState>,
    query: web::Query<CandleQuery>,
) -> ActixResult<HttpResponse> {
    let started = Instant::now();

    let filter = match query.filter() {
        Ok(filter) => filter,
        Err(message) => return Ok(bad_request(message)),
    };

    match state.candle_service.get_latest(filter, query.limit).await {
        Ok(candles) => {
            let series = group_by_series(candles);
            Ok(HttpResponse::Ok().json(LatestCandlesResponse {
                series_count: series.len(),
                series,
                elapsed_ms: started.elapsed().as_millis(),
            }))
        }
        Err(err) => Ok(service_error(err)),
    }
}

/// GET /api/candles/meta — справочник серий: пары, таймфреймы, покрытие по
/// времени и число свечей. По нему видно, какие фильтры имеют смысл.
/// Фильтры те же, что и у остальных ручек (`from`/`to` сужают период сводки).
pub async fn candles_meta(
    state: web::Data<AppState>,
    query: web::Query<CandleQuery>,
) -> ActixResult<HttpResponse> {
    let started = Instant::now();

    let filter = match query.filter() {
        Ok(filter) => filter,
        Err(message) => return Ok(bad_request(message)),
    };

    match state.candle_service.get_meta(filter).await {
        Ok(series) => Ok(HttpResponse::Ok().json(CandleMetaResponse {
            series_count: series.len(),
            series,
            elapsed_ms: started.elapsed().as_millis(),
        })),
        Err(err) => Ok(service_error(err)),
    }
}

/// GET /api/candles/series/{symbol} — серия(ы) по паре для графика.
/// Символ берётся из пути и перекрывает query-параметр `symbol`; порядок по
/// умолчанию `asc`, чтобы данные шли слева направо без сортировки на клиенте.
pub async fn candle_series(
    state: web::Data<AppState>,
    path: web::Path<String>,
    query: web::Query<CandleQuery>,
) -> ActixResult<HttpResponse> {
    let started = Instant::now();
    let symbol = path.into_inner();

    let mut filter = match query.filter() {
        Ok(filter) => filter,
        Err(message) => return Ok(bad_request(message)),
    };
    filter.symbols = Some(vec![symbol.clone()]);

    let order = match query.order(CandleOrder::Asc) {
        Ok(order) => order,
        Err(message) => return Ok(bad_request(message)),
    };

    match state
        .candle_service
        .get_candles(filter, order, query.limit, query.offset)
        .await
    {
        Ok(candles) => {
            let series = group_by_series(candles);
            Ok(HttpResponse::Ok().json(CandleSeriesResponse {
                symbol,
                series_count: series.len(),
                series,
                elapsed_ms: started.elapsed().as_millis(),
            }))
        }
        Err(err) => Ok(service_error(err)),
    }
}

/// Разбирает время из query-параметра: unix-секунды, RFC3339 или
/// `YYYY-MM-DD[ HH:MM:SS]` (последние два — в UTC).
fn parse_ts(raw: &str) -> Result<i64, String> {
    let value = raw.trim();
    if value.is_empty() {
        return Err("пустое значение времени".to_string());
    }

    if let Ok(unix) = value.parse::<i64>() {
        return Ok(unix);
    }
    if let Ok(datetime) = chrono::DateTime::parse_from_rfc3339(value) {
        return Ok(datetime.timestamp());
    }
    if let Ok(date) = chrono::NaiveDate::parse_from_str(value, "%Y-%m-%d")
        && let Some(datetime) = date.and_hms_opt(0, 0, 0)
    {
        return Ok(datetime.and_utc().timestamp());
    }
    if let Ok(naive) = chrono::NaiveDateTime::parse_from_str(value, "%Y-%m-%d %H:%M:%S") {
        return Ok(naive.and_utc().timestamp());
    }

    Err(format!(
        "не удалось разобрать время '{raw}': ожидается unix-секунды, RFC3339 или YYYY-MM-DD[ HH:MM:SS]"
    ))
}

/// Единичное значение: пустая строка = фильтр не задан.
fn single_value(raw: Option<&str>) -> Option<String> {
    raw.map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

/// Список значений через запятую; пустые элементы отбрасываются.
fn split_list(raw: Option<&str>) -> Option<Vec<String>> {
    let values: Vec<String> = raw?
        .split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .collect();

    if values.is_empty() {
        None
    } else {
        Some(values)
    }
}

fn bad_request(message: String) -> HttpResponse {
    HttpResponse::BadRequest().json(json!({ "error": message }))
}

fn service_error(err: impl std::fmt::Display) -> HttpResponse {
    error!("Service error: {}", err);
    HttpResponse::InternalServerError().json(json!({ "error": "Service error" }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn query() -> CandleQuery {
        CandleQuery {
            exchange: None,
            symbol: None,
            timeframe: None,
            from: None,
            to: None,
            order: None,
            limit: None,
            offset: None,
        }
    }

    #[test]
    fn parses_filter_values() {
        let query = CandleQuery {
            exchange: Some(" kucoin ".to_string()),
            symbol: Some("BTC-USDT, ETH-USDT ,".to_string()),
            timeframe: Some("1hour,1day".to_string()),
            from: Some("1788998400".to_string()),
            to: Some("2026-09-10T12:00:00Z".to_string()),
            ..query()
        };

        let filter = query.filter().expect("фильтр должен разобраться");

        assert_eq!(filter.exchange.as_deref(), Some("kucoin"));
        assert_eq!(
            filter.symbols.as_deref(),
            Some(["BTC-USDT".to_string(), "ETH-USDT".to_string()].as_slice())
        );
        assert_eq!(
            filter.timeframes.as_deref(),
            Some(["1hour".to_string(), "1day".to_string()].as_slice())
        );
        assert_eq!(filter.from_ts, Some(1_788_998_400));
        assert_eq!(filter.to_ts, Some(1_789_041_600));
    }

    #[test]
    fn empty_filter_values_mean_no_filter() {
        let query = CandleQuery {
            exchange: Some("   ".to_string()),
            symbol: Some(" , ".to_string()),
            ..query()
        };

        let filter = query.filter().expect("пустые значения не ошибка");

        assert_eq!(filter.exchange, None);
        assert_eq!(filter.symbols, None);
        assert_eq!(filter.timeframes, None);
        assert_eq!(filter.from_ts, None);
    }

    #[test]
    fn rejects_reversed_period() {
        let query = CandleQuery {
            from: Some("1789041600".to_string()),
            to: Some("1788998400".to_string()),
            ..query()
        };

        assert!(query.filter().is_err());
    }

    #[test]
    fn order_defaults_and_case_insensitive() {
        assert_eq!(query().order(CandleOrder::Desc).unwrap(), CandleOrder::Desc);
        assert_eq!(query().order(CandleOrder::Asc).unwrap(), CandleOrder::Asc);

        let upper = CandleQuery {
            order: Some(" ASC ".to_string()),
            ..query()
        };
        assert_eq!(upper.order(CandleOrder::Desc).unwrap(), CandleOrder::Asc);

        let broken = CandleQuery {
            order: Some("up".to_string()),
            ..query()
        };
        assert!(broken.order(CandleOrder::Desc).is_err());
    }

    #[test]
    fn parses_time_in_supported_formats() {
        assert_eq!(parse_ts("1788998400").unwrap(), 1_788_998_400);
        assert_eq!(parse_ts("2026-09-10T12:00:00Z").unwrap(), 1_789_041_600);
        assert_eq!(parse_ts("2026-09-10 12:00:00").unwrap(), 1_789_041_600);
        assert_eq!(parse_ts("2026-09-10").unwrap(), 1_788_998_400);
        assert!(parse_ts("  ").is_err());
        assert!(parse_ts("вчера").is_err());
    }
}
