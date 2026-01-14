use std::path::PathBuf;
use std::process;
use std::fs;

use serde::{Deserialize, Serialize};

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

fn clean_input(order: Order) -> Order {
    let event_id = match order.event_id {
        None => None,
        Some(v) => {
            let trimmed = v.trim();
            if trimmed.is_empty(){
                None
            } else {
                Some(trimmed.to_string())
            }
        }
    };

    let part_number = match order.part_number {
        None => None,
        Some(v) => {
            let trimmed = v.trim();
            if trimmed.is_empty(){
                None
            } else {
                Some(trimmed.to_ascii_uppercase())
            }
        }
    };

    let timestamp = match order.timestamp {
        None => None,
        Some(v) => {
            let trimmed = v.trim();
            if trimmed.is_empty(){
                None
            } else {
                Some(trimmed.to_string())
            }
        }
    };


    Order{
        event_id,
        part_number,
        timestamp,
    }
}

fn get_json_files(dir: &PathBuf) -> Vec<PathBuf> {
    let mut files = Vec::new();

    let entries = fs::read_dir(dir).unwrap_or_else(|e| {
        panic!("failed to read dir '{}': {}", dir.display(), e);
    });

    for entry in entries {
        let entry = entry.unwrap();
        let path = entry.path();

        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("json"){
            files.push(path);
        }
    }

    files.sort();
    files
}

fn main() {
    let cli = Cli::parse();

    if !cli.input.exists() {
        eprintln!("input path does not exist: {}", cli.input.display());
        process::exit(1);
    }

    if cli.input.is_dir() {
        let files = get_json_files(&cli.input);

        if files.is_empty() {
            eprintln!("no .json files found in {}", cli.input.display());
            process::exit(1);
        }

        let mut any_failed = false;

        for path in files {
            println!("== {} ==", path.display());

            let raw = fs::read_to_string(&path).unwrap_or_else(|e| {
                eprintln!("failed to read '{}': {e}", path.display());
                process::exit(1);
            });

            let order: Order = serde_json::from_str(&raw).unwrap_or_else(|e| {
                eprintln!("invalid JSON: {e}");
                process::exit(1);
            });

            let normalized_order: Order = clean_input(order);
            let issues = validate(&normalized_order);

            print_report(&issues);

            if issues.iter().any(|i| matches!(i.severity, Severity::Error)) {
                any_failed = true;
            }
        }

        if any_failed {
            process::exit(1);
        }
        return;
    }

    if cli.input.is_file() {
        let raw = fs::read_to_string(&cli.input).unwrap_or_else(|e| {
            eprintln!("failed to read '{}': {e}", cli.input.display());
            process::exit(1);
        });
        let order: Order = serde_json::from_str(&raw).unwrap_or_else(|e| {
            eprintln!("invalid JSON: {e}");
            process::exit(1);
        });

        let normalized_order: Order = clean_input(order);
        
        let issues = validate(&normalized_order);

        print_report(&issues);

        if issues.iter().any(|i| matches!(i.severity, Severity::Error)) {
            process::exit(1);
        }
    } else {
        eprintln!("input path is not a regular file or directory: {}", cli.input.display());
        process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clean_input_trims_and_uppercases_part_number() {
        let test_order = Order {
            event_id: Some("  abc  ".to_string()),
            part_number: Some("  zx-9  ".to_string()),
            timestamp: Some(" 2020-01-01 ".to_string()),
        };

        let cleaned = clean_input(test_order);

        assert_eq!(cleaned.event_id, Some("abc".to_string()));
        assert_eq!(cleaned.part_number, Some("ZX-9".to_string()));
        assert_eq!(cleaned.timestamp, Some("2020-01-01".to_string()));
    }

    #[test]
    fn clean_input_convert_blank_whitespace_to_none() {
        let test_order = Order {
            event_id: Some(" ".to_string()),
            part_number: Some("  ".to_string()),
            timestamp: Some(" ".to_string()),
        };

        let cleaned = clean_input(test_order);

        assert_eq!(cleaned.event_id, None);
        assert_eq!(cleaned.part_number, None);
        assert_eq!(cleaned.timestamp, None);
    }

    #[test]
    fn validate_return_zero_order() {
        let test_order = Order {
            event_id: Some("abc".to_string()),
            part_number: Some("zx-9".to_string()),
            timestamp: Some("2020-01-01".to_string()),
        };

        let issues = validate(&test_order);

        assert_eq!(issues.len(), 0);
    }

    #[test]
    fn validate_missing_required_field() {
        let test_order = Order {
            event_id: None,
            part_number: Some("zx-9".to_string()),
            timestamp: Some("2020-01-01".to_string()),
        };

        let issues = validate(&test_order);

        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].field, "event_id");
        assert_eq!(issues[0].message, "missing required field");
    }

    #[test]
    fn whitespace_only_become_missing_after_cleaning() {
        let test_order = Order {
            event_id: Some(" ".to_string()),
            part_number: Some("zx-9".to_string()),
            timestamp: Some("2020-01-01".to_string()),
        };

        let normalized = clean_input(test_order);
        let issues = validate(&normalized);

        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].field, "event_id");
        assert_eq!(issues[0].message, "missing required field");
    
    }
    #[test]
    fn validate_must_not_be_empty() {
        let test_order = Order {
            event_id: Some(" ".to_string()),
            part_number: Some("zx-9".to_string()),
            timestamp: Some("2020-01-01".to_string()),
        };

        let issues = validate(&test_order);

        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].field, "event_id");
        assert_eq!(issues[0].message, "must not be empty");
    }
}