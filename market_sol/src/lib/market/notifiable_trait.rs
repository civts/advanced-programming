use super::sol_market::{SOLMarket, TOKEN_DURATION};
use unitn_market_2022::{
    event::{
        event::{Event, EventKind},
        notifiable::Notifiable,
    },
    good::{consts::DEFAULT_GOOD_KIND, good::Good},
};

impl Notifiable for SOLMarket {
    fn add_subscriber(&mut self, subscriber: Box<dyn Notifiable>) {
        self.subscribers.push(subscriber);
    }

    fn on_event(&mut self, event: Event) {
        match event.kind {
            EventKind::Bought | EventKind::Sold => {
                let exchange_rate = event.quantity / event.price;
                self.meta
                    .other_markets
                    .update(&event.good_kind, exchange_rate);
                //TODO
                //Update price after successful buy, slightly decrease the price as qnty increases
                // self.good_labels.iter_mut().for_each(|gl| {
                //     if gl.good_kind.eq(&event.good_kind) {
                //         gl.exchange_rate_sell *= 1.05;
                //     }
                // });
            }

            EventKind::LockedBuy => {}
            EventKind::LockedSell => {}
            EventKind::Wait => {
                //TODO
                // // change some exchange rate -> buy_prices - as for now it's enough to decrease the price a bit
                // // as time goes on with goods left unsold you tend to decrease the price
                // self.good_labels.iter_mut().for_each(|gl| {
                //     if gl.good_kind.ne(&GoodKind::EUR) {
                //         gl.exchange_rate_sell *= 1.05;
                //     }
                // });
            }
        }

        // Reinstate any good which has an expired token
        for (_, meta) in self.meta.locked_buys.iter() {
            let days_since = self.meta.current_day - meta.created_on;
            if days_since == TOKEN_DURATION {
                let good = self.goods.get(&meta.kind).unwrap();
                let replenished_good_qty = good.get_qty() + meta.quantity;
                self.goods
                    .insert(meta.kind, Good::new(meta.kind, replenished_good_qty));
            }
        }
        for (_, meta) in self.meta.locked_sells.iter() {
            let days_since = self.meta.current_day - meta.created_on;
            if days_since == TOKEN_DURATION {
                let default_good = self.goods.get(&DEFAULT_GOOD_KIND).unwrap();
                let new_def_good_qty = default_good.get_qty() + meta.price;
                self.goods.insert(
                    DEFAULT_GOOD_KIND,
                    Good::new(DEFAULT_GOOD_KIND, new_def_good_qty),
                );
            }
        }

        // Every 100 days update exporters and importers
        if self.meta.current_day % 100 == 0 {
            let mut goods_vec = Vec::new();
            for (_, g) in self.goods.iter() {
                goods_vec.push(g.clone())
            }
            self.internal_needs = SOLMarket::set_internal_needs(goods_vec);
        }

        // Perform an internal trade if needed
        self.internal_trade_if_needed();

        //progress one day in any case
        self.meta.current_day += 1;
    }
}

impl SOLMarket {
    /// Notify every market including ours of an event
    pub(crate) fn notify_everyone(&mut self, e: Event) {
        for subscriber in &mut self.subscribers {
            subscriber.on_event(e.clone())
        }
        // UNCOMMENT THIS LINE TO NOTIFY YOURSELF TOO, AND NOT ONLY YOUR NEIGHBOURS
        self.on_event(e);
    }
}
