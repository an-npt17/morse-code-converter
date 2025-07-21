mod morse_converter;
mod serial_send;

use morse_converter::MorseConverter;
// use serial_send::SerialSender;
use std::io::{self, Write};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Morse Code Serial Transmitter");
    println!("Enter text to convert to Morse code and send via serial:");

    // let mut serial_sender = SerialSender::new("/dev/serial0", 9600)?;
    let morse_converter = MorseConverter::new();

    loop {
        print!("> ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        let input = input.trim();
        if input.is_empty() || input == "quit" || input == "exit" {
            break;
        }

        match morse_converter.text_to_morse(input) {
            Ok(morse_code) => {
                println!("Morse code: {morse_code}");

                // match serial_sender.send_morse(&morse_code) {
                //     Ok(_) => println!("Successfully sent via serial!"),
                //     Err(e) => eprintln!("Failed to send via serial: {e}"),
                // }
            }
            Err(e) => eprintln!("Error converting to Morse: {e}"),
        }

        println!();
    }

    println!("Goodbye!");
    Ok(())
}
