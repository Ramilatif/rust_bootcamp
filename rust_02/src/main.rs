use clap::{ArgGroup, Parser};
use std::fs::OpenOptions;
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::path::PathBuf;

/// Read and write binary files in hexadecimal
#[derive(Parser, Debug)]
#[command(name = "hextool")]
#[command(about = "Read and write binary files in hexadecimal")]
#[command(group(
    ArgGroup::new("mode")
        .required(true)
        .args(&["read", "write"]),
))]
struct Args {
    /// Target file
    #[arg(short, long)]
    file: PathBuf,

    /// Read mode (display hex)
    #[arg(short, long)]
    read: bool,

    /// Write mode (hex string to write)
    #[arg(short, long)]
    write: Option<String>,

    /// Offset in bytes (decimal or 0x hex)
    #[arg(short, long, default_value = "0")]
    offset: String,

    /// Number of bytes to read
    #[arg(short, long, default_value = "16")]
    size: usize,
}

fn main() -> io::Result<()> {
    let args = Args::parse();

    let offset = match parse_offset(&args.offset) {
        Ok(o) => o,
        Err(e) => {
            eprintln!("Invalid offset '{}': {}", args.offset, e);
            std::process::exit(1);
        }
    };

    if args.read {
        read_mode(&args.file, offset, args.size)?;
    } else if let Some(hex_str) = args.write {
        match hex_to_bytes(&hex_str) {
            Ok(bytes) => write_mode(&args.file, offset, &bytes)?,
            Err(e) => {
                eprintln!("Invalid hex string '{}': {}", hex_str, e);
                std::process::exit(1);
            }
        }
    }

    Ok(())
}

/// Parse offset given as decimal or 0x... hex
fn parse_offset(s: &str) -> Result<u64, String> {
    if let Some(rest) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
        u64::from_str_radix(rest, 16).map_err(|e| e.to_string())
    } else {
        s.parse::<u64>().map_err(|e| e.to_string())
    }
}

/// Convert hex string "48656c6c6f" -> Vec<u8>
fn hex_to_bytes(s: &str) -> Result<Vec<u8>, String> {
    // Clippy: ne plus faire `len() % 2 != 0`, utiliser is_multiple_of
    if !s.len().is_multiple_of(2) {
        return Err("length must be even".into());
    }

    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).map_err(|e| e.to_string()))
        .collect()
}

/// Convert bytes to ASCII string, printable 0x20..0x7E else '.'
fn bytes_to_ascii(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|b| {
            if (0x20..=0x7e).contains(b) {
                *b as char
            } else {
                '.'
            }
        })
        .collect()
}

/// READ MODE: hex dump
fn read_mode(path: &PathBuf, offset: u64, size: usize) -> io::Result<()> {
    let mut file = OpenOptions::new().read(true).open(path)?;

    file.seek(SeekFrom::Start(offset))?;

    let mut buffer = vec![0u8; size];
    let n = file.read(&mut buffer)?;
    buffer.truncate(n);

    let mut current_offset = offset;

    for chunk in buffer.chunks(16) {
        // offset
        print!("{:08x}: ", current_offset);

        // hex bytes (16 max)
        for i in 0..16 {
            if i < chunk.len() {
                print!("{:02x} ", chunk[i]);
            } else {
                // padding for incomplete last line
                print!("   ");
            }
        }

        // ASCII representation
        let ascii = bytes_to_ascii(chunk);
        println!("|{}|", ascii);

        current_offset += chunk.len() as u64;
    }

    Ok(())
}

/// WRITE MODE: write bytes at given offset and display info
fn write_mode(path: &PathBuf, offset: u64, data: &[u8]) -> io::Result<()> {
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        // Clippy: préciser le comportement de truncate
        .truncate(false)
        .open(path)?;

    file.seek(SeekFrom::Start(offset))?;
    file.write_all(data)?;
    file.flush()?;

    println!("Writing {} bytes at offset 0x{:08x}", data.len(), offset);

    print!("Hex: ");
    for b in data {
        print!("{:02x} ", b);
    }
    println!();

    let ascii = bytes_to_ascii(data);
    println!("ASCII: {}", ascii);
    println!("✓ Successfully written");

    Ok(())
}
