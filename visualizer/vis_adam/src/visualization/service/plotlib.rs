use ipc_utils::trader_state::ALL_GOOD_KINDS;
use plotlib::page::Page;
use plotlib::repr::Plot;
use plotlib::view::ContinuousView;
use plotlib::style::{LineStyle, PointMarker, PointStyle};
use tui::style::Color;
use unitn_market_2022::good::good_kind::GoodKind;
use unitn_market_2022::good::good_kind::GoodKind::{EUR, YEN};
use crate::visualization::repository::repository::read_balance;

pub fn generate_all_balances_plot() {
    let mut lines = vec![];

    let ops = read_balance(EUR).unwrap().len();
    let mut maxes = vec![];


    ALL_GOOD_KINDS.iter().for_each(|good| {
        maxes.push(get_max_value(good.clone()));
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
        .y_range(0., get_max_from_vec(maxes) as f64)
        .x_range(0., ops as f64)
        .x_label("Number of operations")
        .y_label("Balance");

    // A page with a single view is then saved to an SVG file
    Page::single(&v).dimensions(640, 480).save("summary/balances.svg").unwrap();
}

pub fn plot_for_gk(gk: GoodKind) {

    let balance = read_balance(gk).unwrap();
    let ops = balance.len();
    let max_value = get_max_value(gk);

    let balances: Vec<f32> = read_balance(gk.clone()).unwrap().iter().map(|x| x.value).collect();
    let data: Vec<(f64, f64)> = balances
        .iter()
        .enumerate()
        .map(|(index, balance)| (index as f64, *balance as f64))
        .collect();

    let plot = Plot::new(data).legend(format!("{} Balances", gk)).line_style(
        LineStyle::new()
            .colour(match gk {
                EUR => { "BLUE" }
                YEN => { "RED" }
                USD => { "GREEN" }
                YUAN => { "YELLOW" }
            }));

    let v = ContinuousView::new()
        .add(plot)
        .y_range(0., max_value as f64)
        .x_range(0., ops as f64)
        .x_label("Number of operations")
        .y_label("Balance");

    // A page with a single view is then saved to an SVG file
    Page::single(&v).dimensions(640, 480).save(format!("summary/{}_balance.svg", gk)).unwrap();
}

fn get_max_value(gk: GoodKind) -> f32 {
    let max_value = read_balance(gk).unwrap().iter().map(|b| b.value).fold(0.0, |max, x| {
        if x > max {
            x
        } else {
            max
        }
    });

    if (max_value == 10000.00) {
        return max_value + 10000.00
    }

    max_value
}

fn get_max_from_vec(x: Vec<f32>) -> f32 {
    return x.iter().fold(0.0, |max, &x| {
        if x > max {
            x
        } else {
            max
        }
    });

}
