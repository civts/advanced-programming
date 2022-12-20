use crate::sol_market::{SOLMarket, MARKET_NAME};

impl Drop for SOLMarket {
    fn drop(&mut self) {
        println!("Looks like it is time to say farewell my friend 👋");
        self.write_to_file();
        println!("Thank you for using the {} market 😌", MARKET_NAME);
    }
}
