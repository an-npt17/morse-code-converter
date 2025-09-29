use japanese::{charset, converter};
use ripmors::encode_string;

pub struct MorseConverter {}

impl MorseConverter {
    pub fn morse_converter(&self, text: &str) -> String {
        let mut katakana_text = String::with_capacity(text.len());

        if !text.chars().all(charset::is_kana) {
            for c in text.chars() {
                if charset::is_kanji(c) {
                    katakana_text.push(converter::convert_hiragana_to_katakana(c));
                } else if c.is_whitespace() {
                    katakana_text.push(' ');
                } else {
                    katakana_text.push(c);
                }
            }
        } else {
            for c in text.chars() {
                katakana_text.push(c);
            }
        }

        println!("Original text: {text}, Converted text: {katakana_text}");
        let encoded_string = encode_string(&katakana_text.to_string()); // Encode all of them
        encoded_string.replace("/", " ") // encode string denotes / as space
    }
}
