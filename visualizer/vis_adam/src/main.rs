mod vizualization;

use vizualization::*;

fn main() {
    let mut viz = Visualization::new();
    viz.start().expect("Visualizer should start!");
}
