use showcase::cool_graphs;

mod lib;
mod showcase;
mod tests;

fn main() {
    for _ in 0..10 {
        cool_graphs();
    }
}
