use chrono::{DateTime, Utc};
use rand::Rng;
use serde::{Deserialize, Serialize};
use serde_json::to_string;
use std::{
    fs::File,
    io::{self, Read},
};
use std::collections::HashMap;
use std::time::Duration;
use ipc_utils::IPCReceiver;
use ipc_utils::trading_event::TradingEvent;
use ipc_utils::trading_event_details::{TradeType, TradingEventDetails};
use thiserror::Error;
use std::string::String;
use unitn_market_2022::good::good_kind::GoodKind;

const TRADE_PATH: &str = "../../data/trade.json";
const LOCK_PATH: &str = "../../data/lock.json";

const EUR_PATH: &str = "../../data/eur.json";
const YEN_PATH: &str = "../../data/yen.json";
const USD_PATH: &str = "../../data/usd.json";
const YUAN_PATH: &str = "../../data/yuan.json";

const REFRESH_RATE_MILLISECONDS: u64 = 100;

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
    pub timestamp: DateTime<Utc>,
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

#[derive(Serialize, Deserialize, Clone)]
pub struct Balance {
    pub value: f32,
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


// receive trading event, add to locks or trades list
pub fn receive() {
    let mut receiver = IPCReceiver::new(Duration::from_millis(REFRESH_RATE_MILLISECONDS));
    let event = receiver.receive();

    match event {
        Ok(trade) => {
            match trade {
                None => {}
                Some(tradeEvent) => {
                    let trader_state = tradeEvent.trader_state;

                    let map = trader_state.cash;

                    for (kind, value) in map {
                        let balance = Balance { value };
                        save_balance(balance, kind);
                    }

                    let market = tradeEvent.market_name;

                    match tradeEvent.details {
                        TradingEventDetails::AskedLock { successful, trade_type, price, good_kind, quantity } => {
                            save_lock_if_successful(market, successful, trade_type, price, good_kind.to_string(), quantity)
                        }
                        TradingEventDetails::TradeFinalized { trade_type, quantity, price, successful, good_kind } => {
                            save_trade_if_successful(market, trade_type, quantity, price, successful, good_kind.to_string())
                        }
                    };
                    ()
                }
            };
        }
        Err(_) => {
            ()
        }
    };
    ()
}

fn save_lock_if_successful(market: String, successful: bool, trade_type: TradeType, price: f32, good_kind: String, quantity: f32) {
    let operation = get_operation_string(trade_type);

    let lock = Lock { quantity: quantity as i32, good_kind: good_kind.to_string(), market, price, operation, timestamp: Utc::now() };
    if successful {
        save_lock(lock);
    }
}

fn save_trade_if_successful(market: String, trade_type: TradeType, quantity: f32, price: f32, successful: bool, good_kind: String) {
    let operation = get_operation_string(trade_type);

    let trade = Trade { quantity: quantity as usize, good_kind: good_kind.to_string(), market, price, operation, timestamp: Utc::now() };
    if successful {
        save_trade(trade);
    }
}

fn get_operation_string(trade_type: TradeType) -> String {
    match trade_type {
        TradeType::Buy => String::from("BUY"),
        TradeType::Sell => String::from("SELL")
    }
}

pub fn read_trades() -> Result<Vec<Trade>, Error> {
    let mut file = File::open(TRADE_PATH).unwrap();
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

pub fn read_balance(gk: GoodKind) -> Result<Vec<Balance>, Error> {
    let path = ged_path_based_on_gk(gk);
    let mut file = File::open(path).unwrap();
    let mut balances_json = String::new();
    file.read_to_string(&mut balances_json).unwrap();
    let balances: Vec<Balance>;

    if balances_json != "" {
        balances = serde_json::from_str(&balances_json).unwrap();
    } else {
        balances = vec![]
    }

    Ok(balances)
}

pub fn find_latest_balance(gk: GoodKind) -> f32 {
    match read_balance(gk) {
        Ok(balances) => {
            if balances.is_empty() {
                0.00
            } else {
                balances.last().unwrap().value
            }
        }
        Err(_) => {
            println!("Failed to read a balance");
            0.00
        }
    }
}

fn ged_path_based_on_gk(gk: GoodKind) -> &'static str {
    let path = match gk {
        GoodKind::EUR => {
            EUR_PATH
        }
        GoodKind::YEN => {
            YEN_PATH
        }
        GoodKind::YUAN => {
            YUAN_PATH
        }
        GoodKind::USD => {
            USD_PATH
        }
    };
    path
}

pub fn save_trade(trade: Trade) {
    let mut trades: Vec<Trade> = read_trades().unwrap();
    trades.insert(0, trade);
    let trade_json = to_string(&trades).unwrap();
    std::fs::write(TRADE_PATH, trade_json).expect("File corrupted !");
}

pub fn save_lock(lock: Lock) {
    let mut locks: Vec<Lock> = read_locks().unwrap();
    locks.insert(0, lock);
    let lock_json = to_string(&locks).unwrap();
    std::fs::write(LOCK_PATH, lock_json).expect("File corrupted !");
}

fn save_balance(balance: Balance, gk: GoodKind) {
    let mut balances: Vec<Balance> = read_balance(gk).unwrap();
    balances.push(balance);
    let balance_json = to_string(&balances).unwrap();
    std::fs::write(ged_path_based_on_gk(gk), balance_json).expect("File corrupted !");
}

fn _remove_lock_randomly() {
    let mut file = File::open("lock.json").expect("file exists");
    let mut contents = String::new();
    file.read_to_string(&mut contents).expect("this works");

    let mut locks: Vec<Balance> = serde_json::from_str(&contents).expect("should work");
    let mut rng = rand::thread_rng();
    locks.remove(rng.gen_range(0..locks.len() - 1));

    let lock_json = to_string(&locks).unwrap();
    std::fs::write(LOCK_PATH, lock_json).expect("WORKS");
}
