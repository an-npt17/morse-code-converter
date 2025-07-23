use japanese::{charset, converter};
use kakasi::convert;
use morsify::{MorseCharacterSet, MorseCode, Options};

pub struct MorseConverter {
    morse_code: MorseCode,
}

impl MorseConverter {
    pub fn new() -> Self {
        let options = Options {
            dash: '-',
            dot: '.',
            space: '/',
            separator: ' ',
            invalid_char_callback: |c| c,
            priority: MorseCharacterSet::Japanese,
        };
        let morse_code = MorseCode::new(options);

        Self { morse_code }
    }

    // pub fn morse_converter(&self, text: &str) -> String {
    //     let mut katakana_text = String::with_capacity(text.len());
    //
    //     for c in text.chars() {
    //         if charset::is_hiragana(c) {
    //             katakana_text.push(converter::convert_hiragana_to_katakana(c));
    //         } else {
    //             katakana_text.push(c);
    //         }
    //     }
    //     println!("Original text: {text}, Converted text: {katakana_text}");
    //
    //     self.morse_code.encode(katakana_text)
    // }
}
