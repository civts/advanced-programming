use std::fs;

use crate::logger::{Logger, TxtStrategy};

#[test]
fn should_log_lock_in_txt() {
    // given
    let logger: Logger = Logger {
        strategy: Box::new(TxtStrategy {}),
    };
    // when
    logger.log_lock(String::from("token"), String::from("EUR"), 30.0, 45.0, String::from("BUY"));
    // then
    let contents =
        fs::read_to_string("log_trader.txt").expect("Should have been able to read the file");
    assert_eq!("LOG INFO - LOCK_BUY token: token, goodKind: EUR, quantity: 30, amount: 45\n".trim(), contents.trim());

    // after
    fs::remove_file("log_trader.txt");
}
