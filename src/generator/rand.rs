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

pub fn rand_i64() -> i64 {
    RNG.with(|rng| rng.borrow_mut().gen_range(0..i64::MAX))
}

pub fn rand_u64() -> u64 {
    RNG.with(|rng| rng.borrow_mut().gen_range(0..u64::MAX))
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
