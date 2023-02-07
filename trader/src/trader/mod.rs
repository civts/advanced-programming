mod arbitrage;
pub mod arbitrages;

use bfb::bfb_market::Bfb;
use dogemarket::dogemarket::DogeMarket;
use rand::Rng;
use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::Range;
use std::rc::Rc;
use std::string::ToString;
use unitn_market_2022::event::wrapper::NotifiableMarketWrapper;
use unitn_market_2022::good::consts::DEFAULT_GOOD_KIND;
use unitn_market_2022::good::good::Good;
use unitn_market_2022::good::good_kind::{GoodKind, GoodKind::*};
use unitn_market_2022::market::{Market, MarketGetterError};
use unitn_market_2022::wait_one_day;
use Pizza_Stock_Exchange_Market::PSE_Market;

const KINDS: [GoodKind; 4] = [EUR, USD, YEN, YUAN];
const TRADER_NAME: &str = "SOLTrader";
const RANGE_GOOD_QTY: Range<f32> = 50_000f32..150_000f32; // TODO: Maybe come up with better idea

pub struct SOLTrader {
    pub(crate) name: String,
    pub(crate) goods: HashMap<GoodKind, Good>,
    pub(crate) markets: Vec<Rc<RefCell<dyn Market>>>,
}

impl SOLTrader {
    pub fn new() -> Self {
        let goods: HashMap<GoodKind, Good> = KINDS
            .iter()
            .map(|k| {
                (
                    *k,
                    Good::new(*k, rand::thread_rng().gen_range(RANGE_GOOD_QTY)),
                )
            })
            .collect();
        let markets = [
            DogeMarket::new_random(),
            Bfb::new_random(),
            PSE_Market::new_random(),
        ]
        .to_vec();
        Self {
            name: TRADER_NAME.to_string(),
            goods,
            markets,
        }
    }

    pub fn new_with_quantities(eur: f32, usd: f32, yen: f32, yuan: f32) -> Self {
        let goods: HashMap<GoodKind, Good> = KINDS
            .iter()
            .map(|k| match k {
                EUR => (*k, Good::new(*k, eur)),
                YEN => (*k, Good::new(*k, yen)),
                USD => (*k, Good::new(*k, usd)),
                YUAN => (*k, Good::new(*k, yuan)),
            })
            .collect();
        let markets = [
            DogeMarket::new_random(),
            Bfb::new_random(),
            PSE_Market::new_random(),
        ]
        .to_vec();
        Self {
            name: TRADER_NAME.to_string(),
            goods,
            markets,
        }
    }

    pub fn subscribe_markets_to_one_another(&self) {
        self.markets.iter().enumerate().for_each(|(i1, m1)| {
            self.markets.iter().enumerate().for_each(|(i2, m2)| {
                if i1 != i2 {
                    NotifiableMarketWrapper::new(m1)
                        .add_subscriber(NotifiableMarketWrapper::new(m2));
                }
            })
        });
    }

    pub fn show_all_market_quantities(&self) {
        for mrk_bind in self.markets.iter() {
            print!("\n\n{}", mrk_bind.borrow().get_name());
            for gl in mrk_bind.borrow().get_goods().iter() {
                print!("\n{}: {}", gl.good_kind, gl.quantity);
            }
        }
    }

    pub fn show_all_market_info(&self) {
        for mrk_bind in self.markets.iter() {
            print!("\n\n{}", mrk_bind.borrow().get_name());
            for gl in mrk_bind.borrow().get_goods().iter() {
                print!(
                    "\n{}: {} buy: {} sell: {}",
                    gl.good_kind, gl.quantity, gl.exchange_rate_buy, gl.exchange_rate_sell
                );
            }
        }
        print!("\n");
    }

    pub fn show_all_self_quantities(&self) {
        println!("Trader stocks");
        for (_, qty) in self.goods.iter() {
            print!("{} ", qty);
        }
        println!("\n");
    }

    //you can still borrow_mut the returned market!
    pub fn get_market_by_name(&self, name: String) -> Option<&Rc<RefCell<dyn Market>>> {
        let mut res: Option<&Rc<RefCell<dyn Market>>> = None;

        for mrk_bind in self.markets.iter() {
            if mrk_bind.borrow().get_name().eq(&name) {
                res = Some(mrk_bind);
            }
        }

        res
    }

