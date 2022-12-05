use lib::market::sol_market::SOLMarket;
use plotters::{
    prelude::{BitMapBackend, ChartBuilder, IntoDrawingArea, LabelAreaPosition},
    series::LineSeries,
    style::{BLUE, WHITE},
};
use unitn_market_2022::{good::good_kind::GoodKind, market::Market, wait_one_day};

mod lib;
mod tests;

fn main() {
    // Create market
    let market = SOLMarket::new_random();
    let days = 1000;
    // Simulate time passing
    let mut prices: Vec<f32> = Vec::new();
    let mut min = f32::MAX;
    let mut max = f32::MIN;
    for _ in 0..days {
        let price = market.borrow().get_buy_price(GoodKind::USD, 1.0).unwrap();
        prices.push(price);
        min = f32::min(min, price);
        max = f32::max(max, price);
        for _ in 0..10 {
            wait_one_day!(market);
        }
    }
    // Config chart
    let drawing_area = BitMapBackend::new("./test.png", (1920, 1080)).into_drawing_area();
    drawing_area.fill(&WHITE).unwrap();
    let mut drawing_context = ChartBuilder::on(&drawing_area)
        .set_label_area_size(LabelAreaPosition::Left, 40.0)
        .set_label_area_size(LabelAreaPosition::Bottom, 40.0)
        .set_label_area_size(LabelAreaPosition::Right, 40.0)
        .set_label_area_size(LabelAreaPosition::Top, 40.0)
        .caption("SOL Market going, USD", ("sans-serif", 40.0))
        .build_cartesian_2d(0.0..(days as f32), min..max)
        .unwrap();

    drawing_context.configure_mesh().draw().unwrap();

    let series = LineSeries::new(
        prices.into_iter().enumerate().map(|t| (t.0 as f32, t.1)),
        &BLUE,
    );
    drawing_context.draw_series(series).unwrap();
}
