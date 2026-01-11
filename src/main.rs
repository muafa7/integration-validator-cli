use std::path::PathBuf;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Path to the input file (e.g., payload.json)
    #[arg(short, long, value_name = "FILE")]
    input: PathBuf,

    /// Input format (default: json)
    #[arg(short = 'f', long, value_name = "FORMAT", default_value = "json")]
    format: String,
}

fn main() {
    let cli = Cli::parse();

    let raw = fs::read_to_string(&cli.input)
        .unwrap_or_else(|e| panic!("Failed to read input file '{}': {e}", cli.input.display()));

    let payload: serde_json::Value = match serde_json::from_str(&raw) {
        Ok(v) => v,
        Err(e) => {
            eprintln!(
                "error: invalid JSON in '{}': {e}",
                cli.input.display()
            );
            process::exit(1);
        }
    };
    
    println!(
        "{}",
        serde_json::to_string_pretty(&payload).unwrap_or_else(|_| payload.to_string())
    );
}
