use super::sol_market::SOLMarket;
use crate::lib::{
    domain::strategy_name::{StrategyName, ALL_STRATEGY_NAMES},
    market::sol_market::ALL_GOOD_KINDS,
    misc::banner::BANNER,
};
use std::{collections::HashMap, fs, path::Path};
use unitn_market_2022::good::{good::Good, good_kind::GoodKind};

mod sol_file_prefixes {
    pub const COMMENT_PREFIX: &str = "#";
    pub const GOOD_PREFIX: &str = "good ";
    pub const WEIGHT_PREFIX: &str = "weight ";
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
                } else {
                    //The file already exists, let's try to rename it
                    let mut new_name = String::from(path.to_str().unwrap_or("./sol.sol"));
                    let date_now = chrono::offset::Local::now();
                    let last_dot = new_name.rfind('.').unwrap_or(new_name.len() - 1);
                    new_name.insert_str(last_dot, format!("{}", date_now.timestamp()).as_str());
                    let _ = fs::rename(path, new_name);
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
        for good in self.goods.values() {
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
            let exchange_rate = self.get_good_sell_exchange_rate(good.get_kind());
            contents.push_str(exchange_rate.to_string().as_str());
            contents.push('\n');
        }
        contents.push('\n');
        for s in ALL_STRATEGY_NAMES {
            let weight_opt = self.meta.weights.get(&s);
            if let Some(weight) = weight_opt {
                contents.push_str(sol_file_prefixes::WEIGHT_PREFIX);
                contents.push_str(s.to_string().as_str());
                contents.push(' ');
                contents.push_str(weight.to_string().as_str());
                contents.push('\n');
            }
        }
        contents
    }

    pub(crate) fn read_weights_from_file(path: &Path) -> HashMap<StrategyName, f32> {
        use sol_file_prefixes::*;

        if let Some(contents) = get_file_content(path) {
            let mut strategy_weights: HashMap<StrategyName, f32> = HashMap::new();
            for (line_number, line) in contents.split('\n').into_iter().enumerate() {
                if line.starts_with(COMMENT_PREFIX) {
                    continue;
                } else if line.starts_with(WEIGHT_PREFIX) {
                    let parts = line.replace(WEIGHT_PREFIX, "");
                    let parts: Vec<&str> = parts.split(' ').collect();
                    //TODO:Create map with all strategies
                    let strategy_name = parts.first().and_then(|name| match *name {
                        "Stocastic" => Some(StrategyName::Stocastic),
                        "Quantity" => Some(StrategyName::Quantity),
                        "Others" => Some(StrategyName::Others),
                        _ => None,
                    });
                    let weight: Option<f32> = parts.get(1).and_then(|quantity_str| {
                        let weight_result = quantity_str.parse();
                        match weight_result {
                            Ok(w) => Some(w),
                            Err(_) => {
                                println!("Line {line_number} should have a valid good quantity, but has '{quantity_str}'");
                                None
                            }
                        }
                    });
                    match strategy_name {
                        Some(strategy_name) => match weight {
                            Some(weight) => {
                                strategy_weights.insert(strategy_name, weight);
                            }
                            None => break,
                        },
                        None => break,
                    }
                }
            }
            let stocastic_weight = strategy_weights.get(&StrategyName::Stocastic);
            let quantity_weight = strategy_weights.get(&StrategyName::Quantity);
            let others_weight = strategy_weights.get(&StrategyName::Others);
            let all_are_something = [stocastic_weight, quantity_weight, others_weight]
                .into_iter()
                .all(|i| i.is_some());
            if all_are_something {
                let mut m = HashMap::with_capacity(ALL_STRATEGY_NAMES.len());
                m.insert(StrategyName::Stocastic, *stocastic_weight.unwrap());
                m.insert(StrategyName::Quantity, *quantity_weight.unwrap());
                m.insert(StrategyName::Others, *others_weight.unwrap());
                return m;
            }
        }
        HashMap::new()
    }

    /// Reads the file at the provided path and optionally returns vector with the goods
    /// that the SOL Market represented in that file has.
    ///
    /// If there is an error reading or parsing the file, None is returned.
    pub(crate) fn read_quantities_from_file(path: &Path) -> Option<Vec<Good>> {
        use sol_file_prefixes::*;

        if let Some(contents) = get_file_content(path) {
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
                            println!(
                                "Line {line_number} should declare a good in the correct format"
                            );
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
                            println!(
                                "Line {line_number} should declare a good in the correct format"
                            );
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
                let mut goods_vec = Vec::with_capacity(ALL_GOOD_KINDS.len());
                for gk in ALL_GOOD_KINDS {
                    let quantity = match gk {
                        GoodKind::EUR => eur_qty,
                        GoodKind::YEN => yen_qty,
                        GoodKind::USD => usd_qty,
                        GoodKind::YUAN => yuan_qty,
                    };
                    goods_vec.push(Good::new(gk, quantity));
                }
                Some(goods_vec)
            }
        } else {
            None
        }
    }
}

fn get_file_content(path: &Path) -> Option<String> {
    let pts = path.to_str().unwrap_or("invalid path");
    let exists = Path::exists(path);
    if !exists {
        println!("SOL Market file at {} does not seem to exist", pts);
        return None;
    }
    let contents = fs::read_to_string(path)
        .unwrap_or_else(|_| panic!("Should have been able to read the file at {pts}"));
    Some(contents)
}
