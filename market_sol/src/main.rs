use lib::{domain::market_meta::MarketMeta, market::sol_market::SOLMarket};
use plotters::{
    prelude::{BitMapBackend, ChartBuilder, Circle, IntoDrawingArea, LabelAreaPosition},
    series::LineSeries,
    style::{Color, BLUE, RED, WHITE},
};
use unitn_market_2022::event::notifiable::Notifiable;
use unitn_market_2022::{good::good_kind::GoodKind, market::Market, wait_one_day};

mod lib;
mod tests;

fn main() {
    // Create market
    let sum = 10000.0;
    let market_ref = SOLMarket::new_with_quantities_and_meta(sum, sum, sum, sum, MarketMeta::new());

    let days = 365;
    let interval = 1;
    // Simulate time passing
    let mut prices: Vec<f32> = Vec::new();
    let mut min = f32::MAX;
    let mut max = f32::MIN;
    let gk = GoodKind::YEN;
    for _ in 0..days {
        let price = market_ref.borrow().get_buy_price(gk, 1.0).unwrap();
        prices.push(price);
        min = f32::min(min, price);
        max = f32::max(max, price);
        for _ in 0..interval {
            wait_one_day!(market_ref);
        }
    }
    // Config chart
    let date_now = chrono::offset::Local::now();
    let name = format!("./test_{gk}_{:?}.png", date_now);
    let drawing_area = BitMapBackend::new(name.as_str(), (1920, 1080)).into_drawing_area();
    drawing_area.fill(&WHITE).unwrap();
    let mut drawing_context = ChartBuilder::on(&drawing_area)
        .set_label_area_size(LabelAreaPosition::Left, 40.0)
        .set_label_area_size(LabelAreaPosition::Bottom, 40.0)
        .set_label_area_size(LabelAreaPosition::Right, 40.0)
        .set_label_area_size(LabelAreaPosition::Top, 40.0)
        .caption(format!("SOL Market going, {gk}"), ("sans-serif", 40.0))
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
    let mut past_seasons = binding.past_seasons.get(&gk).unwrap().clone();
    if let Some(current_season) = binding.seasons.get(&gk) {
        past_seasons.push(*current_season);
    }
    let season_marks = past_seasons
        .iter()
        .map(|s| Circle::new((s.starting_day as f32, s.starting_price), 4.0, RED.filled()));
    drawing_context.draw_series(season_marks).unwrap();

    println!("Drawn");
}
