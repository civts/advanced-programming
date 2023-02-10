use crate::trader::strategies::History;
use crate::trader::SOLTrader;
use std::collections::HashMap;
use unitn_market_2022::good::consts::DEFAULT_GOOD_KIND;
use unitn_market_2022::good::good_kind::GoodKind;

//here we can implement the stategy of the trader
pub(crate) fn make_trade_all_random(trader: &mut SOLTrader, max_qty: i32) {
    use rand::Rng;
    let mut rng = rand::thread_rng();

    let market_names = ["DogeMarket", "PSE_Market", "Baku stock exchange"];
    //select next trade partner
    let name = market_names[rng.gen_range(0..market_names.len())];

    let all_kinds = vec![GoodKind::USD, GoodKind::YEN, GoodKind::YUAN];
    //select next good
    let kind = all_kinds[rng.gen_range(0..all_kinds.len())];
    //select next quantity
    let qty = rng.gen_range(1..max_qty) as f32;
    //trade!

    if rng.gen_range(0..2) == 0 {
        trader.buy_from_market(name.to_owned(), kind, qty);
    } else {
        trader.sell_to_market(name.to_owned(), kind, qty);
    }
}

pub(crate) fn make_best_trade(trader: &mut SOLTrader, buy_deltas: &History, sell_deltas: &History) {
    use rand::Rng;
    let mut rng = rand::thread_rng();

    let (kind_buy, name_buy, b_delta) = get_best_buy_delta(buy_deltas);

    let (kind_sell, name_sell, s_delta) = get_best_sell_delta(sell_deltas);

    //select next quantity
    let qty = rng.gen_range(500..1000) as f32;

    //trade!
    if b_delta.abs() > s_delta {
        trader.buy_from_market(name_buy.to_owned(), kind_buy, qty);
    } else {
        trader.sell_to_market(name_sell.to_owned(), kind_sell, qty);
    }
}

pub(crate) fn make_best_historical_trade(
    trader: &mut SOLTrader,
    h_buy: &History,
    h_sell: &History,
    do_nothing_count: &mut i32,
) {
    let (kind_buy, name_buy, b_delta) = get_best_buy_delta_from_historical_avg(h_buy);

    let (kind_sell, name_sell, s_delta) = get_best_sell_delta_from_historical_avg(h_sell);

    //select next quantity
    let std_qty = 10.0;
    let threshold = 1.05;

    //new condition: if the delta is too small, don't make any trade
    //but if you haven't made any trade for too long, then force a trade to shuffle the markets
    if (*do_nothing_count < 5 && (b_delta > threshold || s_delta > threshold))
        || (*do_nothing_count >= 5)
    {
        *do_nothing_count = 0;

        if b_delta.abs() > s_delta {
            println!("buy {} {} {}", b_delta, name_buy, kind_buy);

            let mut qty = {
                if b_delta > 5.0 {
                    std_qty * b_delta
                } else if b_delta > 1.0 {
                    std_qty * b_delta.powi(2).round()
                } else {
                    std_qty
                }
            };

            let upperbound = trader.get_cur_good_qty_from_market(&kind_buy, name_buy.clone()) / 2.0; //upperbound for buy is the market qty
            if qty > upperbound {
                qty = upperbound;
            }

            println!("qty {}", qty);
            trader.buy_from_market(name_buy.to_owned(), kind_buy, qty);
        } else {
            println!("sell {} {} {}", s_delta, name_sell, kind_sell);

            let mut qty = {
                if s_delta > 5.0 {
                    std_qty * s_delta
                } else if s_delta > 1.0 {
                    std_qty * s_delta.powi(2).round()
                } else {
                    std_qty
                }
            };
            //upperbound for sell is my own qty
            let upperbound = trader.get_cur_good_qty(&kind_sell) / 2.0;
            if qty > upperbound {
                qty = upperbound;
            }

            println!("qty {}", qty);

            trader.sell_to_market(name_sell.to_owned(), kind_sell, qty);
        }
    } else {
        *do_nothing_count += 1;
        println!("i'm doing nothing today");
        trader.all_wait_one_day();
    }
}

pub(crate) fn fake_trade(trader: &SOLTrader) {
    trader.all_wait_one_day();
}

//tested: delta is zero qith no trades
pub(crate) fn get_delta_last_day(
    history: Vec<HashMap<String, HashMap<GoodKind, f32>>>,
) -> Option<HashMap<String, HashMap<GoodKind, f32>>> {
    if history.len() >= 2 {
        // if at least one day has passed

        let mut delta: HashMap<String, HashMap<GoodKind, f32>> = HashMap::new();

        let yesterday = history[history.len() - 1].clone();

        for (name, abc) in history[history.len() - 2].clone() {
            let mut tmp: HashMap<GoodKind, f32> = HashMap::new();

            for (good, rate) in abc {
                tmp.insert(good.clone(), rate - yesterday[&name][&good]);
            }

            delta.insert(name.clone(), tmp);
        }

        return Some(delta);
    }
    None
}

