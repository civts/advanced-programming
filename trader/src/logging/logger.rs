use std::fs::OpenOptions;
use std::io::Write;

pub struct Logger {
    pub strategy: Box<dyn LoggingStrategy>,
}

impl Logger {
    pub fn log_lock(
        &self,
        token: String,
        good_kind: String,
        quant: f32,
        amount: f32,
        operation_type: String,
    ) {
        self.strategy
            .log_lock(token, good_kind, quant, amount, operation_type)
    }

    pub fn log_trade(
        &self,
        operation_type: String,
        token: String,
        quant: f32,
        value: f32,
        good_kind: String,
    ) {
        self.strategy
            .log_trade(operation_type, token, quant, value, good_kind)
    }
}

pub trait LoggingStrategy {
    fn log_lock(
        &self,
        token: String,
        good_kind: String,
        quant: f32,
        amount: f32,
        operation_type: String,
    );
    fn log_trade(
        &self,
        operation_type: String,
        token: String,
        quant: f32,
        value: f32,
        good_kind: String,
    );
}

pub struct PdfStrategy {}

pub struct TxtStrategy {}

impl TxtStrategy {}

impl LoggingStrategy for TxtStrategy {
    fn log_lock(
        &self,
        token: String,
        good_kind: String,
        quant: f32,
        amount: f32,
        operation_type: String,
    ) {
        let filename = format!("log_trader.txt");
        let mut file = OpenOptions::new()
            .append(true)
            .create(true)
            .open(filename)
            .unwrap();

        if let Err(e) = writeln!(file, "LOG INFO - LOCK_{operation_type} token: {token}, goodKind: {good_kind}, quantity: {quant}, amount: {amount}") {
            eprintln!("Error while writing to file {}", e);
        }
    }

    fn log_trade(
        &self,
        operation_type: String,
        token: String,
        quant: f32,
        value: f32,
        good_kind: String,
    ) {
        let filename = format!("log_trader.txt");
        let mut file = OpenOptions::new()
            .append(true)
            .create(true)
            .open(filename)
            .unwrap();

        if let Err(e) = writeln!(file, "LOG INFO - {operation_type} token: {token}, quantity: {quant}, value: {value}, goodKind: {good_kind}") {
            eprintln!("Error while writing to file {}", e);
        }
    }
}
