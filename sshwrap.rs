use regex::Regex;
use serde::Deserialize;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::{exit, Command};

#[derive(Debug, Deserialize)]
struct Config {
    patterns: Vec<Pattern>,
}

#[derive(Debug, Deserialize)]
struct Pattern {
    pattern: String,
    add: String,
}

fn load_config() -> Result<Config, Box<dyn std::error::Error>> {
    let mut path = PathBuf::from(env::var("HOME").unwrap());
    path.push(".ssh/wrapper.toml");

    let contents = fs::read_to_string(path)?;
    let config: Config = toml::from_str(&contents)?;
    Ok(config)
}

fn match_pattern(pattern: &str, host: &str, debug: bool) -> Option<Vec<String>> {
    // Use the pattern directly as a regex
    let regex_pattern = format!("^{}$", pattern);
    debug_print(
        debug,
        &format!(
            "Generated regex for pattern '{}': {}",
            pattern, regex_pattern
        ),
    );

    let re = Regex::new(&regex_pattern).expect("Invalid regex pattern");

    // Match the host against the regex
    if let Some(captures) = re.captures(host) {
        // Collect all capture groups
        let groups: Vec<String> = captures
            .iter()
            .skip(1) // Skip the full match
            .filter_map(|m| m.map(|v| v.as_str().to_string()))
            .collect();

        debug_print(
            debug,
            &format!("Regex matched. Captured groups: {:?}", groups),
        );
        return Some(groups);
    } else {
        debug_print(debug, &format!("Regex did not match for host: {}", host));
    }
    None
}

fn debug_print(debug: bool, message: &str) {
    if debug {
        println!("[DEBUG] {}", message);
    }
}

fn main() {
    // Collect arguments
    let args = env::args().collect::<Vec<String>>();
    let debug = args.contains(&"--debug".to_string());

    if args.len() < 2 {
        eprintln!("Usage: sshwrap [--debug] <host> [ssh-args...]");
        exit(1);
    }

    // Extract host and optional SSH args
    let original_host = if debug { &args[2] } else { &args[1] };
    let ssh_args = if debug { &args[3..] } else { &args[2..] };

    debug_print(debug, &format!("Original host: {}", original_host));

    let config = match load_config() {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Failed to load config: {}", e);
            Config {
                patterns: Vec::new(),
            } // Use an empty config
        }
    };

    let mut transformed_host = None;

    for pattern in &config.patterns {
        debug_print(debug, &format!("Trying pattern: {}", pattern.pattern));
        if let Some(groups) = match_pattern(&pattern.pattern, original_host, debug) {
            debug_print(debug, &format!("Matched pattern: {}", pattern.pattern));
            debug_print(debug, &format!("Captured groups: {:?}", groups));

            // Replace numbered groups in the `add` field
            let mut transformed = pattern.add.clone();
            for (i, group) in groups.iter().enumerate() {
                let placeholder = format!("{{{}}}", i + 1);
                transformed = transformed.replace(&placeholder, group);
            }

            debug_print(debug, &format!("Transformed host: {}", transformed));
            transformed_host = Some(transformed);
            break;
        }
    }

    let transformed_host = match transformed_host {
        Some(host) => {
            // debug_print(debug, &format!("Transformed host: {}", host));
            host
        }
        None => {
            debug_print(
                debug,
                "No matching pattern found. Passing original host to ssh.",
            );
            // If no pattern matches, use the original host and args
            let status = Command::new("ssh")
                .arg(original_host)
                .args(ssh_args)
                .status()
                .expect("Failed to launch ssh");

            if let Some(code) = status.code() {
                exit(code);
            } else {
                exit(1);
            }
            return; // Early return to prevent further execution
        }
    };

    // Construct SSH command
    debug_print(
        debug,
        &format!("Executing ssh with host: {}", transformed_host),
    );
    let status = Command::new("ssh")
        .arg(transformed_host)
        .args(ssh_args)
        .status()
        .expect("Failed to launch ssh");

    if let Some(code) = status.code() {
        exit(code);
    } else {
        exit(1);
    }
}
