use plotters::{
    prelude::{ChartBuilder, Circle, IntoDrawingArea, LabelAreaPosition, SVGBackend},
    series::LineSeries,
    style::{Color, IntoFont, RGBColor},
};
use rand::Rng;
use std::collections::HashMap;
use unitn_market_2022::{
    event::{event::Event, notifiable::Notifiable},
    good::{consts::DEFAULT_GOOD_KIND, good::Good},
};
use unitn_market_2022::{good::good_kind::GoodKind, market::Market, wait_one_day};

use crate::lib::{
    domain::strategy_name::StrategyName,
    market::{
        price_strategies::stocastic::{
            MAX_NOISE_CLAMP, MAX_SEASON_LENGTH, MIN_NOISE_CLAMP, MIN_SEASON_LENGTH,
            MIN_VARIATION_IN_SEASON,
        },
        sol_market::SOLMarket,
    },
};

const SHOW_STOCHASTIC_PRICE: bool = true;
const SHOW_SEASON_MARKS: bool = SHOW_STOCHASTIC_PRICE;
const SHOW_QUANTITY_PRICE: bool = true;
const SHOW_OTHER_MARKETS_PRICE: bool = true;
const SHOW_OVERALL_PRICE: bool = true;

const PURPLE: RGBColor = RGBColor(84, 13, 110);
const TEAL: RGBColor = RGBColor(22, 152, 115);
const WHITE: RGBColor = RGBColor(246, 247, 235);
const BLACK: RGBColor = RGBColor(9, 10, 12);
const RED: RGBColor = RGBColor(246, 22, 52);
const OCRA: RGBColor = RGBColor(241, 143, 1);
const BLUE: RGBColor = RGBColor(53, 129, 184);
// const PINK: RGBColor = RGBColor(238, 66, 102);
// const ORANGE: RGBColor = RGBColor(205, 106, 19);

// These functions are just to showcase the market, no need for them to be used
// at all times
#[allow(dead_code)]

/// This function allows us to test how is the overall trend of our market,
/// for any good. Ideally, we want this to be a slightly positive percentage.
///
/// Pro tip: if you care about execution speed, disable all prints since the I/O
/// takes much longer than the computation
pub(crate) fn test_overall_market_change_percentage() {
    let mut overall = Vec::new();
    for _ in 0..1000 {
        let sum = 10000.0;
        let days = 3650 / 4;
        let interval = 1;

        for gk in [GoodKind::USD, GoodKind::YEN, GoodKind::YUAN] {
            //Generate data
            let market_ref = SOLMarket::new_with_quantities(sum, sum, sum, sum);
            let mut prices: Vec<f32> = Vec::new();
            let mut min = f32::MAX;
            let mut max = f32::MIN;
            let starting_price = market_ref.borrow().get_sell_price(gk, 1.0).unwrap();
            for _ in 0..days {
                let good_eur_rate = market_ref.borrow().get_sell_price(gk, 1.0).unwrap();
                let price = 1.0 / good_eur_rate; //EUR/Good rate
                prices.push(price);
                min = f32::min(min, price);
                max = f32::max(max, price);
                for _ in 0..interval {
                    wait_one_day!(market_ref);
                }
            }
            let final_price = market_ref.borrow().get_sell_price(gk, 1.0).unwrap();
            overall.push(final_price / starting_price);
        }
        let sum = overall.iter().fold(0.0, |acc, i| acc + i);
        let mean_price_change_overall = (sum / (overall.len() as f32) - 1.0) * 100.0;
        println!("Mean change was {}%", mean_price_change_overall);
    }
}

