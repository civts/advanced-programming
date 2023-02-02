use std::{
    fs::File,
    io::{self, Read},
};
use chrono::{DateTime, Utc};
use rand::Rng;
use serde::{Deserialize, Serialize};
use serde_json::to_string;
use thiserror::Error;

const DB_PATH: &str = "../../data/trade.json";
const LOCK_PATH: &str = "../../data/lock.json";


#[derive(Error, Debug)]
pub enum Error {
    #[error("error reading the DB file: {0}")]
    ReadDBError(#[from] io::Error),
    #[error("error parsing the DB file: {0}")]
    ParseDBError(#[from] serde_json::Error),
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Lock {
    pub operation: String,
    pub quantity: i32,
    pub price: f32,
    pub good_kind: String,
    pub market: String,
    pub token: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Trade {
    pub operation: String,
    pub market: String,
    pub good_kind: String,
    pub quantity: usize,
    pub timestamp: DateTime<Utc>,
    pub price: f32,
}

pub fn read_locks() -> Result<Vec<Lock>, Error> {
    let mut file = File::open(LOCK_PATH).unwrap();
    let mut locks_json = String::new();
    file.read_to_string(&mut locks_json).unwrap();
    let locks: Vec<Lock>;

    if locks_json != "" {
        locks = serde_json::from_str(&locks_json).unwrap();
    } else {
        locks = vec![]
    }

    Ok(locks)
}

pub fn read_trades() -> Result<Vec<Trade>, Error> {
    let mut file = File::open(DB_PATH).unwrap();
    let mut trades_json = String::new();
    file.read_to_string(&mut trades_json).unwrap();
    let trades: Vec<Trade>;

    if trades_json != "" {
        trades = serde_json::from_str(&trades_json).unwrap();
    } else {
        trades = vec![]
    }

    Ok(trades)
}

pub fn add_random_trade_to_db() -> Result<Vec<Trade>, Error> {
    let mut trades: Vec<Trade> = read_trades().unwrap();

    let mut rng = rand::thread_rng();

    let ops = vec!["SELL", "BUY"];
    let markets = vec!["NYC", "PIZZA", "GPW"];
    let good_kind = vec!["EUR", "YEN", "USD"];

    let random_trade = Trade {
        operation: String::from(ops[rng.gen_range(0..2)]),
        market: String::from(markets[rng.gen_range(0..3)]),
        good_kind: String::from(good_kind[rng.gen_range(0..3)]),
        quantity: rng.gen_range(1..100),
        timestamp: Utc::now(),
        price: rng.gen_range(0.01..450.00),
    };

    trades.push(random_trade);

    // Serialize the struct to a JSON string
    let trade_json = to_string(&trades).unwrap();

    // Open a file to save the JSON string
    std::fs::write(DB_PATH, trade_json)?;

    Ok(trades)
}

pub fn add_random_lock_to_db() -> Result<Vec<Lock>, Error> {
    let mut locks: Vec<Lock> = read_locks().unwrap();
    let mut rng = rand::thread_rng();
    let ops = vec!["LOCK SELL", "LOCK BUY"];
    let markets = vec!["NYC", "PIZZA", "GPW"];
    let good_kind = vec!["EUR", "YEN", "USD"];

    let random_lock = Lock {
        operation: String::from(ops[rng.gen_range(0..2)]),
        quantity: rng.gen_range(1..100),
        price: rng.gen_range(0.01..450.00),
        good_kind: String::from(good_kind[rng.gen_range(0..3)]),
        market: String::from(markets[rng.gen_range(0..3)]),
        token: rng.gen_range(10000000..
            999999999).to_string(),
    };

    locks.push(random_lock);

    // Serialize the struct to a JSON string
    let lock_json = to_string(&locks).unwrap();

    // Open a file to save the JSON string
    std::fs::write(LOCK_PATH, lock_json)?;

    Ok(locks)
}

fn _remove_lock_randomly() {
    let mut file = File::open("lock.json").expect("file exists");
    let mut contents = String::new();
    file.read_to_string(&mut contents).expect("this works");

    let mut locks: Vec<Lock> = serde_json::from_str(&contents).expect("should work");
    let mut rng = rand::thread_rng();
    locks.remove(rng.gen_range(0..locks.len() - 1));

    let lock_json = to_string(&locks).unwrap();
    std::fs::write(LOCK_PATH, lock_json).expect("WORKS");
}