pub(crate) fn get_historical_average(
    h: &History,
) -> Option<HashMap<String, HashMap<GoodKind, f32>>> {
    if h.len() >= 2 {
        let mut avg: HashMap<String, HashMap<GoodKind, f32>> = h[0].clone();
        let days = h.len() as f32;

        for day in h[1..].iter() {
            for (market, rates) in day {
                for (good, single_rate) in rates {
                    let tmp = avg.get_mut(&market[..]).unwrap().get_mut(good).unwrap();
                    *tmp += *single_rate;
                }
            }
        }

        for (market, rates) in avg.clone() {
            for (good, _) in rates {
                let tmp = avg.get_mut(&market[..]).unwrap().get_mut(&good).unwrap();
                *tmp /= days;
            }
        }

        return Some(avg);
    }
    None
}

pub(crate) fn get_delta_from_historical_avg(
    h: &History,
) -> Option<HashMap<String, HashMap<GoodKind, f32>>> {
    if h.len() >= 2 {
        let mut delta = h[h.len() - 1].clone(); //assign last day
        let mut avg = get_historical_average(h).unwrap();

        for (market, rates) in avg.clone() {
            for (good, _) in rates {
                let tmp = delta.get_mut(&market[..]).unwrap().get_mut(&good).unwrap();
                let tmp2 = avg.get_mut(&market[..]).unwrap().get_mut(&good).unwrap();
                *tmp = tmp.abs() - tmp2.abs();
            }
        }

        return Some(delta);
    }
    None
}

pub(crate) fn get_best_buy_delta_from_historical_avg(h: &History) -> (GoodKind, String, f32) {
    let delta_buy = get_delta_from_historical_avg(h).unwrap();
    let mut res_kind: GoodKind = GoodKind::USD;
    let mut res_market: String = String::from("DogeMarket");
    let mut min_found: f32 = delta_buy["DogeMarket"][&GoodKind::USD];

    for (market, rates) in delta_buy {
        for (good, delta) in rates {
            if good != DEFAULT_GOOD_KIND {
                if delta < min_found {
                    res_kind = good;
                    res_market = market.clone();
                    min_found = delta;
                }
            }
        }
    }

    //return the abs()
    (res_kind, res_market, min_found.abs())
}

//again i could have made only one function, but i would have had to encode the selected operation somehow. it's just cleaner this way
pub(crate) fn get_best_sell_delta_from_historical_avg(h: &History) -> (GoodKind, String, f32) {
    let delta_sell = get_delta_from_historical_avg(h).unwrap();
    let mut res_kind: GoodKind = GoodKind::USD;
    let mut res_market: String = String::from("DogeMarket");
    let mut max_found: f32 = delta_sell["DogeMarket"][&GoodKind::USD];

    for (market, rates) in delta_sell {
        for (good, delta) in rates {
            if good != DEFAULT_GOOD_KIND {
                //it's important that i use no abs() here
                if delta > max_found {
                    res_kind = good;
                    res_market = market.clone();
                    max_found = delta;
                }
            }
        }
    }

    (res_kind, res_market, max_found)
}

pub(crate) fn show_delta() {}

pub(crate) fn get_best_sell_delta(h: &History) -> (GoodKind, String, f32) {
    let delta = &h[h.len() - 1];
    let mut res_kind: GoodKind = GoodKind::USD;
    let mut res_market: String = String::from("DogeMarket");
    let mut max_found: f32 = delta["DogeMarket"][&GoodKind::USD];

    for (market, map) in delta {
        for (good, delta) in map {
            if *good != DEFAULT_GOOD_KIND {
                if *delta > max_found {
                    res_kind = *good;
                    res_market = market.clone();
                    max_found = *delta;
                }
            }
        }
    }
    (res_kind, res_market, max_found)
}

pub(crate) fn get_best_buy_delta(h: &History) -> (GoodKind, String, f32) {
    let delta = &h[h.len() - 1];
    let mut res_kind: GoodKind = GoodKind::USD;
    let mut res_market: String = String::from("DogeMarket");
    let mut min_found: f32 = delta["DogeMarket"][&GoodKind::USD];

    for (market, map) in delta {
        for (good, delta) in map {
            if *good != DEFAULT_GOOD_KIND {
                if *delta < min_found {
                    res_kind = *good;
                    res_market = market.clone();
                    min_found = *delta;
                }
            }
        }
    }
    (res_kind, res_market, min_found)
}
