mod huffman;

use huffman::{Huffman, Message};

fn main() {
    let text = "Hello, world! Hello Huffman!";
    let bytes = text.as_bytes();

    let message: Message = match Huffman::encode(bytes) {
        Ok(msg) => msg,
        Err(err) => {
            eprintln!("Failed to encode message: {}", err);
            std::process::exit(1);
        }
    };

    println!("Original text   : {}", text);
    println!("Original bytes  : {}", bytes.len());
    println!("Encoded bit len : {} bits", message.encoded_data.len());

    let decoded_bytes = match Huffman::decode(&message) {
        Ok(b) => b,
        Err(err) => {
            eprintln!("Failed to decode message: {}", err);
            std::process::exit(1);
        }
    };

    let decoded = match String::from_utf8(decoded_bytes) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Decoded bytes are not valid UTF-8: {}", e);
            std::process::exit(1);
        }
    };

    println!("Decoded text    : {}", decoded);
    println!("Round-trip equal: {}", decoded == text);
}
