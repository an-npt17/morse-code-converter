use crate::send_lamp;
use rand::distr::weighted::WeightedIndex;
use rand::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformerConfig {
    // Tempo configuration
    pub tempo_choices: Vec<u64>,
    pub lamp_tempo_ms: u64,

    // Dot message configuration
    pub dot_percussion_probability: f64,
    pub dot_choice_1_weight: u32,
    pub dot_choice_2_weight: u32,
    pub dot_choice_3_weight: u32,

    // Dash message configuration
    pub dash_string_probability: f64,
    pub dash_choice_1_weight: u32,
    pub dash_choice_2_weight: u32,
    pub dash_choice_3_weight: u32,
    pub dash_choice_4_weight: u32,

    // Lamp probabilities
    pub lamp_probability_when_lamp_mode: f64, // Probability when send_lamp() returns true
    pub lamp_probability_normal: f64,         // Probability when send_lamp() returns false
}

impl Default for TransformerConfig {
    fn default() -> Self {
        TransformerConfig {
            tempo_choices: vec![400, 700, 1000],
            lamp_tempo_ms: 400,

            dot_percussion_probability: 0.12,
            dot_choice_1_weight: 70,
            dot_choice_2_weight: 20,
            dot_choice_3_weight: 10,

            dash_string_probability: 0.2,
            dash_choice_1_weight: 20,
            dash_choice_2_weight: 20,
            dash_choice_3_weight: 30,
            dash_choice_4_weight: 30,

            lamp_probability_when_lamp_mode: 0.2,
            lamp_probability_normal: 0.1,
        }
    }
}

pub fn convert_dot_message(config: &TransformerConfig) -> String {
    let mut rng = rand::rng();
    let mut selected = Vec::new();
    let choices = ['1', '2', '3'];
    let weights = [
        config.dot_choice_1_weight,
        config.dot_choice_2_weight,
        config.dot_choice_3_weight,
    ];

    selected.push('<');

    let is_lamp_mode = send_lamp();

    if is_lamp_mode {
        // Lamp mode: all instruments off
        for _instrument in 1..=20 {
            selected.push('0');
        }
    } else {
        // Normal mode: percussion instruments
        for _percussion in 1..=12 {
            if rng.random_bool(config.dot_percussion_probability) {
                let dist = WeightedIndex::new(weights).unwrap();
                let choice = choices[dist.sample(&mut rng)];
                selected.push(choice);
            } else {
                selected.push('0')
            }
        }

        // String instruments (positions 13-20)
        for _string in 1..=8 {
            selected.push('0')
        }

        // Ensure at least one instrument is active
        if selected[1..=20].iter().all(|&c| c == '0') {
            let idx = rng.random_range(1..=12);
            let dist = WeightedIndex::new(weights).unwrap();
            let choice = choices[dist.sample(&mut rng)];
            selected[idx] = choice;
        }
    }

    // Lamps - use different probability based on mode
    let lamp_prob = if is_lamp_mode {
        config.lamp_probability_when_lamp_mode
    } else {
        config.lamp_probability_normal
    };

    for _lamp in 1..=6 {
        if rng.random_bool(lamp_prob) {
            selected.push('1');
        } else {
            selected.push('0');
        }
    }
    if selected[21..=26].to_vec().iter().all(|&c| c == '0') {
        let idx = rng.random_range(21..25);
        selected[idx] = '1';
    }

    selected.push('>');
    selected.push('\n');
    selected.iter().collect()
}

pub fn convert_dash_message(config: &TransformerConfig) -> String {
    let mut rng = rand::rng();
    let mut selected = Vec::new();
    let choices = ['1', '2', '3', '4'];
    let weights = [
        config.dash_choice_1_weight,
        config.dash_choice_2_weight,
        config.dash_choice_3_weight,
        config.dash_choice_4_weight,
    ];

    selected.push('<');

    let is_lamp_mode = send_lamp();

    if is_lamp_mode {
        // Lamp mode: all instruments off
        for _instrument in 1..=20 {
            selected.push('0');
        }
    } else {
        // Normal mode: rest positions
        for _rest in 1..=12 {
            selected.push('0')
        }

        // String instruments
        for _percussion in 1..=8 {
            if rng.random_bool(config.dash_string_probability) {
                let dist = WeightedIndex::new(weights).unwrap();
                let choice = choices[dist.sample(&mut rng)];
                selected.push(choice);
            } else {
                selected.push('0')
            }
        }

        // Ensure at least one instrument is active
        if selected[1..=20].iter().all(|&c| c == '0') {
            let idx = rng.random_range(13..=20);
            let dist = WeightedIndex::new(weights).unwrap();
            let choice = choices[dist.sample(&mut rng)];
            selected[idx] = choice;
        }
    }

    // Lamps - use different probability based on mode
    let lamp_prob = if is_lamp_mode {
        config.lamp_probability_when_lamp_mode
    } else {
        config.lamp_probability_normal
    };

    for _lamp in 1..=6 {
        if rng.random_bool(lamp_prob) {
            selected.push('2');
        } else {
            selected.push('0');
        }
    }
    if selected[21..=26].iter().all(|&c| c == '0') {
        let idx = rng.random_range(21..25);
        selected[idx] = '2';
    }

    selected.push('>');
    selected.push('\n');
    selected.iter().collect()
}

pub fn convert_space_message(_config: &TransformerConfig) -> String {
    let mut selected = Vec::new();
    selected.push('<');
    for _any in 1..=26 {
        selected.push('0');
    }
    selected.push('>');
    selected.push('\n');
    selected.iter().collect()
}
