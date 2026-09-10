//! Чтение свечей, которые собирает сервис kcs-monitor (таблица `candles`).
//!
//! Схема таблицы: PK (exchange, symbol, timeframe, start_ts), числовые колонки
//! `numeric`, `start_ts` — unix-секунды. Запросы здесь только читают данные.

use crate::api::models::{Candle, CandleFilter, CandleMeta, CandleOrder};
use crate::repositories::RepositoryResult;
use async_trait::async_trait;
use sqlx::PgPool;

#[async_trait]
pub trait CandleRepository: Send + Sync {
    /// Свечи по фильтру в заданном порядке, с постраничностью.
    async fn get_candles(
        &self,
        filter: &CandleFilter,
        order: CandleOrder,
        limit: i64,
        offset: i64,
    ) -> RepositoryResult<Vec<Candle>>;

    /// Последние `points` свечей каждой серии (биржа + пара + таймфрейм).
    async fn get_latest_candles(
        &self,
        filter: &CandleFilter,
        points: i64,
    ) -> RepositoryResult<Vec<Candle>>;

    /// Сводка по сериям: сколько свечей, за какой период, когда обновлялись.
    /// Фильтры те же, что и у выборки свечей (например, чтобы получить
    /// справочник только по одной бирже или паре).
    async fn get_meta(&self, filter: &CandleFilter) -> RepositoryResult<Vec<CandleMeta>>;
}

pub struct PostgresCandleRepository {
    pool: PgPool,
}

impl PostgresCandleRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

/// Колонки свечи: `numeric` приводим к `float8` (в JSON — числа), а unix-секунды
/// начала свечи дублируем готовым timestamptz.
const CANDLE_COLUMNS: &str = "\
    exchange, symbol, timeframe, start_ts, \
    to_timestamp(start_ts) AS start_time, \
    open::float8 AS open, high::float8 AS high, low::float8 AS low, \
    close::float8 AS close, volume::float8 AS volume, turnover::float8 AS turnover, \
    update_time";

/// Общие условия фильтрации (параметры $1..$5): необязательные биржа, список пар,
/// список таймфреймов и границы периода. NULL-параметр = фильтр не задан.
const CANDLE_FILTERS: &str = "\
    ($1::text IS NULL OR exchange = $1) \
    AND ($2::text[] IS NULL OR symbol = ANY($2)) \
    AND ($3::text[] IS NULL OR timeframe = ANY($3)) \
    AND ($4::bigint IS NULL OR start_ts >= $4) \
    AND ($5::bigint IS NULL OR start_ts <= $5)";

/// Последние N свечей каждой серии: оконная функция вместо запроса на серию.
const LATEST_CANDLES_QUERY: &str = r#"
    SELECT exchange, symbol, timeframe, start_ts,
           to_timestamp(start_ts) AS start_time,
           open::float8 AS open, high::float8 AS high, low::float8 AS low,
           close::float8 AS close, volume::float8 AS volume, turnover::float8 AS turnover,
           update_time
    FROM (
        SELECT c.*,
               row_number() OVER (
                   PARTITION BY c.exchange, c.symbol, c.timeframe
                   ORDER BY c.start_ts DESC
               ) AS rn
        FROM candles c
        WHERE ($1::text IS NULL OR c.exchange = $1)
          AND ($2::text[] IS NULL OR c.symbol = ANY($2))
          AND ($3::text[] IS NULL OR c.timeframe = ANY($3))
          AND ($4::bigint IS NULL OR c.start_ts >= $4)
          AND ($5::bigint IS NULL OR c.start_ts <= $5)
    ) t
    WHERE t.rn <= $6
    ORDER BY exchange, symbol, timeframe, start_ts DESC
"#;

/// Справочник серий — из него видно, какие значения фильтров вообще есть в БД.
/// Условия те же, что и в выборке свечей: `from`/`to` ограничивают период,
/// за который считается сводка.
const META_QUERY: &str = r#"
    SELECT exchange, symbol, timeframe,
           count(*)::bigint AS candle_count,
           min(start_ts) AS first_start_ts,
           to_timestamp(min(start_ts)) AS first_start_time,
           max(start_ts) AS last_start_ts,
           to_timestamp(max(start_ts)) AS last_start_time,
           max(update_time) AS last_update_time
    FROM candles
    WHERE ($1::text IS NULL OR exchange = $1)
      AND ($2::text[] IS NULL OR symbol = ANY($2))
      AND ($3::text[] IS NULL OR timeframe = ANY($3))
      AND ($4::bigint IS NULL OR start_ts >= $4)
      AND ($5::bigint IS NULL OR start_ts <= $5)
    GROUP BY exchange, symbol, timeframe
    ORDER BY exchange, symbol, timeframe
"#;

#[async_trait]
impl CandleRepository for PostgresCandleRepository {
    async fn get_candles(
        &self,
        filter: &CandleFilter,
        order: CandleOrder,
        limit: i64,
        offset: i64,
    ) -> RepositoryResult<Vec<Candle>> {
        // Направление сортировки подставляется из enum (CandleOrder::as_sql), а
        // не из query-строки, поэтому подстановка безопасна — значение и
        // обоснование зафиксированы в AssertSqlSafe.
        let sql = format!(
            "SELECT {CANDLE_COLUMNS} FROM candles WHERE {CANDLE_FILTERS} \
             ORDER BY start_ts {}, exchange ASC, symbol ASC, timeframe ASC \
             LIMIT $6 OFFSET $7",
            order.as_sql()
        );

        let candles = sqlx::query_as::<_, Candle>(sqlx::AssertSqlSafe(sql))
            .bind(filter.exchange.clone())
            .bind(filter.symbols.clone())
            .bind(filter.timeframes.clone())
            .bind(filter.from_ts)
            .bind(filter.to_ts)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await?;

        Ok(candles)
    }

    async fn get_latest_candles(
        &self,
        filter: &CandleFilter,
        points: i64,
    ) -> RepositoryResult<Vec<Candle>> {
        let candles = sqlx::query_as::<_, Candle>(LATEST_CANDLES_QUERY)
            .bind(filter.exchange.clone())
            .bind(filter.symbols.clone())
            .bind(filter.timeframes.clone())
            .bind(filter.from_ts)
            .bind(filter.to_ts)
            .bind(points)
            .fetch_all(&self.pool)
            .await?;

        Ok(candles)
    }

    async fn get_meta(&self, filter: &CandleFilter) -> RepositoryResult<Vec<CandleMeta>> {
        let meta = sqlx::query_as::<_, CandleMeta>(META_QUERY)
            .bind(filter.exchange.clone())
            .bind(filter.symbols.clone())
            .bind(filter.timeframes.clone())
            .bind(filter.from_ts)
            .bind(filter.to_ts)
            .fetch_all(&self.pool)
            .await?;

        Ok(meta)
    }
}
