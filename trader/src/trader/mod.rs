use bfb::bfb_market::Bfb;
use core::borrow;
use dogemarket::dogemarket::DogeMarket;
use market_sol::SOLMarket;
use rand::Rng;
use std::cell::RefCell;
use std::collections::HashMap;
use std::hash::Hash;
use std::ops::Range;
use std::rc::Rc;
use std::string::ToString;
use unitn_market_2022::event::wrapper::NotifiableMarketWrapper;
use unitn_market_2022::good::consts::DEFAULT_GOOD_KIND;
use unitn_market_2022::good::good::Good;
use unitn_market_2022::good::good_kind::{GoodKind, GoodKind::*};
use unitn_market_2022::market::Market;
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
        let mut goods: HashMap<GoodKind, Good> = KINDS
            .iter()
            .map(|k| {
                (
                    *k,
                    Good::new(*k, rand::thread_rng().gen_range(RANGE_GOOD_QTY)),
                )
            })
            .collect();
        let mut markets = [
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
        let mut goods: HashMap<GoodKind, Good> = KINDS
            .iter()
            .map(|k| match k {
                EUR => (*k, Good::new(*k, eur)),
                YEN => (*k, Good::new(*k, yen)),
                USD => (*k, Good::new(*k, usd)),
                YUAN => (*k, Good::new(*k, yuan)),
            })
            .collect();
        let mut markets = [
            DogeMarket::new_random(),
            Bfb::new_random(),
            SOLMarket::new_random(),
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
                    "\n{}: {} buy: {} sell:{}",
                    gl.good_kind, gl.quantity, gl.exchange_rate_buy, gl.exchange_rate_sell
                );
            }
        }
        print!("\n");
    }

    pub fn show_all_buy_prices(&self) {}

    pub fn show_all_sell_prices(&self) {}

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
}

#[cfg(test)]
mod trader_tests {
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

        // let my_m = "PSE_Market";
        // let tmp = trader.get_market_by_name(my_m.to_owned()).unwrap();
        // assert_eq!(my_m.to_owned(), tmp.borrow().get_name().to_owned());

        let my_m = "SOL";
        let tmp = trader.get_market_by_name(my_m.to_owned()).unwrap();
        assert_eq!(my_m.to_owned(), tmp.borrow().get_name().to_owned());
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
