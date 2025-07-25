mod message_transformer;
mod morse_converter;
mod serial_send;
use clokwerk::Interval::*;
use clokwerk::{Scheduler, TimeUnits};
use message_transformer::{convert_dash_message, convert_dot_message, convert_space_message};
use rand::prelude::*;
use std::fs;

use morse_converter::MorseConverter;
// use serial_send::SerialSender;
use std::io::{self, Write};

fn get_last_line() -> Option<String> {
    let contents = fs::read_to_string("test_file.txt").expect("This should be a string");
    contents.lines().last().map(|p| p.to_string())
}
fn get_random_line() -> Option<String> {
    let mut rng = rand::rng();
    let contents = fs::read_to_string("test_file.txt").expect("This should be a string");
    let lines = contents.lines();
    lines.choose(&mut rng).map(|p| p.to_string())
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Morse Code Serial Transmitter");
    println!("Enter text to convert to Morse code and send via serial:");

    let morse_converter = MorseConverter {};

    loop {
        print!("> ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        let input = input.trim();
        if input.is_empty() || input == "quit" || input == "exit" {
            break;
        }

        let morse_code = morse_converter.morse_converter(input);
        let list_morse_code = morse_code.chars();
        for char in list_morse_code {
            if char == '.' {
                let dot_message = convert_dot_message();
                println!("{dot_message}\n")
            }
            if char == '-' {
                let dash_message = convert_dash_message();
                println!("{dash_message}\n")
            }

            if char == ' ' {
                let space_message = convert_space_message();
                println!("{space_message}\n")
            }
        }

        // match serial_sender.send_raw(morse_code.as_bytes()) {
        //     Ok(_) => println!("Successfully sent via serial!"),
        //     Err(e) => eprintln!("Failed to send via serial: {e}"),
        // }
        println!("Morse code: {morse_code}");
    }

    println!("Goodbye!");
    Ok(())
}
