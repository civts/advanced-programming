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
const MAX_SEASON_LENGTH: u16 = 365;
const MIN_SEASON_LENGTH: u16 = 20;
const MIN_VARIATION_IN_SEASON: f32 = 0.3;

///Holds all the info that we need to determine the price of a good on a given day
#[derive(Debug)]
pub(crate) struct PriceState {
    historic_prices: HashMap<GoodKind, Vec<f32>>,
    seasons: HashMap<GoodKind, Season>,
    rand: ChaCha20Rngg,
    gaus: Gaussian,
    max_increase_in_season: f32,
    max_decrease_per_season: f32,
}

#[derive(Debug)]
struct Season {
    starting_day: u32,
    duration: u16,
    starting_price: f32,
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
        let change = starting_price * (intensity.clamp(-max_decrease, max_increase) as f32);
        let final_price = starting_price + change;
        let final_price = f32::max(final_price, 0.001);
        Season {
            starting_day: current_day,
            duration,
            starting_price,
            ending_price: final_price,
        }
    }

    ///Returns the day when this season ends
    fn end(&self) -> u32 {
        self.starting_day + (self.duration as u32)
    }

    fn get_price(&self, day: u32, random_for_noise: f64) -> f32 {
        let passed_since_start = day - self.starting_day;
        let perc = passed_since_start as f32 / (self.duration as f32);
        let price_diff = self.ending_price + self.starting_price;
        let price = self.starting_price + (price_diff * perc);
        let noise: f32 = (random_for_noise as f32 * 2.0).clamp(-1.0, 1.0) * price_diff;
        f32::max(price + noise, 0.0)
    }
}

impl PriceState {
    pub(crate) fn new() -> Self {
        let mut rng = ChaCha20Rngg::new();
        let max_decrease_per_season = rng.gen_range(MIN_VARIATION_IN_SEASON..0.95);
        let max_increase_in_season = rng.gen_range(MIN_VARIATION_IN_SEASON..5.0);
        PriceState {
            historic_prices: HashMap::new(),
            seasons: HashMap::new(),
            rand: rng,
            gaus: Gaussian::new(0.0, 0.25),
            max_decrease_per_season,
            max_increase_in_season,
        }
    }

    ///Returns the price of the given goodkind on the given day
    pub fn get_price(&mut self, good_kind: &GoodKind, day: u32) -> f32 {
        //If we talk about the default good, its price is one.
        if *good_kind == DEFAULT_GOOD_KIND {
            return 1.0;
        }
        let random = self.gaus.sample(&mut self.rand);

        //Get the current season of the market
        let current_season = self.get_current_season(good_kind, day);
        current_season.get_price(day, random)
    }

    fn latest_price(&self, gk: &GoodKind) -> f32 {
        let historic = self.historic_prices.get(gk).and_then(|v| v.last());
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
            Some(s) => s.end() > day,
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
            self.seasons.insert(*good_kind, new_season);
        }
        let season = self
            .seasons
            .get(good_kind)
            .expect("A valid season must exist now");
        season
    }
}
