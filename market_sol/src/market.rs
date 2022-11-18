use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::ops::Deref;
use std::ptr::hash;
use std::time::SystemTime;
use unitn_market_2022::good::good::Good;
use unitn_market_2022::good::good_kind::GoodKind;
use unitn_market_2022::market::{MarketError, MarketTrait};

pub struct Market {
    name: String,
    budget: Good,
    goods: Vec<Good>,
    meta: MarketMetadata,
}

pub struct MarketMetadata {
    goods_meta: HashMap<GoodKind, GoodMeta>,
}

impl MarketMetadata {
    fn get_mut_good_meta(&mut self, good_kind: &GoodKind) -> Result<&mut GoodMeta, MarketError> {
        if let Some(good_meta) = self.goods_meta.get_mut(good_kind) {
            Ok(good_meta)
        } else {
            Err(MarketError::GeneralError("Good Kind not found".to_string()))
        }
    }

    fn get_mut_good_meta_from_token(&mut self, token: String) -> Result<(&GoodKind, &mut GoodMeta), MarketError> {
        for (kind, meta) in self.goods_meta.iter_mut() {
            if meta.token == token { return Ok((kind, meta))}
        }
        Err(MarketError::GeneralError("Token not found".to_string()))
    }
}

struct GoodMeta {
    sell_price: f32,
    buy_price: f32,
    quantity_locked: f32,
    price_locked: f32,
    quantity_available: f32,
    token: String,
}

impl GoodMeta {
    fn new(price: f32, quantity: f32) -> Self {
        Self {
            sell_price: price,
            buy_price: price * 0.99, // TODO: Come with a better option
            quantity_locked: 0.0,
            price_locked: price,
            quantity_available: quantity,
            token: "".to_string()
        }
    }

    fn is_locked(&self) -> bool {
        self.quantity_locked > 0f32
    }
}

impl Market {
    pub(crate) fn new() -> Market {
        let goods = vec![Good::new(GoodKind::EUR, 100.00),
                         Good::new(GoodKind::YEN, 100.00),
                         Good::new(GoodKind::YUAN, 100.00),
                         Good::new(GoodKind::USD, 100.00)];
        Market {
            name: String::from("SOL"),
            budget: Good::new_default(), // TODO we need to decide on that
            goods: goods.clone(),
            meta: MarketMetadata { goods_meta: goods.into_iter().map(|g|
                (g.get_kind(), GoodMeta::new(1.01, g.get_q()))).collect()
            } // TODO: Change price with g.get_kind().get_default_exchange_rate()
        }
    }
}

impl MarketTrait for Market {
    fn get_market_name(&self) -> String {
        return self.name.clone();
    }

    fn get_budget(&self) -> f32 {
        self.goods.iter().fold(0f32, |acc, good| {
            let value = good.get_q() * self.meta.goods_meta.get(&good.get_kind()).unwrap().sell_price; //TODO: Check
            acc + value
        })
    }

    fn get_goods(&self) -> HashMap<GoodKind, &Good> {
        todo!()
    }

    fn lock_trader_buy_from_market(&mut self, g: GoodKind, p: f32, q: f32, d: String) -> Result<String, MarketError> {
        let mut good_meta = self.meta.get_mut_good_meta(&g)?;

        // Check good is locked
        if good_meta.is_locked() { return Err(MarketError::GoodAlreadyLocked()); }

        // Check quantity
        let quantity_available = good_meta.quantity_available;
        if quantity_available < q { return Err(MarketError::NotEnoughQuantity()); }

        // Check price
        let market_price = good_meta.sell_price;
        if p < market_price { return Err(MarketError::OfferTooLow()); }

        // Create token TODO: Maybe find a better way
        let mut hasher = DefaultHasher::new();
        (g.to_string(), p.to_string(), q.to_string(), d).hash(&mut hasher);
        let token = hasher.finish().to_string();

        // Update quantity locked and available
        good_meta.quantity_locked = q;
        good_meta.quantity_available -= q;
        good_meta.token = token.clone();
        good_meta.price_locked = p;

        Ok(token)
    }

    fn trader_buy_from_market(&mut self, token: String, cash: &mut Good) -> Result<Good, MarketError> {
        // Get GoodMeta from token
        let (good_kind, mut good_meta)  = self.meta.get_mut_good_meta_from_token(token)?;

        // Check if locked
        if !good_meta.is_locked() { return Err(MarketError::GoodNotLocked())}

        // Check cash
        if cash.get_kind() != GoodKind::DEFAULT_GOOD_KIND { return Err(MarketError::CashNotDefaultGood())}
        if cash.get_q() < good_meta.quantity_locked / good_meta.price_locked { return Err(MarketError::TooFewQuantityGiven())} // Meaning we accept more cash if given TODO: check

        // Take the cash
        let eur = self.goods.iter_mut().find(|g| g.get_kind() == GoodKind::DEFAULT_GOOD_KIND).unwrap();
        eur.merge(cash).unwrap();



        let mut good = self.goods.iter_mut().find(|g| g.get_kind() == *good_kind).unwrap(); // TODO: deal with error or write proper method
        let good_to_give = good.split(good_meta.quantity_locked).unwrap(); // TODO: deal with error
        // Update good and meta
        good_meta.quantity_locked = 0f32;
        good_meta.token = "".to_string();
        // TODO: Change prices in good_meta..

        Ok(good_to_give)
    }

    fn lock_trader_sell_to_market(&mut self, g: GoodKind, qty: f32, price: f32, d: String) -> Result<String, MarketError> {
        todo!()
    }

    fn trader_sell_to_market(&mut self, token: String, good: &mut Good) -> Result<Good, MarketError> {
        todo!()
    }
}