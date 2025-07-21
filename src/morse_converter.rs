use std::collections::HashMap;

#[derive(Debug)]
pub enum MorseError {
    UnsupportedCharacter(char),
}

impl std::fmt::Display for MorseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MorseError::UnsupportedCharacter(c) => write!(f, "Unsupported character: {c}"),
        }
    }
}

impl std::error::Error for MorseError {}

pub struct MorseConverter {
    morse_map: HashMap<char, &'static str>,
}

impl MorseConverter {
    pub fn new() -> Self {
        let mut morse_map = HashMap::new();

        // Letters
        morse_map.insert('A', ".-");
        morse_map.insert('B', "-...");
        morse_map.insert('C', "-.-.");
        morse_map.insert('D', "-..");
        morse_map.insert('E', ".");
        morse_map.insert('F', "..-.");
        morse_map.insert('G', "--.");
        morse_map.insert('H', "....");
        morse_map.insert('I', "..");
        morse_map.insert('J', ".---");
        morse_map.insert('K', "-.-");
        morse_map.insert('L', ".-..");
        morse_map.insert('M', "--");
        morse_map.insert('N', "-.");
        morse_map.insert('O', "---");
        morse_map.insert('P', ".--.");
        morse_map.insert('Q', "--.-");
        morse_map.insert('R', ".-.");
        morse_map.insert('S', "...");
        morse_map.insert('T', "-");
        morse_map.insert('U', "..-");
        morse_map.insert('V', "...-");
        morse_map.insert('W', ".--");
        morse_map.insert('X', "-..-");
        morse_map.insert('Y', "-.--");
        morse_map.insert('Z', "--..");

        // Numbers
        morse_map.insert('0', "-----");
        morse_map.insert('1', ".----");
        morse_map.insert('2', "..---");
        morse_map.insert('3', "...--");
        morse_map.insert('4', "....-");
        morse_map.insert('5', ".....");
        morse_map.insert('6', "-....");
        morse_map.insert('7', "--...");
        morse_map.insert('8', "---..");
        morse_map.insert('9', "----.");

        // Punctuation
        morse_map.insert('.', ".-.-.-");
        morse_map.insert(',', "--..--");
        morse_map.insert('?', "..--..");
        morse_map.insert('\'', ".----.");
        morse_map.insert('!', "-.-.--");
        morse_map.insert('/', "-..-.");
        morse_map.insert('(', "-.--.");
        morse_map.insert(')', "-.--.-");
        morse_map.insert('&', ".-...");
        morse_map.insert(':', "---...");
        morse_map.insert(';', "-.-.-.");
        morse_map.insert('=', "-...-");
        morse_map.insert('+', ".-.-.");
        morse_map.insert('-', "-....-");
        morse_map.insert('_', "..--.-");
        morse_map.insert('"', ".-..-.");
        morse_map.insert('$', "...-..-");
        morse_map.insert('@', ".--.-.");

        Self { morse_map }
    }

    pub fn text_to_morse(&self, text: &str) -> Result<String, MorseError> {
        let mut morse_result = Vec::new();

        for ch in text.chars() {
            if ch.is_whitespace() {
                morse_result.push("/".to_string()); // Word separator
            } else {
                let upper_ch = ch.to_uppercase().next().unwrap_or(ch);

                if let Some(morse_code) = self.morse_map.get(&upper_ch) {
                    morse_result.push(morse_code.to_string());
                } else {
                    return Err(MorseError::UnsupportedCharacter(ch));
                }
            }
        }

        Ok(morse_result.join(" "))
    }

    pub fn char_to_morse(&self, ch: char) -> Result<&str, MorseError> {
        let upper_ch = ch.to_uppercase().next().unwrap_or(ch);
        self.morse_map
            .get(&upper_ch)
            .copied()
            .ok_or(MorseError::UnsupportedCharacter(ch))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_conversion() {
        let converter = MorseConverter::new();
        assert_eq!(converter.text_to_morse("SOS").unwrap(), "... --- ...");
        assert_eq!(
            converter.text_to_morse("HELLO").unwrap(),
            ".... . .-.. .-.. ---"
        );
    }

    #[test]
    fn test_with_spaces() {
        let converter = MorseConverter::new();
        assert_eq!(
            converter.text_to_morse("HI THERE").unwrap(),
            ".... .. / - .... . .-. ."
        );
    }

    #[test]
    fn test_numbers() {
        let converter = MorseConverter::new();
        assert_eq!(converter.text_to_morse("123").unwrap(), ".---- ..--- ...--");
    }

    #[test]
    fn test_case_insensitive() {
        let converter = MorseConverter::new();
        assert_eq!(
            converter.text_to_morse("hello").unwrap(),
            converter.text_to_morse("HELLO").unwrap()
        );
    }
}
