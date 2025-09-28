use rand::distr::weighted::WeightedIndex;
use rand::prelude::*;
pub fn convert_dot_message() -> String {
    let mut rng = rand::rng();
    let mut selected = Vec::new();
    let choices = ['1', '2', '3'];
    let weights = [70, 20, 10];
    selected.push('<');
    // The probability of the dot chars:
    // 70% equals to 1
    // 20% equals to 2
    // 10% equals to 3
    for _percussion in 1..=12 {
        if rng.random_bool(0.12) {
            let dist = WeightedIndex::new(weights).unwrap();
            let choice = choices[dist.sample(&mut rng)];
            selected.push(choice)
        } else {
            selected.push('0')
        }
    }
    // For dot messages, last 8 chars are all 0s
    for _string in 1..=8 {
        selected.push('0')
    }
    // We have 6 lamps, each lamp has 10% chance to be lighten up
    for _lamp in 1..=6 {
        if rng.random_bool(0.2) {
            selected.push('1')
        } else {
            selected.push('0')
        }
    }
    // The probability of all the instrument chars equal to 0 - we pick randomly one to replace
    if selected[1..=20].to_vec().iter().all(|&c| c == '0') {
        let idx = rng.random_range(1..=12);
        let dist = WeightedIndex::new(weights).unwrap();
        let choice = choices[dist.sample(&mut rng)];
        selected[idx] = choice;
    }
    if selected[21..=26].to_vec().iter().all(|&c| c == '0') {
        let idx = rng.random_range(21..25);
        selected[idx] = '1';
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
    // First 12 chars are 0s
    for _rest in 1..=12 {
        selected.push('0')
    }
    for _percussion in 1..=8 {
        if rng.random_bool(0.2) {
            let dist = WeightedIndex::new(weights).unwrap();
            let choice = choices[dist.sample(&mut rng)];
            selected.push(choice)
        } else {
            selected.push('0')
        }
    }
    // 6 lamps
    for _lamp in 1..=6 {
        if rng.random_bool(0.2) {
            // probability to send each char is 20%
            selected.push('2')
        } else {
            selected.push('0')
        }
    }
    if selected[1..=20].iter().all(|&c| c == '0') {
        let idx = rng.random_range(13..=20);
        let dist = WeightedIndex::new(weights).unwrap();
        let choice = choices[dist.sample(&mut rng)];
        selected[idx] = choice;
    }
    if selected[21..=26].iter().all(|&c| c == '0') {
        let idx = rng.random_range(21..=26);
        selected[idx] = '2';
    }
    selected.push('>');
    selected.push('\n');
    selected.iter().collect()
}
pub fn convert_space_message() -> String {
    let mut selected = Vec::new();
    selected.push('<');
    // All chars become 0s
    for _any in 1..=26 {
        selected.push('0');
    }
    selected.push('>');
    selected.push('\n');
    selected.iter().collect()
}
