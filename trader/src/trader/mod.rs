pub mod strategies;

use bfb::bfb_market::Bfb;
use dogemarket::dogemarket::DogeMarket;
use ipc_utils::trader_state::TraderState;
use ipc_utils::trading_event::TradingEvent;
use ipc_utils::trading_event_details::{TradeOperation, TradeType, TradingEventDetails};
use ipc_utils::IPCSender;
use rand::Rng;
use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::Range;
use std::rc::Rc;
use std::string::ToString;
use unitn_market_2022::good::consts::DEFAULT_GOOD_KIND;
use unitn_market_2022::good::good::Good;
use unitn_market_2022::good::good_kind::{GoodKind, GoodKind::*};
use unitn_market_2022::market::{Market, MarketGetterError};
use unitn_market_2022::{subscribe_each_other, wait_one_day};
use Pizza_Stock_Exchange_Market::PSE_Market;

const KINDS: [GoodKind; 4] = [EUR, USD, YEN, YUAN];
const RANGE_GOOD_QTY: Range<f32> = 50_000f32..150_000f32;

pub struct SOLTrader {
    pub(crate) name: String,
    pub(crate) goods: HashMap<GoodKind, Good>,
    pub(crate) markets: Vec<Rc<RefCell<dyn Market>>>,
    pub(crate) ipc_sender: Option<IPCSender>,
}

impl SOLTrader {
    /// Initialise a trader with DOGE, BFB and PSE markets.
    /// Trader's good quantities are random number ranging from 50,000 to 150,000
    pub fn new(name: String) -> Self {
        let goods: HashMap<GoodKind, Good> = KINDS
            .iter()
            .map(|k| {
                (
                    *k,
                    Good::new(*k, rand::thread_rng().gen_range(RANGE_GOOD_QTY)),
                )
            })
            .collect();
        let doge = DogeMarket::new_random();
        let bfb = Bfb::new_random();
        let pse = PSE_Market::new_random();
        subscribe_each_other!(doge, bfb, pse);
        Self {
            name,
            goods,
            markets: [doge, bfb, pse].to_vec(),
            ipc_sender: None,
        }
    }

    /// Initialise a trader with DOGE, BFB and PSE markets and specific quantities
    pub fn new_with_quantities(name: String, eur: f32, usd: f32, yen: f32, yuan: f32) -> Self {
        let goods: HashMap<GoodKind, Good> = KINDS
            .iter()
            .map(|k| match k {
                EUR => (*k, Good::new(*k, eur)),
                YEN => (*k, Good::new(*k, yen)),
                USD => (*k, Good::new(*k, usd)),
                YUAN => (*k, Good::new(*k, yuan)),
            })
            .collect();
        let doge = DogeMarket::new_random();
        let bfb = Bfb::new_random();
        let pse = PSE_Market::new_random();
        subscribe_each_other!(doge, bfb, pse);
        Self {
            name,
            goods,
            markets: [doge, bfb, pse].to_vec(),
            ipc_sender: None,
        }
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
        println!("*** Markets info ***");
        for mrk_bind in self.markets.iter() {
            println!("{}", mrk_bind.borrow().get_name());
            for gl in mrk_bind.borrow().get_goods().iter() {
                println!(
                    "{:<5}:\t{:<15} buy: {:<15} sell: {:<15}",
                    gl.good_kind, gl.quantity, gl.exchange_rate_buy, gl.exchange_rate_sell
                );
            }
            println!();
        }
    }

    pub fn show_all_self_quantities(&self) {
        println!("*** Trader stocks ({}) ***", self.name.clone());
        for (_, qty) in self.goods.iter() {
            println!("{}", qty);
        }
        println!();
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
        
        self.log_visualizer(
            mrk_bind.borrow().get_name().to_string(),
            TradingEventDetails {
                successful: true,
                trade_type: TradeType::Buy,
                good_kind: kind,
                quantity: qty,
                price: bid,
                operation: TradeOperation::AskedLock,
            },
        );

        //split the cash!
        let mut cash = self
            .goods
            .get_mut(&DEFAULT_GOOD_KIND)
            .unwrap()
            .split(bid)
            .unwrap();

        let buy_result = mrk_bind.borrow_mut().buy(token, &mut cash);

        //add the good locally
        self.goods
            .get_mut(&kind)
            .unwrap()
            .merge(buy_result.unwrap())
            .unwrap();

        println!("\n Bought from {} {} of {}", name, qty, kind);

        self.log_visualizer(
            mrk_bind.borrow().get_name().to_string(),
            TradingEventDetails {
                successful: true,
                trade_type: TradeType::Buy,
                good_kind: kind,
                quantity: qty,
                price: bid,
                operation: TradeOperation::TradeFinalized,
            },
        );
    }

