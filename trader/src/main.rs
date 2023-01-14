//the main runs the trader forever

//take the markets out of the trader into a separate playground object?
//maybe too complicated

use trader::trader::SOLTrader;

pub fn main() {
    println!("ciao");

    let generic_init_quantity = 1000.0;
    let mut trader = SOLTrader::new_with_quantities(generic_init_quantity, 0.0, 0.0, 0.0);
    trader.subscribe_markets_to_one_another();

    trader.show_all_self_quantities();

    trader.show_all_market_info();
}


pub fn make_trade(){
    //select next trade partner

    //select next good

    //select next quantity

    //trade!
}

fn show_delta(){

}