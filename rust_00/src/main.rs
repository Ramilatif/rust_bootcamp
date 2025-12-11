use clap::Parser;

#[derive(Parser)]
#[command(name = "hello")]
#[command(about = "Un programme de salutation simple", long_about = None)]
struct Args {
    ///Nom a saluer
    #[arg(default_value = "World")]
    name: String,

    ///Convertit le text en MAJ
    #[arg(long)]
    upper: bool,

    ///Repete la salutation N fois
    #[arg(long, default_value_t = 1)]
    repeat: u32,
}

fn main() {
    let args = Args::parse();

    // Construire le message
    let mut message = format!("Hello, {}!", args.name);

    if args.upper {
        message = message.to_uppercase();
    }

    // Répéter le message
    for _ in 0..args.repeat {
        println!("{}", message);
    }
}
