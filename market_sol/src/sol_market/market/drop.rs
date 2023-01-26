use crate::sol_market::SOLMarket;

impl Drop for SOLMarket {
    fn drop(&mut self) {
        self.write_to_file();
    }
}
