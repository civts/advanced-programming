use lib::{domain::market_meta::MarketMeta, market::sol_market::SOLMarket};
use plotters::{
    prelude::{
        BitMapBackend, ChartBuilder, Circle, IntoDrawingArea, IntoLinspace, IntoSegmentedCoord,
        LabelAreaPosition, PathElement, Rectangle,
    },
    series::{Histogram, LineSeries},
    style::{Color, BLUE, GREEN, RED, WHITE},
};
use probability::prelude::{Gaussian, Sample};
use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;
use unitn_market_2022::event::notifiable::Notifiable;
use unitn_market_2022::{good::good_kind::GoodKind, market::Market, wait_one_day};

use crate::lib::domain::price_state::ChaCha20Rngg;

mod lib;
mod tests;

fn main() {
    let sd = 0.25;

    let mut rrng = ChaCha20Rngg::new();
    let gaus = Gaussian::new(0.0, sd);
    let mut random_points = Vec::new();
    for _ in 0..1000 {
        random_points.push(gaus.sample(&mut rrng));
    }

    let root = BitMapBackend::new("./gaus.png", (1024, 768)).into_drawing_area();

    root.fill(&WHITE).unwrap();

    let mut chart = ChartBuilder::on(&root)
        .margin(5)
        .caption("1D Gaussian Distribution Demo", ("sans-serif", 30))
        .set_label_area_size(LabelAreaPosition::Left, 60)
        .set_label_area_size(LabelAreaPosition::Bottom, 60)
        .set_label_area_size(LabelAreaPosition::Right, 60)
        .build_cartesian_2d(-4f64..4f64, 0f64..0.1)
        .unwrap()
        .set_secondary_coord(
            (-4f64..4f64).step(0.1).use_round().into_segmented(),
            0u32..500u32,
        );

    chart
        .configure_mesh()
        .disable_x_mesh()
        .disable_y_mesh()
        .y_label_formatter(&|y| format!("{:.0}%", *y * 100.0))
        .y_desc("Percentage")
        .draw()
        .unwrap();

    chart.configure_secondary_axes().y_desc("Count").draw().unwrap();

    let actual = Histogram::vertical(chart.borrow_secondary())
        .style(GREEN.filled())
        .margin(3)
        .data(random_points.iter().map(|x| (*x, 1)));

    chart
        .draw_secondary_series(actual)
        .unwrap()
        .label("Observed")
        .legend(|(x, y)| Rectangle::new([(x, y - 5), (x + 10, y + 5)], GREEN.filled()));

    let pdf = LineSeries::new(
        (-400..400).map(|x| x as f64 / 100.0).map(|x| {
            (
                x,
                (-x * x / 2.0 / sd / sd).exp() / (2.0 * std::f64::consts::PI * sd * sd).sqrt()
                    * 0.1,
            )
        }),
        &RED,
    );

    chart
        .draw_series(pdf)
        .unwrap()
        .label("PDF")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], RED.filled()));

    chart.configure_series_labels().draw().unwrap();

    // To avoid the IO failure being ignored silently, we manually call the present function
    root.present().expect("Unable to write result to file, please make sure 'plotters-doc-data' dir exists under current dir");
    println!("Result has been saved to ./gaus.png");
}
