use rand::distr::weighted::WeightedIndex;
use rand::prelude::*;
pub fn convert_dot_message() -> String {
    let mut rng = rand::rng();
    let mut selected = Vec::new();
    let choices = ['1', '2', '3'];
    let weights = [70, 20, 10];
    selected.push('<');
    for _percussion in 1..=12 {
        if rng.random_bool(0.12) {
            let dist = WeightedIndex::new(weights).unwrap();
            let choice = choices[dist.sample(&mut rng)];
            selected.push(choice)
        } else {
            selected.push('0')
        }
    }
    if selected.iter().skip(1).all(|&c| c == '0') {
        let idx = rng.random_range(1..13);
        let dist = WeightedIndex::new(weights).unwrap();
        let choice = choices[dist.sample(&mut rng)];
        selected[idx] = choice;
    }
    for _rest in 1..=8 {
        selected.push('0')
    }
    selected.push('>');
    selected.iter().collect()
}

pub fn convert_dash_message() -> String {
    let mut rng = rand::rng();
    let mut selected = Vec::new();
    let choices = ['1', '2', '3', '4'];
    let weights = [30, 30, 20, 20];
    selected.push('<');
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
    if selected.iter().skip(1).all(|&c| c == '0') {
        let idx = rng.random_range(13..21);
        let dist = WeightedIndex::new(weights).unwrap();
        let choice = choices[dist.sample(&mut rng)];
        selected[idx] = choice;
    }
    selected.push('>');
    selected.iter().collect()
}
pub fn convert_space_message() -> String {
    let mut selected = Vec::new();
    selected.push('<');
    for _any in 1..=20 {
        selected.push('0');
    }
    selected.push('>');
    selected.iter().collect()
}
