use rand::prelude::SliceRandom;
use rand::Rng;
use rand::SeedableRng;
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

pub fn gen<T>() -> T
where
    rand::distributions::Standard: rand::distributions::Distribution<T>,
{
    RNG.with(|rng| rng.borrow_mut().gen())
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

pub fn gen_printable_string(length: usize) -> String {
    const PRINTABLE_CHARS: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789!@#$%^&*()-_=+[{]};:'\",<.>/?";

    RNG.with(|rng| {
        let mut rng = rng.borrow_mut();
        let random_bytes: Vec<u8> = (0..length)
            .map(|_| PRINTABLE_CHARS[rng.gen_range(0..PRINTABLE_CHARS.len())] as u8)
            .collect();

        String::from_utf8_lossy(&random_bytes).to_string()
    })
}

pub fn shuffle<T>(values: &mut [T]) {
    RNG.with(|rng| values.shuffle(&mut *rng.borrow_mut()));
}
