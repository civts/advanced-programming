use std::collections::HashMap;
use unitn_market_2022::good::good::Good;
use unitn_market_2022::good::good_kind::GoodKind;
use unitn_market_2022::market::{MarketError, MarketTrait};

pub struct Market {
    name: String,
    budget: Good,
    goods: Vec<Good>,
}

pub struct MarketMetadata {}

impl Market {
    pub(crate) fn new() -> Market {
        Market {
            name: String::from("SOL"),
            budget: Good::new_default(),
            // TODO we need to decide on that
            goods: vec![Good::new(GoodKind::EUR, 100.00),
                        Good::new(GoodKind::YEN, 100.00),
                        Good::new(GoodKind::YUAN, 100.00),
                        Good::new(GoodKind::USD, 100.00)]
        }
    }
}

impl MarketTrait for Market {
    fn get_market_name(&self) -> String {
        return self.name.clone();
    }

    fn get_budget(&self) -> f32 {
        todo!()
    }

    fn get_goods(&self) -> HashMap<GoodKind, &Good> {
        todo!()
    }

    fn lock_trader_buy_from_market(&mut self, g: GoodKind, p: f32, q: f32, d: String) -> Result<String, MarketError> {
        todo!()
    }

    fn trader_buy_from_market(&mut self, token: String, cash: &mut Good) -> Result<Good, MarketError> {
        todo!()
    }

    fn lock_trader_sell_to_market(&mut self, g: GoodKind, qty: f32, price: f32, d: String) -> Result<String, MarketError> {
        todo!()
    }

    fn trader_sell_to_market(&mut self, token: String, good: &mut Good) -> Result<Good, MarketError> {
        todo!()
    }
}