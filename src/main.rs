use std::path::PathBuf;
use std::process;
use std::fs;

use serde::{Deserialize, Serialize};
use serde_json::Result;

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

#[derive(Debug, Serialize, Deserialize)]
struct Order {
    event_id: Option<String>,
    part_number: Option<String>,
    timestamp: Option<String>,
}

#[derive(Debug)]
enum Severity {
    Error,
    Warning,
}

#[derive(Debug)]
struct Issue {
    field: &'static str,
    severity: Severity,
    message: String,
}

impl Issue {
    fn error(field: &'static str, msg: &str) -> Self {
        Self {
            field,
            severity: Severity::Error,
            message: msg.to_string(),
        }
    }
}

fn validate(order: &Order) -> Vec<Issue> {
    let mut issues = Vec::new();

    match &order.event_id {
        None => issues.push(Issue::error("event_id", "missing required field")),
        Some(v) if v.trim().is_empty() => {
            issues.push(Issue::error("event_id", "must not be empty"))
        }
        _ => {}
    }

    match &order.part_number {
        None => issues.push(Issue::error("part_number", "missing required field")),
        Some(v) if v.trim().is_empty() => {
            issues.push(Issue::error("part_number", "must not be empty"))
        }
        _ => {}
    }

    match &order.timestamp {
        None => issues.push(Issue::error("timestamp", "missing required field")),
        Some(v) if v.trim().is_empty() => {
            issues.push(Issue::error("timestamp", "must not be empty"))
        }
        _ => {}
    }

    issues
}

fn print_report(issues: &[Issue]) {
    let error_count = issues
        .iter()
        .filter(|i| matches!(i.severity, Severity::Error))
        .count();

    println!("Validation report");
    println!("-----------------");
    println!("errors: {}", error_count);
    println!("total issues: {}", issues.len());
    println!();

    for issue in issues {
        println!(
            "ERROR {}: {}",
            issue.field,
            issue.message
        );
    }
}

fn main() {
    let cli = Cli::parse();

    let raw = fs::read_to_string(&cli.input).unwrap_or_else(|e| {
        eprintln!("failed to read '{}': {e}", cli.input.display());
        process::exit(1);
    });

    let order: Order = serde_json::from_str(&raw).unwrap_or_else(|e| {
        eprintln!("invalid JSON: {e}");
        process::exit(1);
    });
    
    let issues = validate(&order);

    print_report(&issues);

    if issues.iter().any(|i| matches!(i.severity, Severity::Error)) {
        process::exit(1);
    }
}
