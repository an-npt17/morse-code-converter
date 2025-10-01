use crate::send_lamp;
use rand::distr::weighted::WeightedIndex;
use rand::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformerConfig {
    pub dot_percussion_probability: f64,
    pub dot_choice_1_weight: u32,
    pub dot_choice_2_weight: u32,
    pub dot_choice_3_weight: u32,

    pub dash_string_probability: f64,
    pub dash_choice_1_weight: u32,
    pub dash_choice_2_weight: u32,
    pub dash_choice_3_weight: u32,
    pub dash_choice_4_weight: u32,

    pub lamp_activation_probability: f64,
    pub lamp_dot_value: char,
    pub lamp_dash_value: char,
}

impl Default for TransformerConfig {
    fn default() -> Self {
        TransformerConfig {
            dot_percussion_probability: 0.12,
            dot_choice_1_weight: 70,
            dot_choice_2_weight: 20,
            dot_choice_3_weight: 10,

            dash_string_probability: 0.2,
            dash_choice_1_weight: 20,
            dash_choice_2_weight: 20,
            dash_choice_3_weight: 30,
            dash_choice_4_weight: 30,

            lamp_activation_probability: 0.3,
            lamp_dot_value: '1',
            lamp_dash_value: '2',
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

    if send_lamp() {
        for _instrument in 1..=20 {
            selected.push('0');
        }
        for _lamp in 1..=6 {
            if rng.random_bool(config.lamp_activation_probability) {
                selected.push(config.lamp_dot_value);
            } else {
                selected.push('0');
            }
        }
    } else {
        // Percussion instruments (positions 1-12)
        for _percussion in 1..=12 {
            if rng.random_bool(config.dot_percussion_probability) {
                let dist = WeightedIndex::new(weights).unwrap();
                let choice = choices[dist.sample(&mut rng)];
                selected.push(choice);
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

        if selected[1..=20].iter().all(|&c| c == '0') {
            let idx = rng.random_range(1..=12);
            let dist = WeightedIndex::new(weights).unwrap();
            let choice = choices[dist.sample(&mut rng)];
            selected[idx] = choice;
        }
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

    if send_lamp() {
        for _instrument in 1..=20 {
            selected.push('0');
        }
        for _lamp in 1..=6 {
            if rng.random_bool(config.lamp_activation_probability) {
                selected.push(config.lamp_dash_value);
            } else {
                selected.push('0');
            }
        }
    } else {
        for _rest in 1..=12 {
            selected.push('0')
        }

        for _percussion in 1..=8 {
            if rng.random_bool(config.dash_string_probability) {
                let dist = WeightedIndex::new(weights).unwrap();
                let choice = choices[dist.sample(&mut rng)];
                selected.push(choice);
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
        }
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