    pub fn sell_to_market(&mut self, name: String, kind: GoodKind, qty: f32) {
        let mrk_bind = self.get_market_by_name(name.clone()).unwrap().clone();

        //sell the good
        let offer = mrk_bind.borrow().get_sell_price(kind, qty).ok().unwrap();
        let token = mrk_bind
            .borrow_mut()
            .lock_sell(kind, qty, offer, String::from("SOLTrader"))
            .unwrap();

        self.log_visualizer(
            mrk_bind.borrow().get_name().to_string(),
            TradingEventDetails {
                successful: true,
                trade_type: TradeType::Sell,
                good_kind: kind,
                quantity: qty,
                price: offer,
                operation: TradeOperation::AskedLock,
            },
        );

        //split the good
        let mut good_to_sell = self.goods.get_mut(&kind).unwrap().split(qty).unwrap();

        let sell_result = mrk_bind.borrow_mut().sell(token, &mut good_to_sell);

        //get the cash
        self.goods
            .get_mut(&DEFAULT_GOOD_KIND)
            .unwrap()
            .merge(sell_result.unwrap())
            .unwrap();

        println!("\n Sold to {} {} of {}", name, qty, kind);

        self.log_visualizer(
            mrk_bind.borrow().get_name().to_string(),
            TradingEventDetails {
                successful: true,
                trade_type: TradeType::Sell,
                good_kind: kind,
                quantity: qty,
                price: offer,
                operation: TradeOperation::TradeFinalized,
            },
        );
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
        Ok(market_good_qty.min(trader_max))
    }

    /// Get the maximum amount of a good the trader can sell to a market according to
    ///     - The amount of good the trader has
    ///     - The amount of cash the market has
    fn max_sell(&self, kind: &GoodKind, market_name: &String) -> Result<f32, MarketGetterError> {
        let good_qty = self.goods.get(&kind).unwrap().get_qty();
        let market = self.get_market_by_name(market_name.clone()).unwrap();
        // Get rate by using get_sell_price because some market give the prices in EUR -> GOOD and others in GOOD -> EUR
        let rate = market.borrow().get_sell_price(kind.clone(), 1f32)?;
        let market_cash_qty =
            self.get_cur_good_qty_from_market(&DEFAULT_GOOD_KIND, market_name.clone());
        let market_max = market_cash_qty / rate;
        Ok(market_max.min(good_qty))
    }

    /// Retrieve the current worth of the trader in DEFAULT_GOOD (EUR)
    pub fn get_current_worth(&self) -> f32 {
        self.goods.iter().fold(0f32, |acc, (_, good)| {
            acc + (good.get_qty() / good.get_kind().get_default_exchange_rate())
        })
    }

    /// Send a trade event to the visualizer.
    /// If no visualizer set (self.ipc_sender = None), we print the event on stdout
    pub fn log_visualizer(&self, market_name: String, trade_event: TradingEventDetails) {
        let trading_event = TradingEvent {
            details: trade_event,
            market_name: market_name.clone(),
            trader_state: TraderState::new(
                self.goods
                    .iter()
                    .map(|(_, g)| (g.get_kind(), g.get_qty()))
                    .collect(),
                self.name.clone(),
            ),
        };

        if let Some(send_to_visualizer) = &self.ipc_sender {
            send_to_visualizer.send(trading_event.clone()).unwrap()
        } else {
            println!("{trading_event:?}");
        }
    }

