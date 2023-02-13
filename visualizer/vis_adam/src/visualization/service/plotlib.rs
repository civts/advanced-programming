use ipc_utils::trader_state::ALL_GOOD_KINDS;
use plotlib::page::Page;
use plotlib::repr::Plot;
use plotlib::view::ContinuousView;
use plotlib::style::{LineStyle, PointMarker, PointStyle};
use tui::style::Color;
use unitn_market_2022::good::good_kind::GoodKind;
use unitn_market_2022::good::good_kind::GoodKind::{EUR, YEN};
use crate::visualization::repository::repository::read_balance;

pub fn generate_all_balances_plot(max_price: f64, max_ops: f64) {
    let mut lines = vec![];

    let ops = read_balance(EUR).unwrap().len();

    let max_value = read_balance(YEN).unwrap().iter().map(|b| b.value).fold(0.0, |max, x| {
        if x > max {
            x
        } else {
            max
        }
    });

    ALL_GOOD_KINDS.iter().for_each(|good| {
        let balances: Vec<f32> = read_balance(good.clone()).unwrap().iter().map(|x| x.value).collect();
        let data: Vec<(f64, f64)> = balances
            .iter()
            .enumerate()
            .map(|(index, balance)| (index as f64, *balance as f64))
            .collect();


        lines.push(Plot::new(data).legend(format!("{} Balances", good)).line_style(
            LineStyle::new()
                .colour(match good {
                    EUR => { "BLUE" }
                    GoodKind::YEN => { "RED" }
                    GoodKind::USD => { "GREEN" }
                    GoodKind::YUAN => { "YELLOW" }
                })
        ));
    });


    // The 'view' describes what set of data is drawn
    let v = ContinuousView::new()
        .add(lines.get(0).unwrap().clone())
        .add(lines.get(1).unwrap().clone())
        .add(lines.get(2).unwrap().clone())
        .add(lines.get(3).unwrap().clone())
        .y_range(0., max_value as f64)
        .x_range(0., ops as f64)
        .x_label("Number of operations")
        .y_label("Balance");

    // A page with a single view is then saved to an SVG file
    Page::single(&v).dimensions(640, 480).save("summary/balances.png").unwrap();
}