    pub fn get_market_buy_rates(&self, name: String) -> HashMap<GoodKind, f32> {
        let mut ret: HashMap<GoodKind, f32> = HashMap::new();

        if let Some(mrk_bind) = self.get_market_by_name(name.clone()) {
            for gl in mrk_bind.borrow().get_goods().iter() {
                ret.insert(gl.good_kind, gl.exchange_rate_buy);
            }
        }
        ret
    }

    pub fn get_market_sell_rates(&self, name: String) -> HashMap<GoodKind, f32> {
        let mut ret: HashMap<GoodKind, f32> = HashMap::new();

        if let Some(mrk_bind) = self.get_market_by_name(name.clone()) {
            for gl in mrk_bind.borrow().get_goods().iter() {
                ret.insert(gl.good_kind, gl.exchange_rate_sell);
            }
        }
        ret
    }

    pub fn get_all_current_buy_rates(&self) -> HashMap<String, HashMap<GoodKind, f32>> {
        let mut ret: HashMap<String, HashMap<GoodKind, f32>> = HashMap::new();

        for m in self.markets.iter() {
            let cur_name = m.borrow().get_name().to_owned();
            ret.insert(cur_name.clone(), self.get_market_buy_rates(cur_name));
        }

        ret
    }

    pub fn get_all_current_sell_rates(&self) -> HashMap<String, HashMap<GoodKind, f32>> {
        let mut ret: HashMap<String, HashMap<GoodKind, f32>> = HashMap::new();

        for m in self.markets.iter() {
            let cur_name = m.borrow().get_name().to_owned();
            ret.insert(cur_name.clone(), self.get_market_sell_rates(cur_name));
        }

        ret
    }

    pub fn get_cur_good_qty(&self, g: &GoodKind) -> f32 {
        self.goods[g].get_qty()
    }

    pub fn get_cur_good_qty_from_market(&self, g: &GoodKind, m: String) -> f32 {
        let markt_bind = self.get_market_by_name(m).unwrap();
        let goods = markt_bind.borrow().get_goods();
        let mut ret = 0.0;
        for i in 0..goods.len() {
            if goods[i].good_kind == *g {
                ret = goods[i].quantity;
            }
        }
        ret
    }

    pub fn all_wait_one_day(&self) {
        //wait one day was done on the mrkt binds -> Rc<Refcell<dyn market>>
        wait_one_day!(&self.markets[0], &self.markets[1]);
        //don't use wait one day on pizza stocck market
    }

    //i'm still assuming that i have enough cash
    // TODO:
    // ADD ERRORS!
    //
    pub fn buy_from_market(&mut self, name: String, kind: GoodKind, qty: f32) {
        let mrk_bind = self.get_market_by_name(name.clone()).unwrap().clone();

        let bid = mrk_bind.borrow().get_buy_price(kind, qty).ok().unwrap();
        let token = mrk_bind
            .borrow_mut()
            .lock_buy(kind, qty, bid, String::from("SOLTrader"))
            .unwrap();

        //split the cash!
        let mut cash = self
            .goods
            .get_mut(&DEFAULT_GOOD_KIND)
            .unwrap()
            .split(bid)
            .unwrap();

        let buy_result = mrk_bind.borrow_mut().buy(token, &mut cash);

        //add the good locally
        let mut cash = self
            .goods
            .get_mut(&kind)
            .unwrap()
            .merge(buy_result.unwrap());

        println!("\n Bought from {} {} of {}", name, qty, kind);
    }

    pub fn sell_to_market(&mut self, name: String, kind: GoodKind, qty: f32) {
        let mrk_bind = self.get_market_by_name(name.clone()).unwrap().clone();

        //sell the good
        let offer = mrk_bind.borrow().get_sell_price(kind, qty).ok().unwrap();
        let token = mrk_bind
            .borrow_mut()
            .lock_sell(kind, qty, offer, String::from("SOLTrader"))
            .unwrap();

        //split the good
        let mut good_to_sell = self.goods.get_mut(&kind).unwrap().split(qty).unwrap();

        let sell_result = mrk_bind.borrow_mut().sell(token, &mut good_to_sell);

        //get the cash
        let mut cash = self
            .goods
            .get_mut(&DEFAULT_GOOD_KIND)
            .unwrap()
            .merge(sell_result.unwrap());

        println!("\n Sold to {} {} of {}", name, qty, kind);
    }

