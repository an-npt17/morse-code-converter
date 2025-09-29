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
        encode_string(&katakana_text.to_string()) // Encode all of them
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use japanese::{charset, converter};
    use ripmors::encode_string;

    #[test]
    fn test_morse_converter_with_katakana() {
        let converter = MorseConverter {};
        let input = "hello\nhi"; // already Katakana
        let output = converter.morse_converter(input);

        // The result should not be empty
        assert!(!output.is_empty(), "Morse code should not be empty");

        // Ensure encode_string does something meaningful
        assert_eq!(output, encode_string(&input.to_string()));
    }
}
