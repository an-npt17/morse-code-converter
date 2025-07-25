use japanese::{charset, converter};
use kakasi::convert;
use ripmors::encode_string;

pub struct MorseConverter {}

impl MorseConverter {
    pub fn morse_converter(&self, text: &str) -> String {
        let kakasi_res = convert(text);
        let hiragana_text = kakasi_res.hiragana; // Convert the string to hiragana

        let mut katakana_text = String::with_capacity(hiragana_text.len());

        for c in hiragana_text.chars() {
            if charset::is_hiragana(c) {
                katakana_text.push(converter::convert_hiragana_to_katakana(c));
            } else {
                katakana_text.push(c);
            }
        }

        println!("Original text: {text}, Converted text: {katakana_text}");
        encode_string(&katakana_text.to_string()) // Encode all of them
    }
}
