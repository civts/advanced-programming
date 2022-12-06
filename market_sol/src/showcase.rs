use plotters::{
    prelude::{ChartBuilder, Circle, IntoDrawingArea, LabelAreaPosition, SVGBackend},
    series::LineSeries,
    style::{Color, IntoFont, BLACK, BLUE, RED, WHITE},
};
use unitn_market_2022::event::notifiable::Notifiable;
use unitn_market_2022::{good::good_kind::GoodKind, market::Market, wait_one_day};

use crate::lib::{
    domain::{
        market_meta::MarketMeta,
        price_state::{
            MAX_NOISE_CLAMP, MAX_SEASON_LENGTH, MIN_NOISE_CLAMP, MIN_SEASON_LENGTH,
            MIN_VARIATION_IN_SEASON,
        },
    },
    market::sol_market::SOLMarket,
};

//These functions are just to showcase the market, no need for them to be used
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
        let days = 3650;
        let interval = 1;

        for gk in [GoodKind::USD, GoodKind::YEN, GoodKind::YUAN] {
            //Generate data
            let market_ref =
                SOLMarket::new_with_quantities_and_meta(sum, sum, sum, sum, MarketMeta::new());
            let mut prices: Vec<f32> = Vec::new();
            let mut min = f32::MAX;
            let mut max = f32::MIN;
            let starting_price = market_ref.borrow().get_sell_price(gk, 1.0).unwrap();
            for _ in 0..days {
                let price = market_ref.borrow().get_sell_price(gk, 1.0).unwrap();
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

    for gk in [GoodKind::USD, GoodKind::YEN, GoodKind::YUAN] {
        //Generate data
        let market_ref =
            SOLMarket::new_with_quantities_and_meta(sum, sum, sum, sum, MarketMeta::new());
        let mut prices: Vec<f32> = Vec::new();
        let mut min = f32::MAX;
        let mut max = f32::MIN;
        for _ in 0..days {
            let price = market_ref.borrow().get_sell_price(gk, 1.0).unwrap();
            prices.push(price);
            min = f32::min(min, price);
            max = f32::max(max, price);
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
            .caption(format!("SOL Market, {gk}"), ("sans-serif", dim))
            .build_cartesian_2d(0.0..(days as f32), min..max)
            .unwrap();

        drawing_context.configure_mesh().draw().unwrap();

        let price_points = LineSeries::new(
            prices.into_iter().enumerate().map(|t| (t.0 as f32, t.1)),
            &BLUE,
        );
        drawing_context.draw_series(price_points).unwrap();

        let meta = &market_ref.borrow().meta;
        let binding = meta.price_state.borrow();
        let hash_map = &binding.past_seasons;
        let mut past_seasons = hash_map.get(&gk).unwrap().clone();
        if let Some(current_season) = binding.seasons.get(&gk) {
            past_seasons.push(*current_season);
        }
        let season_marks = past_seasons.iter().map(|s| {
            Circle::new(
                (s.starting_day as f32, s.starting_price),
                dim / 10.0,
                RED.filled(),
            )
        });
        drawing_context.draw_series(season_marks).unwrap();
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
