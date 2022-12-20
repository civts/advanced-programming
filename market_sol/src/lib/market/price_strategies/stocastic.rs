use probability::{
    prelude::{Gaussian, Sample},
    source::Source,
};
use rand::{
    distributions::uniform::{SampleRange, SampleUniform},
    Rng, SeedableRng,
};
use rand_chacha::ChaCha20Rng;
use std::collections::HashMap;
use unitn_market_2022::good::{
    consts::{
        DEFAULT_EUR_USD_EXCHANGE_RATE, DEFAULT_EUR_YEN_EXCHANGE_RATE,
        DEFAULT_EUR_YUAN_EXCHANGE_RATE, DEFAULT_GOOD_KIND,
    },
    good_kind::GoodKind,
};

///How long a season can be, max
pub(crate) const MAX_SEASON_LENGTH: u32 = 65;
pub(crate) const MIN_SEASON_LENGTH: u32 = 20;
pub(crate) const MIN_VARIATION_IN_SEASON: f32 = 0.3;
pub(crate) const MAX_NOISE_CLAMP: f32 = 1.0;
pub(crate) const MIN_NOISE_CLAMP: f32 = -MAX_NOISE_CLAMP;

///Holds all the info that we need to determine the price of a good on a given day
#[derive(Debug)]
pub(crate) struct StocasticPrice {
    last_price: HashMap<GoodKind, f32>,
    day_price: HashMap<GoodKind, (u32, f32)>,
    pub(crate) seasons: HashMap<GoodKind, Season>,
    rand: ChaCha20Rngg,
    gaus: Gaussian,
    max_increase_in_season: f32,
    max_decrease_per_season: f32,
    //debug_only
    pub(crate) past_seasons: HashMap<GoodKind, Vec<Season>>,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct Season {
    pub(crate) starting_day: u32,
    duration: u32,
    pub(crate) starting_price: f32,
    ending_price: f32,
}

#[derive(Debug)]
struct ChaCha20Rngg {
    rng: ChaCha20Rng,
}

impl ChaCha20Rngg {
    fn new() -> Self {
        ChaCha20Rngg {
            rng: ChaCha20Rng::from_entropy(),
        }
    }

    fn gen_range<T, R>(&mut self, range: R) -> T
    where
        T: SampleUniform,
        R: SampleRange<T>,
    {
        assert!(!range.is_empty(), "cannot sample empty range");
        range.sample_single(&mut self.rng)
    }
}

impl Source for ChaCha20Rngg {
    fn read_u64(&mut self) -> u64 {
        self.rng.gen_range(0..=u64::MAX)
    }
}

impl Season {
    fn new(
        current_day: u32,
        rng: &mut ChaCha20Rngg,
        gauss: Gaussian,
        starting_price: f32,
        max_decrease: f32,
        max_increase: f32,
    ) -> Self {
        let duration = rng.gen_range(MIN_SEASON_LENGTH..=MAX_SEASON_LENGTH);
        let intensity = gauss.sample(rng) as f32;
        let change_percentage = intensity.clamp(-max_decrease, max_increase) as f32;
        let change = starting_price * change_percentage;
        let final_price = starting_price + change;
        let final_price = f32::max(final_price, 0.0);
        println!(">>>>>>>>>>> New season");
        println!("Today is the {current_day}");
        let d = current_day + duration;
        println!("End: {d}");
        println!("Starting price: {starting_price}");
        println!("Target price: {final_price}");
        let cp = change_percentage * 100.0;
        println!("Change %: {cp}");
        Season {
            starting_day: current_day,
            duration,
            starting_price,
            ending_price: final_price,
        }
    }

    ///Returns the day when this season ends
    fn end(&self) -> u32 {
        self.starting_day + self.duration
    }

    fn get_price(&self, day: u32, random_for_noise: f64) -> f32 {
        let passed_since_start = day - self.starting_day;
        let perc = passed_since_start as f32 / (self.duration as f32);
        let price_diff = self.ending_price - self.starting_price;
        let price = self.starting_price + (price_diff * perc);
        let noise: f32 =
            (random_for_noise as f32).clamp(MIN_NOISE_CLAMP, MAX_NOISE_CLAMP) * price_diff;
        f32::max(price + noise, 0.05)
    }
}

impl StocasticPrice {
    pub(crate) fn new() -> Self {
        let mut rng = ChaCha20Rngg::new();
        let max_decrease_per_season = rng.gen_range(MIN_VARIATION_IN_SEASON..0.95);
        let max_increase_in_season = rng.gen_range(MIN_VARIATION_IN_SEASON..5.0);
        StocasticPrice {
            last_price: HashMap::new(),
            seasons: HashMap::new(),
            rand: rng,
            gaus: Gaussian::new(0.0, 0.25),
            max_decrease_per_season,
            max_increase_in_season,
            past_seasons: HashMap::new(),
            day_price: HashMap::new(),
        }
    }

    ///Returns the price of (a unit of) the given goodkind on the given day
    pub(crate) fn get_rate(&mut self, good_kind: &GoodKind, day: u32) -> f32 {
        //If we talk about the default good, its price is one.
        if *good_kind == DEFAULT_GOOD_KIND {
            1.0
        } else {
            let day_price_opt = self.day_price.get(good_kind);
            let already_have_price_for_today =
                day_price_opt.map(|tuple| tuple.0 == day).unwrap_or(false);
            //If we already generated the price for today, we return that
            if already_have_price_for_today {
                day_price_opt.unwrap().1
            } else {
                //Else we generate a new one
                let random = self.gaus.sample(&mut self.rand);

                let current_season = self.get_current_season(good_kind, day);
                let price = current_season.get_price(day, random);
                self.day_price.insert(*good_kind, (day, price));
                price
            }
        }
    }

    fn latest_price(&self, gk: &GoodKind) -> f32 {
        let historic = self.last_price.get(gk);
        let default_price: f32 = match gk {
            &DEFAULT_GOOD_KIND => 1.0,
            GoodKind::YEN => DEFAULT_EUR_YEN_EXCHANGE_RATE,
            GoodKind::USD => DEFAULT_EUR_USD_EXCHANGE_RATE,
            GoodKind::YUAN => DEFAULT_EUR_YUAN_EXCHANGE_RATE,
        };
        *historic.unwrap_or(&default_price)
    }

    /// Returns the current season, for the given goodkind on the given day.
    /// If, for the given goodkind, the current season is ended (or not
    /// present), a new season is created (and returned).
    fn get_current_season(&mut self, good_kind: &GoodKind, day: u32) -> &Season {
        let season_opt = self.seasons.get(good_kind);
        let need_new_season = match season_opt {
            Some(s) => {
                let finished = day >= s.end();
                if finished {
                    self.last_price.insert(*good_kind, s.ending_price);
                }
                finished
            }
            None => true,
        };
        if need_new_season {
            let latest_price = self.latest_price(good_kind);
            let new_season = Season::new(
                day,
                &mut self.rand,
                self.gaus,
                latest_price,
                self.max_decrease_per_season,
                self.max_increase_in_season,
            );
            let s = self.seasons.get_mut(good_kind);
            if let Some(ended_season) = s {
                let vec_opt = self.past_seasons.get_mut(good_kind);
                match vec_opt {
                    Some(v) => v.push(*ended_season),
                    None => {
                        let v: Vec<Season> = Vec::from_iter([*ended_season]);
                        self.past_seasons.insert(*good_kind, v);
                    }
                }
            }
            self.seasons.insert(*good_kind, new_season);
        }
        let season = self
            .seasons
            .get(good_kind)
            .expect("A valid season must exist now");
        season
    }
}
