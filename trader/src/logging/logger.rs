use std::fs::OpenOptions;
use std::io::Write;

pub struct Logger {
    pub strategy: Box<dyn LoggingStrategy>,
}

impl Logger {
    pub fn log_lock(
        &self,
        token: String,
        goodKind: String,
        quant: f32,
        amount: f32,
        operationType: String,
    ) {
        self.strategy.log_lock(token, goodKind, quant, amount, operationType)
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
    fn log_trade(&self);
}

pub struct PdfStrategy {}

pub struct TxtStrategy {}

impl TxtStrategy {}

impl LoggingStrategy for TxtStrategy {
    
    fn log_lock(&self, token: String, good_kind: String, quant: f32, amount: f32, operation_type: String) {
        let filename = format!("log_trader.txt");
        let mut file = OpenOptions::new()
            .append(true)
            .create(true)
            .open(filename)
            .unwrap();

        let time = chrono::Local::now()
            .format("%y:%m:%d:%H:%M:%S:%3f")
            .to_string();

        if let Err(e) = writeln!(file, "LOG INFO - LOCK_{operation_type} token: {token}, goodKind: {good_kind}, quantity: {quant}, amount: {amount}") {
            eprintln!("Error while writing to file {}", e);
        }
    }

    fn log_trade(&self) {
        todo!()
    }
}
