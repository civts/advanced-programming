use crate::trader::{SOLTrader, KINDS};
use unitn_market_2022::good::consts::DEFAULT_GOOD_KIND;
use unitn_market_2022::good::good_kind::GoodKind;

const MIN_BENEFITS: f32 = 5_000f32; // Set minimum profit to exploit arbitrage
const MIN_MARGIN: f32 = 0.1; // Set minimum margin for exploiting arbitrage

#[derive(Debug, Clone)]
pub struct Arbitrage {
    pub buying_market_name: String,
    pub selling_market_name: String,
    pub good_kind: GoodKind,
    pub qty: f32,
    pub benefits: f32,
    pub margin: f32,
}

impl Arbitrage {
    pub fn new(
        buying_market_name: String,
        selling_market_name: String,
        good_kind: GoodKind,
        qty: f32,
        benefits: f32,
        margin: f32,
    ) -> Self {
        Self {
            buying_market_name,
            selling_market_name,
            good_kind,
            qty,
            benefits,
            margin,
        }
    }
}

pub trait Arbitrages {
    /// Find arbitrage opportunities from every markets the trader is connected to
    fn find_arbitrages(&self) -> Vec<Arbitrage>;

    /// This method exploit a weakness of the PSE market to find an arbitrage opportunity
    ///
    /// Weakness of PSE market:
    /// - When lock buying a null quantity of goods on the market the prices starts to fluctuate a lot after some time,
    /// giving us the opportunity to make some benefits with an arbitrage method.
    fn exploit_pse_market(&mut self);
}

impl Arbitrages for SOLTrader {
    fn find_arbitrages(&self) -> Vec<Arbitrage> {
        let mut arbitrages: Vec<Arbitrage> = Vec::new();
        for (i1, buy_market) in self.markets.iter().enumerate() {
            for (i2, sell_market) in self.markets.iter().enumerate() {
                if i1 == i2 {
                    // If same market, pass
                    continue;
                }
                let buy_market_name = buy_market.borrow().get_name().to_string();
                let sell_market_name = sell_market.borrow().get_name().to_string();
                for kind in &KINDS {
                    if kind.eq(&DEFAULT_GOOD_KIND) {
                        continue;
                    }

                    // Get the maximum qty the trader and markets can trade
                    let max_buy_qty = self.max_buy(&kind, &buy_market_name).unwrap_or(0f32);
                    let max_sell_qty = self.max_sell(&kind, &sell_market_name).unwrap_or(0f32);
                    let max_qty = max_buy_qty.min(max_sell_qty) * 0.50; // 50% just in case the market wants to keep a reserve

                    // Get the Buy and Sell prices
                    // If an error occurs, we set the buy price at the max and the sell price at the min possible
                    let buy_price = buy_market
                        .borrow()
                        .get_buy_price(kind.clone(), max_qty)
                        .unwrap_or(f32::MAX);
                    let sell_price = sell_market
                        .borrow()
                        .get_sell_price(kind.clone(), max_qty)
                        .unwrap_or(f32::MIN_POSITIVE);

                    let benefits = sell_price - buy_price;
                    let margin = benefits / buy_price;

                    // Check if we have an arbitrage
                    if sell_price > buy_price && buy_price > 0f32 && sell_price > 0f32 {
                        arbitrages.push(Arbitrage::new(
                            buy_market_name.clone(),
                            sell_market_name.clone(),
                            kind.clone(),
                            max_qty,
                            benefits,
                            margin,
                        ));
                    }
                }
            }
        }
        arbitrages
    }

    fn exploit_pse_market(&mut self) {
        let pse = self
            .markets
            .iter()
            .find(|&m| m.borrow().get_name().eq(&"PSE_Market".to_string()))
            .unwrap();

        // Make the price fluctuate by lock buying a null quantity
        for k in &KINDS {
            if k.eq(&DEFAULT_GOOD_KIND) {
                continue;
            }
            self.lock_buy_from_market_ref(pse.clone(), *k, 0f32);
        }

        let mut arbitrages = self.find_arbitrages();

        // Get all the arbitrages opportunities and take the worthiest one
        arbitrages.sort_by(|a1, a2| a1.benefits.total_cmp(&a2.benefits));
        let highest_benefits_arbitrage = arbitrages.pop();

        if let Some(arbitrage) = highest_benefits_arbitrage {
            // We are not playing for peanuts
            if arbitrage.benefits < MIN_BENEFITS || arbitrage.margin < MIN_MARGIN {
                return;
            }

            println!("\nFound a worthy arbitrage: {:?}", arbitrage);

            let buy_market_name = arbitrage.buying_market_name.clone();
            let sell_market_name = arbitrage.selling_market_name.clone();
            let kind = &arbitrage.good_kind;
            let qty = arbitrage.qty.clone();

            let buy_market = self
                .get_market_by_name(buy_market_name.clone())
                .unwrap()
                .clone();
            let sell_market = self
                .get_market_by_name(sell_market_name.clone())
                .unwrap()
                .clone();

            let (bid, buy_token) = self.lock_buy_from_market_ref(buy_market.clone(), *kind, qty);
            let (offer, sell_token) = self.lock_sell_to_market_ref(sell_market.clone(), *kind, qty);

            self.buy_from_market_ref(buy_market.clone(), buy_token.clone(), bid, qty, *kind);
            self.sell_to_market_ref(sell_market.clone(), sell_token.clone(), offer, qty, *kind);

            println!("Arbitrage exploited!\n");
        }
    }
}
