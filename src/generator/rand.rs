use chrono::{DateTime, Duration, NaiveDateTime, Utc};
use num_traits::Bounded;
use rand::distributions::uniform::SampleUniform;
use rand::prelude::SliceRandom;
use rand::Rng;
use rand::SeedableRng;
use rand_regex;
use rand_xorshift::XorShiftRng;
use std::cell::RefCell;

thread_local! {
    pub static RNG: RefCell<XorShiftRng> = RefCell::new(XorShiftRng::seed_from_u64(42));
}

pub fn seedrand() {
    seed(rand::rngs::OsRng.gen());
}

pub fn seed(seed: u64) {
    RNG.with(|rng| *rng.borrow_mut() = XorShiftRng::seed_from_u64(seed));
}

pub fn gen_range<T>(low: T, high: T) -> T
where
    T: rand::distributions::uniform::SampleUniform + std::cmp::PartialOrd + Copy,
{
    RNG.with(|rng| rng.borrow_mut().gen_range(low..high))
}

pub fn rand_int<T: Bounded + SampleUniform + std::cmp::PartialOrd>() -> T {
    RNG.with(|rng| rng.borrow_mut().gen_range(T::min_value()..T::max_value()))
}

// This function generates a date and time between 50 years in the past and 50 years in the future.
pub fn rand_date_time() -> String {
    // max 100 years in future
    let end = Utc::now() + Duration::weeks(100 * 52);

    let random_timestamp = RNG.with(|rng| rng.borrow_mut().gen_range(0..end.timestamp()));
    // Convert the random timestamp to a DateTime
    let date_time: DateTime<Utc> = DateTime::from_utc(
        NaiveDateTime::from_timestamp_opt(random_timestamp, 0).unwrap(),
        Utc,
    );
    // Return the DateTime as a string
    date_time.to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
}

pub fn gen_printable_string(length: usize, charset: Option<&[u8]>) -> String {
    const PRINTABLE_CHARS: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789!@#$%^&*()-_=+[{]};:'\",<.>/?";

    let cs = match charset {
        Some(cs) => cs,
        None => PRINTABLE_CHARS,
    };

    RNG.with(|rng| {
        let mut rng = rng.borrow_mut();
        let random_bytes: Vec<u8> = (0..length)
            .map(|_| cs[rng.gen_range(0..cs.len())] as u8)
            .collect();

        String::from_utf8_lossy(&random_bytes).to_string()
    })
}

pub fn shuffle<T>(values: &mut [T]) {
    RNG.with(|rng| values.shuffle(&mut *rng.borrow_mut()));
}

pub fn rand_str_regex(regex: &str) -> String {
    RNG.with(|rng| {
        let gen = rand_regex::Regex::compile(regex, 25)
            .expect(format!("could not compile regex '{}'", &regex).as_str());

        rng.borrow_mut().sample(gen)
    })
}
