use clap::Parser;
use num_format::{Locale, ToFormattedString};
use regex::Regex;
use std::collections::HashMap;
use std::io::{self, Read};

/// Count word frequency in text
#[derive(Parser)]
#[command(
    name = "wordfreq",
    about = "Count word frequency in text",
    author,
    version
)]
struct Args {
    /// Text to analyze (or use stdin)
    #[arg(value_name = "TEXT")]
    text: Option<String>,

    /// Show top N words
    #[arg(long)]
    top: Option<usize>,

    /// Case insensitive counting
    #[arg(long)]
    ignore_case: bool,

    /// Minimum word length to count
    #[arg(long)]
    min_length: Option<usize>,
}

fn read_stdin() -> io::Result<String> {
    let mut buf = String::new();
    io::stdin().read_to_string(&mut buf)?;
    Ok(buf)
}

fn main() {
    let args = Args::parse();

    let mut input = match args.text {
        Some(t) => t,
        None => read_stdin().expect("Failed to read from stdin"),
    };

    if args.ignore_case {
        input = input.to_lowercase();
    }

    // Gestion des mots avec guillemets + mots normaux
    // - "World"  → token = "World"
    // - 'Hello'  → token = 'Hello'
    // - hello    → token = hello
    let re = Regex::new(r#""[^"]+"|'[^']+'|\w+"#).expect("invalid regex");

    let mut freq: HashMap<String, usize> = HashMap::new();
    for m in re.find_iter(&input) {
        let w = m.as_str().to_string();

        // Filtre de longueur minimale
        if let Some(min) = args.min_length {
            if w.chars().count() < min {
                continue;
            }
        }

        *freq.entry(w).or_insert(0) += 1;
    }

    let mut items: Vec<(String, usize)> = freq.into_iter().collect();
    // tri: d'abord par fréquence décroissante, puis par ordre alphabétique
    items.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));

    if let Some(n) = args.top {
        println!("Top {} words:\n", n);
        for (w, c) in items.into_iter().take(n) {
            println!("{}: {}", w, c.to_formatted_string(&Locale::en));
        }
    } else {
        println!("Word frequency:\n");
        for (w, c) in items {
            println!("{}: {}", w, c.to_formatted_string(&Locale::en));
        }
    }
}
