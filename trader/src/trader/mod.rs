mod arbitrage;

use crate::trader::arbitrage::Arbitrage;
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

    /// Get the maximum amount of a good the trader can buy from a market according to
    ///     - The amount of cash the trader has
    ///     - The amount of good the market has
    fn maximum_buy(&self, kind: &GoodKind, market_name: &String) -> Result<f32, MarketGetterError> {
        let cash_qty = self.goods.get(&DEFAULT_GOOD_KIND).unwrap().get_qty();
        let market = self.get_market_by_name(market_name.clone()).unwrap();
        // Get rate by using get_buy_price because some market give the prices in EUR -> GOOD and others in GOOD -> EUR
        let rate = market.borrow().get_buy_price(kind.clone(), 1f32)? * 1.05; // Overestimate the rate by 5%
        let market_good_qty = market
            .borrow()
            .get_goods()
            .iter()
            .find(|&g| g.good_kind.eq(&kind))
            .unwrap()
            .quantity;
        let trader_max = cash_qty / rate;
        Ok(market_good_qty.min(trader_max))
    }

    /// Get the maximum amount of a good the trader can sell to a market according to
    ///     - The amount of good the trader has
    ///     - The amount of cash the market has
    fn maximum_sell(&self, kind: &GoodKind, market_name: &String) -> Result<f32, MarketGetterError> {
        let good_qty = self.goods.get(&kind).unwrap().get_qty();
        let market = self.get_market_by_name(market_name.clone()).unwrap();
        // Get rate by using get_sell_price because some market give the prices in EUR -> GOOD and others in GOOD -> EUR
        let rate = market.borrow().get_sell_price(kind.clone(), 1f32)? * 1.05; // Overestimate the rate by 5%
        let market_cash_qty = market
            .borrow()
            .get_goods()
            .iter()
            .find(|&g| g.good_kind.eq(&DEFAULT_GOOD_KIND))
            .unwrap()
            .quantity;
        let market_max = market_cash_qty / rate;
        Ok(market_max.min(good_qty))
    }

    /// Find arbitrage opportunities from every markets
    /// Return a list of arbitrages opportunities
    fn find_arbitrages(&self) -> Vec<Arbitrage> {
        let mut arbitrages: Vec<Arbitrage> = Vec::new();
        for (i1, buy_market) in self.markets.iter().enumerate() {
            for (i2, sell_market) in self.markets.iter().enumerate() {
                if i1 == i2 { // If same market, pass
                    continue;
                }
                let buy_market_name = buy_market.borrow().get_name().to_string();
                let sell_market_name = sell_market.borrow().get_name().to_string();
                for kind in &KINDS {
                    if kind.eq(&DEFAULT_GOOD_KIND) { continue; }

                    // Get the maximum qty the trader and markets can trade
                    let max_buy_qty = self.maximum_buy(&kind, &buy_market_name).unwrap_or(0f32);
                    let max_sell_qty = self.maximum_sell(&kind, &sell_market_name).unwrap_or(0f32);
                    let max_qty = max_buy_qty.min(max_sell_qty) * 0.9; // 90% just in case the market wants to keep a reserve

                    // Get the Buy and Sell prices
                    // If an error occurs, we set the buy price at the max and the sell price at the min possible
                    let buy_price = buy_market.borrow().get_buy_price(kind.clone(), max_qty).unwrap_or(f32::MAX);
                    let sell_price = sell_market.borrow().get_sell_price(kind.clone(), max_qty).unwrap_or(f32::MIN_POSITIVE);

                    let benefits = sell_price - buy_price;
                    let margin = benefits / buy_price;

                    // Check if we have an arbitrage
                    if sell_price > buy_price && buy_price > 0f32 && sell_price > 0f32 {
                        arbitrages.push(Arbitrage::new(buy_market_name.clone(), sell_market_name.clone(), kind.clone(), benefits, margin, max_qty));
                    }
                }
            }
        }
        arbitrages
    }

    /// This method exploit a weakness of the PSE market to find an arbitrage opportunity
    ///
    /// Weakness of PSE market:
    ///     When lock buying a null quantity of goods on the market the prices starts to fluctuate a lot after some time,
    ///     giving us the opportunity to make some benefits with an arbitrage method.
    pub(crate) fn exploit_pse_market(&mut self) {
        let pse = self.markets.iter().find(|&m| m.borrow().get_name().eq(&"PSE_Market".to_string())).unwrap();
        for d in 0..=1500 { // TODO: Determine how many days we want to trade
            // Make the price fluctuate by lock buying a null quantity
            for k in &KINDS {
                if k.eq(&EUR) { continue; }
                pse.borrow_mut().lock_buy(k.clone(), 0f32, f32::MAX, self.name.clone()).unwrap();
            }

            // Get all the arbitrages opportunities and take the worthiest one
            let mut arbitrages = self.find_arbitrages();
            arbitrages.sort_by(|a1, a2| a1.benefits.total_cmp(&a2.benefits));
            let highest_benefits_arbitrage = arbitrages.pop();

            if let Some(arbitrage) = highest_benefits_arbitrage {

                // We are not playing for peanuts
                if arbitrage.benefits < 10_000f32 || arbitrage.margin < 0.1 { continue; }

                println!("{d}\tFound a worthy arbitrage {:?}", arbitrage);

                let buy_market_name = arbitrage.buying_market_name;
                let sell_market_name = arbitrage.selling_market_name;
                let kind = arbitrage.good_kind;
                let qty = arbitrage.max_qty;

                let buy_market = self.markets.iter().find(|&m| m.borrow().get_name().eq(&buy_market_name.clone())).unwrap();
                let sell_market = self.markets.iter().find(|&m| m.borrow().get_name().eq(&sell_market_name.clone())).unwrap();

                // Get bid & offer (-/+5% Because some markets does not integrate their margin in these methods)
                let bid = buy_market.borrow().get_buy_price(kind.clone(), qty).unwrap() * 1.05;
                let offer = sell_market.borrow().get_sell_price(kind.clone(), qty).unwrap() * 0.95;

                println!("BUY \t{} {}\tfrom {:<20}\tat {}", qty, kind, buy_market_name, bid);
                println!("SELL\t{} {}\tto   {:<20}\tat {}\n", qty, kind, sell_market_name, offer);

                let buy_token = buy_market.borrow_mut().lock_buy(kind, qty, bid, self.name.clone()).unwrap();
                let sell_token = sell_market.borrow_mut().lock_sell(kind, qty, offer, self.name.clone()).unwrap();

                let recv_good = buy_market.borrow_mut().buy(buy_token, self.goods.get_mut(&DEFAULT_GOOD_KIND).unwrap()).unwrap();
                let recv_cash = sell_market.borrow_mut().sell(sell_token, self.goods.get_mut(&arbitrage.good_kind).unwrap()).unwrap();

                self.goods.get_mut(&DEFAULT_GOOD_KIND).unwrap().merge(recv_cash).unwrap();
                self.goods.get_mut(&arbitrage.good_kind).unwrap().merge(recv_good).unwrap();
            }
        }
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
    fn exploit_pse() {
        let mut trader = SOLTrader::new();
        trader.subscribe_markets_to_one_another();
        let value_before = trader.goods.iter().fold(0f32, |acc, (_, good)| acc + (good.get_qty() / good.get_kind().get_default_exchange_rate()));
        trader.exploit_pse_market();
        let value_after = trader.goods.iter().fold(0f32, |acc, (_, good)| acc + (good.get_qty() / good.get_kind().get_default_exchange_rate()));
        let profit = value_after - value_before;
        let margin_percentage = (profit / value_before) * 100f32;
        assert!(value_after > value_before);
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
