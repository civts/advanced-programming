use super::sol_market::{SOLMarket, TOKEN_DURATION};
use unitn_market_2022::{
    event::{
        event::{Event, EventKind},
        notifiable::Notifiable,
    },
    good::{consts::DEFAULT_GOOD_KIND, good_kind::GoodKind},
};

impl Notifiable for SOLMarket {
    fn add_subscriber(&mut self, subscriber: Box<dyn Notifiable>) {
        self.subscribers.push(subscriber);
    }

    fn on_event(&mut self, event: Event) {
        match event.kind {
            EventKind::Bought => {
                //Update price after successful buy, slightly decrease the price as qnty increases
                self.good_labels.iter_mut().for_each(|gl| {
                    if gl.good_kind.eq(&event.good_kind) {
                        gl.exchange_rate_sell *= 1.05;
                    }
                });
            }

            EventKind::Sold => {
                //Update price after successful sell, slightly increase the price as qnty increases
                // i'm just chaniging the price :/
                self.good_labels.iter_mut().for_each(|gl| {
                    if gl.good_kind.eq(&event.good_kind) {
                        gl.exchange_rate_buy *= 0.95;
                        // println!("ciaoo {}", gl.exchange_rate_buy);
                    }
                });
            }

            EventKind::LockedBuy => {}
            EventKind::LockedSell => {}
            EventKind::Wait => {
                // change some exchange rate -> buy_prices - as for now it's enough to decrease the price a bit
                // as time goes on with goods left unsold you tend to decrease the price
                self.good_labels.iter_mut().for_each(|gl| {
                    if gl.good_kind.ne(&GoodKind::EUR) {
                        gl.exchange_rate_sell *= 1.05;
                    }
                });
            }
        }
        //progress one day in any case
        self.meta.current_day += 1;

        // Reinstate any good which has an expired token
        for (_, meta) in self.meta.locked_buys.iter() {
            let days_since = self.meta.current_day - meta.created_on;
            if days_since == TOKEN_DURATION {
                let good = self
                    .good_labels
                    .iter_mut()
                    .find(|l| l.good_kind.eq(&meta.kind))
                    .unwrap();
                good.quantity += meta.quantity;
            }
        }
        for (_, meta) in self.meta.locked_sells.iter() {
            let days_since = self.meta.current_day - meta.created_on;
            if days_since == TOKEN_DURATION {
                let default_good = self
                    .good_labels
                    .iter_mut()
                    .find(|l| l.good_kind.eq(&DEFAULT_GOOD_KIND))
                    .unwrap();
                default_good.quantity += meta.price;
            }
        }
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
