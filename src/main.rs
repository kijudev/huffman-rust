use std::fs;
use std::path::PathBuf;
use std::process;

mod huffman;

use clap::{Parser, Subcommand};
use huffman::{Huffman, Message};
use rmp_serde::{from_slice, to_vec};

#[derive(Parser)]
#[command(author, version, about)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Compress {
        #[arg(value_name = "INPUT")]
        input: PathBuf,

        #[arg(short, long, value_name = "OUTPUT")]
        output: Option<PathBuf>,
    },

    Decompress {
        #[arg(value_name = "INPUT")]
        input: PathBuf,

        #[arg(short, long, value_name = "OUTPUT")]
        output: Option<PathBuf>,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Compress { input, output } => {
            if let Err(e) = compress_cmd(&input, output.as_ref()) {
                eprintln!("Compression failed: {}", e);
                process::exit(1);
            }
        }
        Commands::Decompress { input, output } => {
            if let Err(e) = decompress_cmd(&input, output.as_ref()) {
                eprintln!("Decompression failed: {}", e);
                process::exit(1);
            }
        }
    }
}

fn compress_cmd(input: &PathBuf, output: Option<&PathBuf>) -> Result<(), String> {
    let data = fs::read(input).map_err(|e| format!("Failed to read input file: {}", e))?;

    let message = Huffman::encode(&data)?;

    let serialized =
        to_vec(&message).map_err(|e| format!("Failed to serialize compressed message: {}", e))?;

    let out_path = match output {
        Some(p) => p.clone(),
        None => {
            let mut p = input.clone();
            p.set_extension(format!(
                "{}.huf",
                input
                    .extension()
                    .and_then(|s| s.to_str())
                    .unwrap_or_default()
            ));
            p
        }
    };

    fs::write(&out_path, &serialized).map_err(|e| {
        format!(
            "Failed to write compressed file '{}': {}",
            out_path.display(),
            e
        )
    })?;

    println!(
        "Compressed '{}' ({} bytes) -> '{}' ({} bytes)",
        input.display(),
        data.len(),
        out_path.display(),
        serialized.len()
    );

    Ok(())
}

fn decompress_cmd(input: &PathBuf, output: Option<&PathBuf>) -> Result<(), String> {
    let serialized =
        fs::read(input).map_err(|e| format!("Failed to read compressed file: {}", e))?;

    let message: Message = from_slice(&serialized)
        .map_err(|e| format!("Failed to deserialize compressed message: {}", e))?;

    let decoded = Huffman::decode(&message)?;

    let out_path = match output {
        Some(p) => p.clone(),
        None => {
            let mut p = input.clone();
            p.set_extension(format!(
                "{}.orig",
                input
                    .extension()
                    .and_then(|s| s.to_str())
                    .unwrap_or_default()
            ));
            p
        }
    };

    fs::write(&out_path, &decoded).map_err(|e| {
        format!(
            "Failed to write decompressed file '{}': {}",
            out_path.display(),
            e
        )
    })?;

    println!(
        "Decompressed '{}' -> '{}' ({} bytes)",
        input.display(),
        out_path.display(),
        decoded.len()
    );

    Ok(())
}
