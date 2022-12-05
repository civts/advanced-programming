use super::sol_market::SOLMarket;
use crate::lib::misc::banner::BANNER;
use std::{collections::HashMap, fs, path::Path};
use unitn_market_2022::good::good_kind::GoodKind;

mod sol_file_prefixes {
    pub const COMMENT_PREFIX: &str = "#";
    pub const GOOD_PREFIX: &str = "good ";
}

///This block contains the logic to serialize/deserialize the market to and from a file
impl SOLMarket {
    /// If the market knows about a file, it means it read the state from there.
    /// This function updates such file with the current state of the market.
    pub(crate) fn write_to_file(&self) {
        match &self.meta.file_path {
            Some(pts) => {
                println!("Writing market info to file 📝");
                let path = Path::new(pts);
                let exists = Path::exists(path);
                //If needed, create target file
                if !exists {
                    let parent = Path::parent(path);
                    //If needed, create parent directory
                    if let Some(directory_path) = parent {
                        let parent_exists = Path::exists(directory_path);
                        if !parent_exists {
                            match fs::create_dir_all(directory_path) {
                                Ok(_) => {}
                                Err(_) => {
                                    panic!("Could not create directory for SOL market status file");
                                }
                            }
                        }
                    }
                    println!("SOL Market file at {} does not seem to exist", pts);
                    return;
                }
                //Get string contents
                let contents = self.serialize_to_file_string();
                match fs::write(path, contents) {
                    Ok(_) => {
                        //Success
                    }
                    Err(_) => {
                        println!("SOL market could not write to its file. Next run contents will not be resotred");
                    }
                }
            }
            None => {
                println!("Not writing the market info to file");
            }
        }
    }

    fn serialize_to_file_string(&self) -> String {
        let mut contents = String::new();
        for banner_line in BANNER {
            let mut s = String::from(sol_file_prefixes::COMMENT_PREFIX);
            s += &" ".repeat(4);
            s += banner_line;
            s += "\n";
            contents.push_str(&s);
        }
        contents.push('\n');
        for good in self.goods.iter() {
            contents.push_str(sol_file_prefixes::GOOD_PREFIX);
            let kind = match good.get_kind() {
                GoodKind::EUR => "EUR",
                GoodKind::YEN => "YEN",
                GoodKind::USD => "USD",
                GoodKind::YUAN => "YUAN",
            };
            contents.push_str(kind);
            contents.push(' ');
            contents.push_str(good.get_qty().to_string().as_str());
            contents.push(' ');
            let exchange_rate = self
                .good_labels
                .iter()
                .find(|gl| gl.good_kind.eq(&good.get_kind()))
                .unwrap()
                .exchange_rate_sell;
            contents.push_str(exchange_rate.to_string().as_str());
            contents.push('\n');
        }
        contents
    }

    /// Reads the file at the provided path and optionally returns a tuple with 4
    /// f32 numbers representing, respectively, the amount of euros, yens, dollars
    /// and yuan that the SOL Market represented in that file has.
    ///
    /// If there is an error reading or parsing the file, None is returned.
    pub(crate) fn read_quantities_from_file(path: &Path) -> Option<(f32, f32, f32, f32)> {
        use sol_file_prefixes::*;

        let pts = path.to_str().unwrap_or("invalid path");
        let exists = Path::exists(path);
        if !exists {
            println!("SOL Market file at {} does not seem to exist", pts);
            return None;
        }
        let contents = fs::read_to_string(path)
            .unwrap_or_else(|_| panic!("Should have been able to read the file at {pts}"));
        let mut reading_failed = false;
        let mut goodmap: HashMap<GoodKind, f32> = HashMap::new();
        for (line_number, line) in contents.split('\n').into_iter().enumerate() {
            if line.starts_with(COMMENT_PREFIX) {
                continue;
            } else if line.starts_with(GOOD_PREFIX) {
                let parts = line.replace(GOOD_PREFIX, "");
                let parts: Vec<&str> = parts.split(' ').collect();
                let good_kind = match parts.first() {
                    Some(ticket) => match *ticket {
                        "USD" => GoodKind::USD,
                        "YEN" => GoodKind::YEN,
                        "EUR" => GoodKind::EUR,
                        "YUAN" => GoodKind::YUAN,
                        _ => {
                            println!("Line {line_number} should have a known good kind, but has '{ticket}'");
                            reading_failed = true;
                            break;
                        }
                    },
                    None => {
                        println!("Line {line_number} should declare a good in the correct format");
                        reading_failed = true;
                        break;
                    }
                };
                let quantity: f32 = match parts.get(1) {
                    Some(quantity_str) => {
                        let qty_result = quantity_str.parse();
                        match qty_result {
                            Ok(qt) => qt,
                            Err(_) => {
                                println!("Line {line_number} should have a valid good quantity, but has '{quantity_str}'");
                                reading_failed = true;
                                break;
                            }
                        }
                    }
                    None => {
                        println!("Line {line_number} should declare a good in the correct format");
                        reading_failed = true;
                        break;
                    }
                };
                if quantity < 0.0 {
                    println!("Line {line_number} should not declare a negative good quanity");
                }
                goodmap.insert(good_kind, quantity);
            }
        }
        let usd_qty = *goodmap.get(&GoodKind::USD).unwrap_or(&-1.0);
        if usd_qty < 0.0 {
            println!("Invalid quantity of usd in the SOL market file");
            reading_failed = true;
        }
        let eur_qty = *goodmap.get(&GoodKind::EUR).unwrap_or(&-1.0);
        if eur_qty < 0.0 {
            println!("Invalid quantity of eur in the SOL market file");
            reading_failed = true;
        }
        let yen_qty = *goodmap.get(&GoodKind::YEN).unwrap_or(&-1.0);
        if yen_qty < 0.0 {
            println!("Invalid quantity of yen in the SOL market file");
            reading_failed = true;
        }
        let yuan_qty = *goodmap.get(&GoodKind::YUAN).unwrap_or(&-1.0);
        if yuan_qty < 0.0 {
            println!("Invalid quantity of yuan in the SOL market file");
            reading_failed = true;
        }

        if reading_failed {
            None
        } else {
            Some((eur_qty, yen_qty, usd_qty, yuan_qty))
        }
    }
}
