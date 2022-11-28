use crate::market_metadata::MarketMetadata;

use std::cell::RefCell;
use std::rc::Rc;
use unitn_market_2022::event::notifiable::Notifiable;
use unitn_market_2022::good::good::Good;
use unitn_market_2022::good::good_kind::GoodKind;
use unitn_market_2022::market::Market;

const MARKET_NAME: &str = "SOL";

pub struct SOLMarket {
    name: String,
    budget: Good,
    goods: Vec<Good>,
    meta: MarketMetadata,
}

impl Market for SOLMarket {
    fn new_random() -> Rc<RefCell<dyn Market>> {
        todo!()
    }

    fn new_with_quantities(eur: f32, yen: f32, usd: f32, yuan: f32) -> Rc<RefCell<dyn Market>> {
        todo!()
    }

    fn new_file(path: &str) -> Rc<RefCell<dyn Market>>
    where
        Self: Sized,
    {
        todo!()
    }

    fn get_name(&self) -> &'static str {
        return MARKET_NAME;
    }

    fn get_budget(&self) -> f32 {
        // self.goods.iter().fold(0f32, |acc, good| {
        //     let value = good.get_qty()
        //         * self
        //             .meta
        //             .goods_meta
        //             .get(&good.get_kind())
        //             .unwrap()
        //             .sell_price; //TODO: Check
        //     acc + value
        // })
        todo!();
    }

    fn get_buy_price(
        &self,
        kind: GoodKind,
        quantity: f32,
    ) -> Result<f32, unitn_market_2022::market::MarketGetterError> {
        todo!()
    }

    fn get_sell_price(
        &self,
        kind: GoodKind,
        quantity: f32,
    ) -> Result<f32, unitn_market_2022::market::MarketGetterError> {
        todo!()
    }

    fn get_goods(&self) -> Vec<unitn_market_2022::market::good_label::GoodLabel> {
        todo!()
    }

    fn lock_buy(
        &mut self,
        kind_to_buy: GoodKind,
        quantity_to_buy: f32,
        bid: f32,
        trader_name: String,
    ) -> Result<String, unitn_market_2022::market::LockBuyError> {
        // let mut good_meta = self.meta.get_mut_good_meta(&g)?;
        // // Check good is locked
        // if good_meta.is_locked() {
        //     good_meta
        //     return Err(LockBuyError::GoodAlreadyLocked{token : });
        // }

        // // Check quantity
        // let quantity_available = good_meta.quantity_available;
        // if quantity_available < q {
        //     return Err(MarketError::NotEnoughQuantity());
        // }

        // // Check price
        // let market_price = good_meta.sell_price;
        // if p < market_price {
        //     return Err(MarketError::OfferTooLow());
        // }

        // // Create token TODO: Maybe find a better way
        // let mut hasher = DefaultHasher::new();
        // (g.to_string(), p.to_string(), q.to_string(), d).hash(&mut hasher);
        // let token = hasher.finish().to_string();

        // // Update quantity locked and available
        // good_meta.quantity_locked = q;
        // good_meta.quantity_available -= q;
        // good_meta.token = token.clone();
        // good_meta.price_locked = p;

        // Ok(token)
        todo!();
    }

    fn buy(
        &mut self,
        token: String,
        cash: &mut Good,
    ) -> Result<Good, unitn_market_2022::market::BuyError> {
        // Get GoodMeta from token
        //         let (good_kind, mut good_meta) = self.meta.get_mut_good_meta_from_token(token)?;

        //         // Check if locked
        //         if !good_meta.is_locked() {
        //             return Err(MarketError::GoodNotLocked());
        //         }

        //         // Check cash
        //         if cash.get_kind() != GoodKind::DEFAULT_GOOD_KIND {
        //             return Err(MarketError::CashNotDefaultGood());
        //         }
        //         if cash.get_q() < good_meta.quantity_locked / good_meta.price_locked {
        //             return Err(MarketError::TooFewQuantityGiven());
        //         } // Meaning we accept more cash if given TODO: check

        //         // Take the cash
        //         let eur = self
        //             .goods
        //             .iter_mut()
        //             .find(|g| g.get_kind() == GoodKind::DEFAULT_GOOD_KIND)
        //             .unwrap();
        //         eur.merge(cash).unwrap();

        //         let mut good = self
        //             .goods
        //             .iter_mut()
        //             .find(|g| g.get_kind() == *good_kind)
        //             .unwrap(); // TODO: deal with error or write proper method
        //         let good_to_give = good.split(good_meta.quantity_locked).unwrap(); // TODO: deal with error
        //                                                                            // Update good and meta
        //         good_meta.quantity_locked = 0f32;
        //         good_meta.token = "".to_string();
        //         // TODO: Change prices in good_meta..

        //         Ok(good_to_give)
        todo!()
    }

    fn lock_sell(
        &mut self,
        kind_to_sell: GoodKind,
        quantity_to_sell: f32,
        offer: f32,
        trader_name: String,
    ) -> Result<String, unitn_market_2022::market::LockSellError> {
        todo!()
    }

    fn sell(
        &mut self,
        token: String,
        good: &mut Good,
    ) -> Result<Good, unitn_market_2022::market::SellError> {
        todo!()
    }
}

impl Notifiable for SOLMarket {
    fn add_subscriber(&mut self, subscriber: Box<dyn Notifiable>) {
        todo!()
    }

    fn on_event(&mut self, event: unitn_market_2022::event::event::Event) {
        todo!()
    }
}
