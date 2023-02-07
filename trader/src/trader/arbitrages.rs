use crate::trader::arbitrage::{Arbitrage, TradeEvent};
use crate::trader::{SOLTrader, KINDS};
use ipc_utils::trading_event_details::TradeType;
use unitn_market_2022::good::consts::DEFAULT_GOOD_KIND;

#[derive(Debug, Clone)]
pub struct Arbitrages {
    arbitrages: Vec<Arbitrage>,
}

impl Arbitrages {
    /// Find arbitrage opportunities from every markets the trader is connected to
    pub fn find_arbitrages(trader: &SOLTrader) -> Self {
        let mut arbitrages: Vec<Arbitrage> = Vec::new();
        for (i1, buy_market) in trader.markets.iter().enumerate() {
            for (i2, sell_market) in trader.markets.iter().enumerate() {
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
                    let max_buy_qty = trader.max_buy(&kind, &buy_market_name).unwrap_or(0f32);
                    let max_sell_qty = trader.max_sell(&kind, &sell_market_name).unwrap_or(0f32);
                    let max_qty = max_buy_qty.min(max_sell_qty) * 0.80; // 90% just in case the market wants to keep a reserve

                    // Get the Buy and Sell prices
                    // If an error occurs, we set the buy price at the max and the sell price at the min possible
                    let buy_price = buy_market
                        .borrow()
                        .get_buy_price(kind.clone(), max_qty)
                        .unwrap_or(f32::MAX)
                        * 1.15;
                    let sell_price = sell_market
                        .borrow()
                        .get_sell_price(kind.clone(), max_qty)
                        .unwrap_or(f32::MIN_POSITIVE)
                        * 0.85;

                    let benefits = sell_price - buy_price;
                    let margin = benefits / buy_price;

                    // Check if we have an arbitrage
                    if sell_price > buy_price && buy_price > 0f32 && sell_price > 0f32 {
                        arbitrages.push(Arbitrage::new(
                            buy_market_name.clone(),
                            sell_market_name.clone(),
                            kind.clone(),
                            max_qty,
                            buy_price,
                            sell_price,
                            benefits,
                            margin,
                        ));
                    }
                }
            }
        }
        Self { arbitrages }
    }

    /// This method exploit a weakness of the PSE market to find an arbitrage opportunity
    ///
    /// Weakness of PSE market:
    ///     When lock buying a null quantity of goods on the market the prices starts to fluctuate a lot after some time,
    ///     giving us the opportunity to make some benefits with an arbitrage method.
    pub fn exploit_pse_market(&mut self, trader: &mut SOLTrader) {
        let pse = trader
            .markets
            .iter()
            .find(|&m| m.borrow().get_name().eq(&"PSE_Market".to_string()))
            .unwrap();

        // Make the price fluctuate by lock buying a null quantity
        for k in &KINDS {
            if k.eq(&DEFAULT_GOOD_KIND) {
                continue;
            }
            pse.borrow_mut()
                .lock_buy(k.clone(), 0f32, f32::MAX, trader.name.clone())
                .unwrap();
        }

        // Get all the arbitrages opportunities and take the worthiest one
        self.arbitrages
            .sort_by(|a1, a2| a1.benefits.total_cmp(&a2.benefits));
        let highest_benefits_arbitrage = self.arbitrages.pop();

        if let Some(arbitrage) = highest_benefits_arbitrage {
            // We are not playing for peanuts
            if arbitrage.benefits < 5_000f32 || arbitrage.margin < 0.1 {
                return;
            }

            println!("Found a worthy arbitrage {:?}", arbitrage);

            let buy_market_name = arbitrage.buying_market_name.clone();
            let sell_market_name = arbitrage.selling_market_name.clone();
            let kind = arbitrage.good_kind.clone();
            let buy_price = arbitrage.buy_price.clone();
            let sell_price = arbitrage.sell_price.clone();
            let qty = arbitrage.qty.clone();

            let buy_market = trader
                .markets
                .iter()
                .find(|&m| m.borrow().get_name().eq(&buy_market_name.clone()))
                .unwrap();
            let sell_market = trader
                .markets
                .iter()
                .find(|&m| m.borrow().get_name().eq(&sell_market_name.clone()))
                .unwrap();

            // Get bid & offer (-/+5% Because some markets does not integrate their margin in these methods)
            let bid = buy_price;
            let offer = sell_price;

            let error = "Error".to_string();

            let buy_token = buy_market
                .borrow_mut()
                .lock_buy(kind, qty, bid, trader.name.clone())
                .unwrap_or(error.clone());

            arbitrage.log_trade_event(
                &trader,
                TradeEvent::Locked,
                if buy_token.ne(&error) { true } else { false },
                TradeType::Buy,
            );

            let sell_token = sell_market
                .borrow_mut()
                .lock_sell(kind, qty, offer, trader.name.clone())
                .unwrap_or(error.clone());

            arbitrage.log_trade_event(
                &trader,
                TradeEvent::Locked,
                if sell_token.ne(&error) { true } else { false },
                TradeType::Sell,
            );

            if buy_token.eq(&error) || sell_token.eq(&error) {
                return;
            }

            let recv_good = buy_market
                .borrow_mut()
                .buy(buy_token, trader.goods.get_mut(&DEFAULT_GOOD_KIND).unwrap())
                .unwrap();

            trader
                .goods
                .get_mut(&kind)
                .unwrap()
                .merge(recv_good)
                .unwrap();

            arbitrage.log_trade_event(&trader, TradeEvent::Finalized, true, TradeType::Buy);

            let recv_cash = sell_market
                .borrow_mut()
                .sell(sell_token, trader.goods.get_mut(&kind).unwrap())
                .unwrap();

            trader
                .goods
                .get_mut(&DEFAULT_GOOD_KIND)
                .unwrap()
                .merge(recv_cash)
                .unwrap();

            arbitrage.log_trade_event(&trader, TradeEvent::Finalized, true, TradeType::Sell);
            println!("Arbitrage exploited!");
        }
    }
}
