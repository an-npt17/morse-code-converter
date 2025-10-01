use crate::send_lamp;
use rand::distr::weighted::WeightedIndex;
use rand::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LampConfig {
    pub probability: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformerConfig {
    // Tempo configuration
    pub tempo_choices: Vec<u64>,

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

    // Lamp configuration (6 lamps for dot) - each lamp sends '1'
    pub lamp_dot_configs: [LampConfig; 6],

    // Lamp configuration (6 lamps for dash) - each lamp sends '2'
    pub lamp_dash_configs: [LampConfig; 6],
}

impl Default for TransformerConfig {
    fn default() -> Self {
        TransformerConfig {
            // Tempo defaults
            tempo_choices: vec![400, 700, 1000],

            // Dot defaults
            dot_percussion_probability: 0.12,
            dot_choice_1_weight: 70,
            dot_choice_2_weight: 20,
            dot_choice_3_weight: 10,

            // Dash defaults
            dash_string_probability: 0.2,
            dash_choice_1_weight: 20,
            dash_choice_2_weight: 20,
            dash_choice_3_weight: 30,
            dash_choice_4_weight: 30,

            // Lamp dot defaults (6 lamps) - each lamp sends '1' with probability
            lamp_dot_configs: [
                LampConfig { probability: 0.3 },
                LampConfig { probability: 0.3 },
                LampConfig { probability: 0.3 },
                LampConfig { probability: 0.3 },
                LampConfig { probability: 0.3 },
                LampConfig { probability: 0.3 },
            ],

            // Lamp dash defaults (6 lamps) - each lamp sends '2' with probability
            lamp_dash_configs: [
                LampConfig { probability: 0.3 },
                LampConfig { probability: 0.3 },
                LampConfig { probability: 0.3 },
                LampConfig { probability: 0.3 },
                LampConfig { probability: 0.3 },
                LampConfig { probability: 0.3 },
            ],
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
        // Apply individual lamp configurations - each lamp sends '1' with its own probability
        for lamp_config in &config.lamp_dot_configs {
            if rng.random_bool(lamp_config.probability) {
                selected.push('1');
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

        // String instruments (positions 13-20)
        for _string in 1..=8 {
            selected.push('0')
        }

        // Lamps (positions 21-26) - all off when not in lamp mode
        for _lamp in 1..=6 {
            selected.push('0');
        }

        // Ensure at least one instrument is active
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
        // Apply individual lamp configurations - each lamp sends '2' with its own probability
        for lamp_config in &config.lamp_dash_configs {
            if rng.random_bool(lamp_config.probability) {
                selected.push('2');
            } else {
                selected.push('0');
            }
        }
    } else {
        // Rest (positions 1-12)
        for _rest in 1..=12 {
            selected.push('0')
        }

        // String instruments (positions 13-20)
        for _percussion in 1..=8 {
            if rng.random_bool(config.dash_string_probability) {
                let dist = WeightedIndex::new(weights).unwrap();
                let choice = choices[dist.sample(&mut rng)];
                selected.push(choice);
            } else {
                selected.push('0')
            }
        }

        // Lamps (positions 21-26) - all off when not in lamp mode
        for _lamp in 1..=6 {
            selected.push('0');
        }

        // Ensure at least one instrument is active
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
