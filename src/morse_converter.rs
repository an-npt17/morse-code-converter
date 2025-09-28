use japanese::{charset, converter};
use ripmors::encode_string;

pub struct MorseConverter {}

impl MorseConverter {
    pub fn morse_converter(&self, text: &str) -> String {
        converted_text = text.replace("\n", "    "); // line breaks become 4 spaces, which means we will sleep for 4 `beats` between each line break
        let mut katakana_text = String::with_capacity(converted_text.len());

        if !converted_text.chars().all(charset::is_kana) {
            for c in converted_text.chars() {
                if charset::is_kanji(c) {
                    katakana_text.push(converter::convert_hiragana_to_katakana(c));
                } else {
                    katakana_text.push(c);
                }
            }
        } else {
            for c in converted_text.chars() {
                katakana_text.push(c);
            }
        }

        println!("Original text: {text}, Converted text: {katakana_text}");
        encode_string(&katakana_text.to_string()) // Encode all of them
    }
}