///Simulates the market plotting the price changes to an svg
/// named test_{current date}.svg
pub(crate) fn cool_graphs() {
    let sum = 10000.0;
    let days = 3650;
    let interval = 1;

    // Config chart
    let date_now = chrono::offset::Local::now();
    let name = format!("./test_{:?}.svg", date_now);
    let margin_bottom = 80;
    let y = 1080;
    let x = 1920;
    let drawing_area = SVGBackend::new(name.as_str(), (x, y)).into_drawing_area();
    drawing_area.fill(&WHITE).unwrap();
    let (top, bottom) = drawing_area.split_vertically(y - margin_bottom);
    let (usd, t2) = top.split_vertically((y - margin_bottom) / 2);
    let half = x as f32 / 2.0;
    let (t3, t4) = t2.split_horizontally(half);
    let (yen, _) = t3.split_horizontally(half * 0.98);
    let (_, yuan) = t4.split_horizontally(half * 0.02);

    let qt = 2.0;
    for gk in [GoodKind::USD, GoodKind::YEN, GoodKind::YUAN] {
        //Generate data
        let mut weights = HashMap::new();
        weights.insert(StrategyName::Stocastic, 1.0);
        weights.insert(StrategyName::Quantity, 1.0);
        weights.insert(StrategyName::Others, 1.0);
        let market_ref = SOLMarket::new_with_quantities_and_path(sum, sum, sum, sum, None, weights);
        let mut prices: Vec<f32> = Vec::new();
        let mut stocastic_prices: Vec<f32> = Vec::new();
        let mut quantity_prices: Vec<f32> = Vec::new();
        let mut other_prices: Vec<f32> = Vec::new();
        let mut min = f32::MAX;
        let mut max = f32::MIN;
        for _ in 0..days {
            let b_price = market_ref.borrow().get_buy_price(gk, qt).unwrap();
            // Simulate a buy on another market which aks twice our prices (i.e., wher EUR is worth half)
            market_ref.borrow_mut().notify_everyone(Event {
                kind: unitn_market_2022::event::event::EventKind::Bought,
                good_kind: gk,
                quantity: qt,
                price: b_price * 2.0,
            });

            let good_eur_rate = market_ref.borrow().get_buy_price(gk, qt).unwrap() / qt;
            let price = 1.0 / good_eur_rate; //EUR/Good rate
            min = f32::min(min, price);
            max = f32::max(max, price);
            prices.push(price);
            other_prices.push(market_ref.borrow().get_other_rate(gk));
            stocastic_prices.push(market_ref.borrow().get_stocastic_rate(gk));
            quantity_prices.push(market_ref.borrow().get_quantity_rate(gk));
            min = f32::min(min, *other_prices.last().unwrap());
            min = f32::min(min, *stocastic_prices.last().unwrap());
            min = f32::min(min, *quantity_prices.last().unwrap());
            max = f32::max(max, *other_prices.last().unwrap());
            max = f32::max(max, *stocastic_prices.last().unwrap());
            max = f32::max(max, *quantity_prices.last().unwrap());

            // Buy the good
            let will_buy = rand::thread_rng().gen_bool(0.55);
            let qt_to_trade = qt * 10.0;
            if will_buy {
                let b_price_r = market_ref.borrow().get_buy_price(gk, qt_to_trade);
                if let Ok(b_price) = b_price_r {
                    let lock_r = market_ref.borrow_mut().lock_buy(
                        gk,
                        qt_to_trade,
                        b_price,
                        String::from("abc"),
                    );
                    if let Ok(lock) = lock_r {
                        let mut cash = Good::new(DEFAULT_GOOD_KIND, f32::MAX);
                        let _ = market_ref.borrow_mut().buy(lock, &mut cash);
                    }
                }
            } else {
                let s_price_r = market_ref.borrow().get_sell_price(gk, qt_to_trade);
                if let Ok(s_price) = s_price_r {
                    let lock_r = market_ref.borrow_mut().lock_sell(
                        gk,
                        qt_to_trade,
                        s_price,
                        String::from("abc"),
                    );
                    if let Ok(lock) = lock_r {
                        let mut cash = Good::new(gk, f32::MAX);
                        let _ = market_ref.borrow_mut().sell(lock, &mut cash);
                    }
                }
            }

            for _ in 0..interval {
                wait_one_day!(market_ref);
            }
        }

        let area = match gk {
            GoodKind::EUR => panic!("eur exchange is always 1"),
            GoodKind::YEN => yen.clone(),
            GoodKind::USD => usd.clone(),
            GoodKind::YUAN => yuan.clone(),
        };
        let dim = match gk {
            GoodKind::EUR => panic!("eur exchange is always 1"),
            GoodKind::YEN => 20.0,
            GoodKind::USD => 40.0,
            GoodKind::YUAN => 20.0,
        };

        let mut drawing_context = ChartBuilder::on(&area)
            .set_label_area_size(LabelAreaPosition::Left, dim)
            .set_label_area_size(LabelAreaPosition::Bottom, dim)
            .set_label_area_size(LabelAreaPosition::Right, dim)
            .set_label_area_size(LabelAreaPosition::Top, dim)
            .caption(
                format!("SOL Market, {DEFAULT_GOOD_KIND}/{gk}"),
                ("sans-serif", dim),
            )
            .build_cartesian_2d(0.0..(days as f32), min..max)
            .unwrap();

        drawing_context.configure_mesh().draw().unwrap();

        if SHOW_QUANTITY_PRICE {
            drawing_context
                .draw_series(LineSeries::new(
                    quantity_prices
                        .into_iter()
                        .enumerate()
                        .map(|t| (t.0 as f32, t.1)),
                    &OCRA,
                ))
                .unwrap();
        }

        if SHOW_STOCHASTIC_PRICE {
            drawing_context
                .draw_series(LineSeries::new(
                    stocastic_prices
                        .into_iter()
                        .enumerate()
                        .map(|t| (t.0 as f32, t.1)),
                    &TEAL,
                ))
                .unwrap();
        }

        if SHOW_OTHER_MARKETS_PRICE {
            drawing_context
                .draw_series(LineSeries::new(
                    other_prices
                        .into_iter()
                        .enumerate()
                        .map(|t| (t.0 as f32, t.1)),
                    &PURPLE,
                ))
                .unwrap();
        }

        if SHOW_OVERALL_PRICE {
            let price_points = LineSeries::new(
                prices.into_iter().enumerate().map(|t| (t.0 as f32, t.1)),
                &BLUE,
            );
            drawing_context.draw_series(price_points).unwrap();
        }

        if SHOW_SEASON_MARKS {
            let meta = &market_ref.borrow().meta;
            let binding = meta.stocastic_price.borrow();
            let hash_map = &binding.past_seasons;
            let mut past_seasons = hash_map.get(&gk).unwrap().clone();
            if let Some(current_season) = binding.seasons.get(&gk) {
                past_seasons.push(*current_season);
            }
            let season_marks = past_seasons.iter().map(|s| {
                Circle::new(
                    // /4 because we have event::bought, lock_buy, buy e wait
                    (s.starting_day as f32 / 4.0, s.starting_price),
                    dim / 20.0,
                    RED.filled(),
                )
            });
            drawing_context.draw_series(season_marks).unwrap();
        }
    }

    let txt = format!(
      "max season length: {}, min season length {}\nmin variation in season {}\nmin noise clamp {}, max noise clamp {}",
      MAX_SEASON_LENGTH, MIN_SEASON_LENGTH, MIN_VARIATION_IN_SEASON,MIN_NOISE_CLAMP,MAX_NOISE_CLAMP
  );
    bottom
        .titled(
            txt.as_str(),
            ("sans-serif", 10).into_font().color(&BLACK.mix(0.5)),
        )
        .unwrap();
}