    /// Lock buy from a market reference.
    /// Before using this method:
    /// - Need to check if trader has enough DEFAULT_GOOD quantity
    /// - Need to check if market has enough GOOD quantity
    ///
    /// This method does not handle errors, all the checks needs to be done beforehand.
    /// It is meant to be used with the arbitrage strategy where all the checks are done in the `find_arbitrages` method
    pub fn lock_buy_from_market_ref(
        &self,
        market: Rc<RefCell<dyn Market>>,
        kind: GoodKind,
        qty: f32,
    ) -> (f32, String) {
        let bid = market.borrow().get_buy_price(kind, qty).unwrap();
        let token = market
            .borrow_mut()
            .lock_buy(kind, qty, bid, self.name.clone())
            .unwrap();
        self.log_visualizer(
            market.borrow().get_name().to_string(),
            TradingEventDetails {
                successful: true,
                trade_type: TradeType::Buy,
                good_kind: kind,
                quantity: qty,
                price: bid,
                operation: TradeOperation::AskedLock,
            },
        );
        (bid, token)
    }

    /// Lock sell to a market reference.
    /// Before using this method be sure:
    /// - Trader has enough GOOD quantity
    /// - Market has enough DEFAULT_GOOD quantity
    ///
    /// This method does not handle errors, all the checks needs to be done beforehand.
    /// It is meant to be used with the arbitrage strategy where all the checks are done in the `find_arbitrages` method
    pub fn lock_sell_to_market_ref(
        &self,
        market: Rc<RefCell<dyn Market>>,
        kind: GoodKind,
        qty: f32,
    ) -> (f32, String) {
        let offer = market.borrow().get_sell_price(kind, qty).unwrap();
        let token = market
            .borrow_mut()
            .lock_sell(kind, qty, offer, self.name.clone())
            .unwrap();
        self.log_visualizer(
            market.borrow().get_name().to_string(),
            TradingEventDetails {
                successful: true,
                trade_type: TradeType::Sell,
                good_kind: kind,
                quantity: qty,
                price: offer,
                operation: TradeOperation::AskedLock,
            },
        );
        (offer, token)
    }

    /// Buy from a market reference.
    /// Using this method implies:
    /// - bid and token need to be retrieve from `lock_sell_to_market_ref` method
    /// - market, qty and kind should be the same as used for `lock_sell_to_market_ref` method
    ///
    /// This method does not handle errors, all the checks needs to be done beforehand.
    /// It is meant to be used with the arbitrage strategy where all the checks are done in the `find_arbitrages` method
    pub fn buy_from_market_ref(
        &mut self,
        market: Rc<RefCell<dyn Market>>,
        token: String,
        bid: f32,
        qty: f32,
        kind: GoodKind,
    ) {
        let mut cash = self
            .goods
            .get_mut(&DEFAULT_GOOD_KIND)
            .unwrap()
            .split(bid)
            .unwrap();
        let good = market.borrow_mut().buy(token, &mut cash).unwrap();
        self.goods.get_mut(&kind).unwrap().merge(good).unwrap();

        self.log_visualizer(
            market.borrow().get_name().to_string(),
            TradingEventDetails {
                successful: true,
                trade_type: TradeType::Buy,
                good_kind: kind,
                quantity: qty,
                price: bid,
                operation: TradeOperation::TradeFinalized,
            },
        );
    }

    /// Sell to a market reference
    /// Using this method implies:
    /// - offer and token need to be retrieve from `lock_sell_to_market_ref` method
    /// - market, qty and kind should be the same as used for `lock_sell_to_market_ref` method
    ///
    /// This method does not handle errors, all the checks needs to be done beforehand.
    /// It is meant to be used with the arbitrage strategy where all the checks are done in the `find_arbitrages` method
    pub fn sell_to_market_ref(
        &mut self,
        market: Rc<RefCell<dyn Market>>,
        token: String,
        offer: f32,
        qty: f32,
        kind: GoodKind,
    ) {
        let mut good = self.goods.get_mut(&kind).unwrap().split(qty).unwrap();
        let cash = market.borrow_mut().sell(token, &mut good).unwrap();
        self.goods
            .get_mut(&DEFAULT_GOOD_KIND)
            .unwrap()
            .merge(cash)
            .unwrap();
        self.log_visualizer(
            market.borrow().get_name().to_string(),
            TradingEventDetails {
                successful: true,
                trade_type: TradeType::Sell,
                good_kind: kind,
                quantity: qty,
                price: offer,
                operation: TradeOperation::TradeFinalized,
            },
        );
    }

    /// Set an IPCSender so the trader can communicate with a visualizer
    pub fn set_ipc_sender(&mut self, ipc_sender: IPCSender) {
        self.ipc_sender = Some(ipc_sender);
    }
}