    /// Get the maximum amount of a good the trader can buy from a market according to
    ///     - The amount of cash the trader has
    ///     - The amount of good the market has
    fn max_buy(&self, kind: &GoodKind, market_name: &String) -> Result<f32, MarketGetterError> {
        let cash_qty = self.goods.get(&DEFAULT_GOOD_KIND).unwrap().get_qty();
        let market = self.get_market_by_name(market_name.clone()).unwrap();
        // Get rate by using get_buy_price because some market give the prices in EUR -> GOOD and others in GOOD -> EUR
        let market_good_qty = self.get_cur_good_qty_from_market(&kind, market_name.clone());
        let rate = market.borrow().get_buy_price(kind.clone(), 1f32)?;
        let trader_max = cash_qty / rate;
        Ok(market_good_qty.min(trader_max) * 0.95)
    }

    /// Get the maximum amount of a good the trader can sell to a market according to
    ///     - The amount of good the trader has
    ///     - The amount of cash the market has
    fn max_sell(&self, kind: &GoodKind, market_name: &String) -> Result<f32, MarketGetterError> {
        let good_qty = self.goods.get(&kind).unwrap().get_qty();
        let market = self.get_market_by_name(market_name.clone()).unwrap();
        // Get rate by using get_sell_price because some market give the prices in EUR -> GOOD and others in GOOD -> EUR
        let rate = market.borrow().get_sell_price(kind.clone(), 1f32)?;
        let market_cash_qty = self.get_cur_good_qty_from_market(&DEFAULT_GOOD_KIND, market_name.clone());
        let market_max = market_cash_qty / rate;
        Ok(market_max.min(good_qty) * 0.95)
    }

    /// Retrieve the current worth of the trader in DEFAULT_GOOD (EUR)
    pub fn get_current_worth(&self) -> f32 {
        self.goods.iter().fold(0f32, |acc, (_, good)| {
            acc + (good.get_qty() / good.get_kind().get_default_exchange_rate())
        })
    }
}

#[cfg(test)]
mod trader_tests {
    use crate::trader::arbitrages::Arbitrages;
    use crate::trader::SOLTrader;
    use std::rc::Rc;

    #[test]
    fn test_get_market_by_name() {
        let trader = SOLTrader::new();

        let my_m = "DogeMarket";
        let tmp = trader.get_market_by_name(my_m.to_owned()).unwrap();
        assert_eq!(my_m.to_owned(), tmp.borrow().get_name().to_owned());

        let my_m = "Baku stock exchange";
        let tmp = trader.get_market_by_name(my_m.to_owned()).unwrap();
        assert_eq!(my_m.to_owned(), tmp.borrow().get_name().to_owned());

        let my_m = "PSE_Market";
        let tmp = trader.get_market_by_name(my_m.to_owned()).unwrap();
        assert_eq!(my_m.to_owned(), tmp.borrow().get_name().to_owned());

        // let my_m = "SOL";
        // let tmp = trader.get_market_by_name(my_m.to_owned()).unwrap();
        // assert_eq!(my_m.to_owned(), tmp.borrow().get_name().to_owned());
    }

    #[test]
    fn exploit_pse() {
        let mut trader: SOLTrader = SOLTrader::new();

        trader.subscribe_markets_to_one_another();
        let value_before = trader.get_current_worth();
        for _ in 0..366 {
            let mut arbitrages = Arbitrages::find_arbitrages(&trader);
            //println!("DAY {d:02}");
            arbitrages.exploit_pse_market(&mut trader);
        }
        let value_after = trader.get_current_worth();
        let profit = value_after - value_before;
        let margin_percentage = (profit / value_before) * 100f32;
        assert!(value_after > value_before, "Trader is not profitable");
        println!("VALUE BEFORE: {value_before}\nVALUE AFTER: {value_after}\nPROFIT: {margin_percentage}%");
    }

    #[test]
    fn test_subscription() {
        let trader = SOLTrader::new();
        let mut strong_count: usize;
        let mut weak_count: usize;

        // Test before subscription
        for market in trader.markets.iter() {
            strong_count = Rc::strong_count(market);
            weak_count = Rc::weak_count(market);
            assert!(strong_count == 1 && weak_count == 0);
        }

        // Test after subscription
        trader.subscribe_markets_to_one_another();
        let nb_sub_per_market = trader.markets.len() - 1;
        for market in trader.markets.iter() {
            strong_count = Rc::strong_count(market);
            weak_count = Rc::weak_count(market);
            assert!(strong_count == 1 && weak_count == nb_sub_per_market);
        }
    }
}
