//! Сервис чтения свечей kcs-monitor: фильтры, лимиты и склейка серий.

use crate::api::models::{Candle, CandleFilter, CandleMeta, CandleOrder, CandleSeries};
use crate::core::error::AppResult;
use crate::repositories::CandleRepository;

/// Постраничность ручки `GET /api/candles`.
pub const DEFAULT_LIMIT: i64 = 500;
pub const MAX_LIMIT: i64 = 5_000;
/// Глубина ручки `GET /api/candles/latest` (свечей на каждую серию).
pub const DEFAULT_LATEST_POINTS: i64 = 1;
pub const MAX_LATEST_POINTS: i64 = 100;

pub struct CandleService<R: CandleRepository> {
    repo: R,
}

impl<R: CandleRepository> CandleService<R> {
    pub fn new(repo: R) -> Self {
        Self { repo }
    }

    /// Свечи по фильтру; `limit`/`offset` зажимаются в допустимый диапазон,
    /// чтобы запрос нельзя было «раздуть» через query-параметры.
    pub async fn get_candles(
        &self,
        filter: CandleFilter,
        order: CandleOrder,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> AppResult<Vec<Candle>> {
        let limit = clamp(limit, DEFAULT_LIMIT, 1, MAX_LIMIT);
        let offset = clamp(offset, 0, 0, i64::MAX);

        self.repo
            .get_candles(&filter, order, limit, offset)
            .await
            .map_err(Into::into)
    }

    /// Последние `points` свечей каждой серии.
    pub async fn get_latest(
        &self,
        filter: CandleFilter,
        points: Option<i64>,
    ) -> AppResult<Vec<Candle>> {
        let points = clamp(points, DEFAULT_LATEST_POINTS, 1, MAX_LATEST_POINTS);

        self.repo
            .get_latest_candles(&filter, points)
            .await
            .map_err(Into::into)
    }

    /// Справочник серий: пары, таймфреймы, покрытие по времени, число свечей.
    pub async fn get_meta(&self, filter: CandleFilter) -> AppResult<Vec<CandleMeta>> {
        self.repo.get_meta(&filter).await.map_err(Into::into)
    }
}

/// Складывает плоский список свечей в серии. Ожидает, что список уже
/// сгруппирован по (exchange, symbol, timeframe) — так его отдаёт репозиторий.
pub fn group_by_series(candles: Vec<Candle>) -> Vec<CandleSeries> {
    let mut series: Vec<CandleSeries> = Vec::new();

    for candle in candles {
        let same_series = series.last().is_some_and(|s| {
            s.exchange == candle.exchange
                && s.symbol == candle.symbol
                && s.timeframe == candle.timeframe
        });

        if same_series {
            if let Some(last) = series.last_mut() {
                last.candles.push(candle);
            }
        } else {
            series.push(CandleSeries {
                exchange: candle.exchange.clone(),
                symbol: candle.symbol.clone(),
                timeframe: candle.timeframe.clone(),
                candles: vec![candle],
            });
        }
    }

    series
}

fn clamp(value: Option<i64>, default: i64, min: i64, max: i64) -> i64 {
    value.unwrap_or(default).clamp(min, max)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn candle(symbol: &str, timeframe: &str, start_ts: i64) -> Candle {
        let time = chrono::DateTime::from_timestamp(start_ts, 0).expect("корректная метка времени");
        Candle {
            exchange: "kucoin".to_string(),
            symbol: symbol.to_string(),
            timeframe: timeframe.to_string(),
            start_ts,
            start_time: time,
            open: 1.0,
            high: 2.0,
            low: 0.5,
            close: 1.5,
            volume: 10.0,
            turnover: 15.0,
            update_time: time,
        }
    }

    #[test]
    fn groups_candles_by_series_keeping_order() {
        let candles = vec![
            candle("BTC-USDT", "1hour", 100),
            candle("BTC-USDT", "1hour", 200),
            candle("BTC-USDT", "1day", 100),
            candle("ETH-USDT", "1hour", 100),
        ];

        let series = group_by_series(candles);

        assert_eq!(series.len(), 3);
        assert_eq!(series[0].symbol, "BTC-USDT");
        assert_eq!(series[0].timeframe, "1hour");
        assert_eq!(
            series[0]
                .candles
                .iter()
                .map(|c| c.start_ts)
                .collect::<Vec<_>>(),
            vec![100, 200]
        );
        assert_eq!(series[1].timeframe, "1day");
        assert_eq!(series[2].symbol, "ETH-USDT");
        assert_eq!(series[2].candles.len(), 1);
    }

    #[test]
    fn group_of_empty_list_is_empty() {
        assert!(group_by_series(Vec::new()).is_empty());
    }

    #[test]
    fn clamps_limits_into_range() {
        assert_eq!(clamp(None, DEFAULT_LIMIT, 1, MAX_LIMIT), DEFAULT_LIMIT);
        assert_eq!(clamp(Some(0), DEFAULT_LIMIT, 1, MAX_LIMIT), 1);
        assert_eq!(clamp(Some(-5), DEFAULT_LIMIT, 1, MAX_LIMIT), 1);
        assert_eq!(clamp(Some(10), DEFAULT_LIMIT, 1, MAX_LIMIT), 10);
        assert_eq!(
            clamp(Some(i64::MAX), DEFAULT_LIMIT, 1, MAX_LIMIT),
            MAX_LIMIT
        );
        assert_eq!(
            clamp(None, DEFAULT_LATEST_POINTS, 1, MAX_LATEST_POINTS),
            DEFAULT_LATEST_POINTS
        );
    }
}
