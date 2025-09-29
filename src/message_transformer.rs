use rand::distr::weighted::WeightedIndex;
use rand::prelude::*;
use std::sync::atomic::{AtomicU32, Ordering};

// Global counter for consecutive instrument messages
static CONSECUTIVE_INSTRUMENT_COUNT: AtomicU32 = AtomicU32::new(0);

pub fn convert_dot_message() -> String {
    let mut rng = rand::rng();
    let mut selected = Vec::new();
    let choices = ['1', '2', '3'];
    let weights = [70, 20, 10];
    selected.push('<');

    let current_count = CONSECUTIVE_INSTRUMENT_COUNT.load(Ordering::SeqCst);
    let should_send_lamps = current_count >= 5;

    if should_send_lamps {
        for _instrument in 1..=20 {
            selected.push('0');
        }

        for _lamp in 1..=6 {
            if rng.random_bool(0.3) {
                selected.push('1');
            } else {
                selected.push('0');
            }
        }

        CONSECUTIVE_INSTRUMENT_COUNT.store(0, Ordering::SeqCst);
    } else {
        let mut has_instruments = false;

        // The probability of the dot chars:
        // 70% equals to 1
        // 20% equals to 2
        // 10% equals to 3
        for _percussion in 1..=12 {
            if rng.random_bool(0.12) {
                let dist = WeightedIndex::new(weights).unwrap();
                let choice = choices[dist.sample(&mut rng)];
                selected.push(choice);
                has_instruments = true;
            } else {
                selected.push('0')
            }
        }

        for _string in 1..=8 {
            selected.push('0')
        }

        for _lamp in 1..=6 {
            selected.push('0');
        }

        // The probability of all the instrument chars equal to 0 - we pick randomly one to replace
        if selected[1..=20].iter().all(|&c| c == '0') {
            let idx = rng.random_range(1..=12);
            let dist = WeightedIndex::new(weights).unwrap();
            let choice = choices[dist.sample(&mut rng)];
            selected[idx] = choice;
            has_instruments = true;
        }

        // Increment counter since we sent instruments
        if has_instruments {
            CONSECUTIVE_INSTRUMENT_COUNT.fetch_add(1, Ordering::SeqCst);
        }
    }

    selected.push('>');
    selected.push('\n');
    selected.iter().collect()
}

pub fn convert_dash_message() -> String {
    let mut rng = rand::rng();
    let mut selected = Vec::new();
    let choices = ['1', '2', '3', '4'];
    let weights = [20, 20, 30, 30];
    selected.push('<');

    let current_count = CONSECUTIVE_INSTRUMENT_COUNT.load(Ordering::SeqCst);
    let should_send_lamps = current_count >= 5 && rng.random_bool(0.3);

    if should_send_lamps {
        for _instrument in 1..=20 {
            selected.push('0');
        }

        for _lamp in 1..=6 {
            if rng.random_bool(0.3) {
                selected.push('2');
            } else {
                selected.push('0');
            }
        }

        CONSECUTIVE_INSTRUMENT_COUNT.store(0, Ordering::SeqCst);
    } else {
        let mut has_instruments = false;

        for _rest in 1..=12 {
            selected.push('0')
        }

        for _percussion in 1..=8 {
            if rng.random_bool(0.2) {
                let dist = WeightedIndex::new(weights).unwrap();
                let choice = choices[dist.sample(&mut rng)];
                selected.push(choice);
                has_instruments = true;
            } else {
                selected.push('0')
            }
        }

        for _lamp in 1..=6 {
            selected.push('0');
        }

        if selected[1..=20].iter().all(|&c| c == '0') {
            let idx = rng.random_range(13..=20);
            let dist = WeightedIndex::new(weights).unwrap();
            let choice = choices[dist.sample(&mut rng)];
            selected[idx] = choice;
            has_instruments = true;
        }

        if has_instruments {
            CONSECUTIVE_INSTRUMENT_COUNT.fetch_add(1, Ordering::SeqCst);
        }
    }

    selected.push('>');
    selected.push('\n');
    selected.iter().collect()
}

pub fn convert_space_message() -> String {
    let mut selected = Vec::new();
    selected.push('<');
    for _any in 1..=26 {
        selected.push('0');
    }
    selected.push('>');
    selected.push('\n');
    selected.iter().collect()
}
