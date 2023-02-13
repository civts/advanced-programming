mod visualization;

use visualization::*;

fn main() {
    let visualisation = Visualization::new();
    visualisation.start().expect("Visualizer should start!");
}
