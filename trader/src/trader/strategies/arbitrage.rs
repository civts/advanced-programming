use crate::trader::{SOLTrader, KINDS};
use unitn_market_2022::good::consts::DEFAULT_GOOD_KIND;
use unitn_market_2022::good::good::Good;
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
    /// Find opportunities (arbitrage, reverse arbitrage) from every markets the trader is connected to
    fn find_opportunities(&self) -> Vec<Arbitrage>;

    /// This method exploit a weakness of the PSE market to find an arbitrage opportunity
    ///
    /// Weakness of PSE market:
    /// - When lock buying a null quantity of goods on the market the prices starts to fluctuate a lot after some time,
    /// giving us the opportunity to make some benefits with an arbitrage method.
    fn exploit_pse_market(&mut self, days: &mut u32);

    /// Make the worst trade possible (lowest negative margin).
    /// Return true if trader's worth < 1 EUR
    fn lose_all(&mut self, day: &mut u32) -> bool;
}

impl Arbitrages for SOLTrader {
    fn find_opportunities(&self) -> Vec<Arbitrage> {
        let mut opportunities: Vec<Arbitrage> = Vec::new();
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
                    let max_buy_qty = self.max_buy(kind, &buy_market_name).unwrap_or(0f32);
                    let max_sell_qty = self
                        .max_sell(kind, &sell_market_name, max_buy_qty)
                        .unwrap_or(0f32);
                    let max_qty = max_buy_qty.min(max_sell_qty) * 0.5;

                    // Get the Buy and Sell prices
                    // If an error occurs, we set the buy price and the sell price at the min possible
                    let buy_price = buy_market
                        .borrow()
                        .get_buy_price(*kind, max_qty)
                        .unwrap_or(f32::MIN);
                    let sell_price = sell_market
                        .borrow()
                        .get_sell_price(*kind, max_qty)
                        .unwrap_or(f32::MIN);

                    let benefits = sell_price - buy_price;
                    let margin = benefits / buy_price;

                    // Consider every prices above 0 (no error)
                    if buy_price > 0f32 && sell_price > 0f32 {
                        opportunities.push(Arbitrage::new(
                            buy_market_name.clone(),
                            sell_market_name.clone(),
                            *kind,
                            max_qty,
                            benefits,
                            margin,
                        ));
                    }
                }
            }
        }
        opportunities
    }

    fn exploit_pse_market(&mut self, days: &mut u32) {
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
            *days += 1;
        }

        let mut opportunities = self.find_opportunities();

        // Get all the opportunities opportunities and take the worthiest one
        opportunities.sort_by(|o1, o2| o1.benefits.total_cmp(&o2.benefits));
        let highest_benefits = opportunities.pop();

        if let Some(arbitrage) = highest_benefits {
            // We are not playing for peanuts
            if arbitrage.benefits < MIN_BENEFITS || arbitrage.margin < MIN_MARGIN {
                return;
            }

            println!("\n*** Found a worthy arbitrage ***\n{:?}", arbitrage);

            let buy_market_name = arbitrage.buying_market_name.clone();
            let sell_market_name = arbitrage.selling_market_name.clone();
            let kind = &arbitrage.good_kind;
            let qty = arbitrage.qty;

            let buy_market = self.get_market_by_name(buy_market_name).unwrap().clone();
            let sell_market = self.get_market_by_name(sell_market_name).unwrap().clone();

            let (bid, buy_token) = self.lock_buy_from_market_ref(buy_market.clone(), *kind, qty);
            let (offer, sell_token) = self.lock_sell_to_market_ref(sell_market.clone(), *kind, qty);

            self.buy_from_market_ref(buy_market, buy_token, bid, qty, *kind);
            self.sell_to_market_ref(sell_market, sell_token, offer, qty, *kind);

            *days += 4;

            println!(
                "Arbitrage profit: {} {}\n",
                arbitrage.benefits, DEFAULT_GOOD_KIND
            );
        }
    }

    fn lose_all(&mut self, day: &mut u32) -> bool {
        if self.get_current_worth() < 1f32 {
            return true;
        }

        // If EUR qty < 1: sell the good with highest qty in order to get some EUR
        if self.get_cur_good_qty(&DEFAULT_GOOD_KIND) < 1f32 {
            self.sell_highest_qty_good();
            *day += 2;
        }

        // Get all the opportunities opportunities and take the worst one
        let mut opportunities = self.find_opportunities();
        opportunities.sort_by(|o1, o2| o2.margin.total_cmp(&o1.margin));
        let lowest_benefits = opportunities.pop();

        if let Some(reverse_arbitrage) = lowest_benefits {
            // Check not profitable
            if reverse_arbitrage.margin > 0f32 {
                return false;
            }

            println!("\n*** Best way to lose money ***\n{:?}", reverse_arbitrage);

            let buy_market_name = reverse_arbitrage.buying_market_name.clone();
            let sell_market_name = reverse_arbitrage.selling_market_name.clone();
            let kind = &reverse_arbitrage.good_kind;
            let qty = reverse_arbitrage.qty;

            let buy_market = self.get_market_by_name(buy_market_name).unwrap().clone();
            let sell_market = self.get_market_by_name(sell_market_name).unwrap().clone();

            let (bid, buy_token) = self.lock_buy_from_market_ref(buy_market.clone(), *kind, qty);
            let (offer, sell_token) = self.lock_sell_to_market_ref(sell_market.clone(), *kind, qty);

            self.buy_from_market_ref(buy_market, buy_token, bid, qty, *kind);
            self.sell_to_market_ref(sell_market, sell_token, offer, qty, *kind);
            *day += 4;

            println!(
                "Money lost: {} {}\n",
                reverse_arbitrage.benefits, DEFAULT_GOOD_KIND
            );
        }
        false
    }
}

impl SOLTrader {
    fn sell_highest_qty_good(&mut self) {
        let mut goods: Vec<Good> = self.goods.values().cloned().collect();
        goods.sort_by(|g1, g2| g1.get_qty().total_cmp(&g2.get_qty()));
        let highest_qty_good = goods.pop().unwrap();
        let kind = highest_qty_good.get_kind();
        let qty = highest_qty_good.get_qty();
        let mut best_market = self.get_market_by_name("PSE_Market".to_string()).unwrap();
        let mut best_sell = 0f32;
        for market in self.markets.iter() {
            let market_name = market.borrow().get_name().to_string();
            let max_sell = self.max_sell(&kind, &market_name, 0f32).unwrap_or(0f32);
            if max_sell > best_sell {
                best_sell = max_sell;
                best_market = market;
            }
        }
        if best_sell == 0f32 {
            return;
        }
        let (offer, sell_token) = self.lock_sell_to_market_ref(best_market.clone(), kind, qty);
        self.sell_to_market_ref(best_market.clone(), sell_token, offer, qty, kind);
    }
}
