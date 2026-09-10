use crate::repositories::PostgresSendOrderRepository;
use crate::repositories::{
    PostgresBalanceRepository, PostgresBotRepository, PostgresCandleRepository,
    PostgresCurrencyRepository, PostgresErrorRepository, PostgresEventOrderRepository,
    PostgresEventRepository, PostgresMsgEventRepository, PostgresMsgSendRepository,
    PostgresPgRepository, PostgresPositionRepository, PostgresStopOrderRepository,
    PostgresSymbolRepository, PostgresTickerRepository,
};
use crate::services::SendOrderService;
use crate::services::{
    BalanceService, BotService, CandleService, CurrencyService, ErrorService, EventService,
    MsgEventService, MsgSendService, OrderService, PgService, PositionService, StaticService,
    StopOrderService, SymbolService, TickerService,
};
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub balance_service: Arc<BalanceService<PostgresBalanceRepository>>,
    pub bot_service: Arc<BotService<PostgresBotRepository>>,
    pub candle_service: Arc<CandleService<PostgresCandleRepository>>,
    pub currency_service: Arc<CurrencyService<PostgresCurrencyRepository>>,
    pub error_service: Arc<ErrorService<PostgresErrorRepository>>,
    pub event_service: Arc<EventService<PostgresEventRepository>>,
    pub msgevent_service: Arc<MsgEventService<PostgresMsgEventRepository>>,
    pub msgsend_service: Arc<MsgSendService<PostgresMsgSendRepository>>,
    pub order_service: Arc<OrderService<PostgresEventOrderRepository>>,
    pub pg_service: Arc<PgService<PostgresPgRepository>>,
    pub position_service: Arc<PositionService<PostgresPositionRepository>>,
    pub symbol_service: Arc<SymbolService<PostgresSymbolRepository>>,
    pub ticker_service: Arc<TickerService<PostgresTickerRepository>>,
    pub static_service: Arc<StaticService>,
    pub sendorder_service: Arc<SendOrderService<PostgresSendOrderRepository>>,
    pub stoporder_service: Arc<StopOrderService<PostgresStopOrderRepository>>,
}

impl AppState {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self {
            balance_service: Arc::new(BalanceService::new(PostgresBalanceRepository::new(
                pool.clone(),
            ))),
            bot_service: Arc::new(BotService::new(PostgresBotRepository::new(pool.clone()))),
            candle_service: Arc::new(CandleService::new(PostgresCandleRepository::new(
                pool.clone(),
            ))),
            currency_service: Arc::new(CurrencyService::new(PostgresCurrencyRepository::new(
                pool.clone(),
            ))),
            error_service: Arc::new(ErrorService::new(PostgresErrorRepository::new(
                pool.clone(),
            ))),
            event_service: Arc::new(EventService::new(PostgresEventRepository::new(
                pool.clone(),
            ))),
            msgevent_service: Arc::new(MsgEventService::new(PostgresMsgEventRepository::new(
                pool.clone(),
            ))),
            msgsend_service: Arc::new(MsgSendService::new(PostgresMsgSendRepository::new(
                pool.clone(),
            ))),
            order_service: Arc::new(OrderService::new(PostgresEventOrderRepository::new(
                pool.clone(),
            ))),
            pg_service: Arc::new(PgService::new(PostgresPgRepository::new(pool.clone()))),
            position_service: Arc::new(PositionService::new(PostgresPositionRepository::new(
                pool.clone(),
            ))),
            symbol_service: Arc::new(SymbolService::new(PostgresSymbolRepository::new(
                pool.clone(),
            ))),
            ticker_service: Arc::new(TickerService::new(PostgresTickerRepository::new(
                pool.clone(),
            ))),
            static_service: Arc::new(StaticService::new()),
            sendorder_service: Arc::new(SendOrderService::new(PostgresSendOrderRepository::new(
                pool.clone(),
            ))),
            stoporder_service: Arc::new(StopOrderService::new(PostgresStopOrderRepository::new(
                pool.clone(),
            ))),
        }
    }
}
