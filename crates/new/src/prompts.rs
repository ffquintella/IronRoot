//! Minimal interactive prompt helpers built on `std::io`.

use std::io::{self, BufRead, Write};

fn read_line() -> Result<String, String> {
    let stdin = io::stdin();
    let mut line = String::new();
    stdin
        .lock()
        .read_line(&mut line)
        .map_err(|e| format!("failed to read input: {e}"))?;
    Ok(line.trim().to_string())
}

/// Ask for a free-form string, with an optional default.
pub fn ask_string(label: &str, default: Option<&str>) -> Result<String, String> {
    loop {
        match default {
            Some(d) => print!("{label} [{d}]: "),
            None => print!("{label}: "),
        }
        io::stdout().flush().ok();
        let input = read_line()?;
        if input.is_empty() {
            if let Some(d) = default {
                return Ok(d.to_string());
            }
            println!("  (a value is required)");
            continue;
        }
        return Ok(input);
    }
}

/// Ask for a yes/no answer with a default.
pub fn ask_yes_no(label: &str, default_yes: bool) -> Result<bool, String> {
    let hint = if default_yes { "Y/n" } else { "y/N" };
    loop {
        print!("{label} [{hint}]: ");
        io::stdout().flush().ok();
        let input = read_line()?.to_lowercase();
        if input.is_empty() {
            return Ok(default_yes);
        }
        match input.as_str() {
            "y" | "yes" => return Ok(true),
            "n" | "no" => return Ok(false),
            _ => println!("  please answer y or n"),
        }
    }
}

/// Ask the user to pick one of the given `(key, description)` choices and
/// return the selected key. `default_idx` is selected on empty input.
pub fn ask_choice(
    label: &str,
    choices: &[(&str, &str)],
    default_idx: usize,
) -> Result<String, String> {
    assert!(!choices.is_empty());
    assert!(default_idx < choices.len());

    println!("{label}");
    for (i, (key, desc)) in choices.iter().enumerate() {
        let marker = if i == default_idx { "*" } else { " " };
        println!("  {marker} {}) {key:<14} — {desc}", i + 1);
    }

    loop {
        print!("Choose [1-{}, default {}]: ", choices.len(), default_idx + 1);
        io::stdout().flush().ok();
        let input = read_line()?;
        if input.is_empty() {
            return Ok(choices[default_idx].0.to_string());
        }
        // Accept either the number or the key directly.
        if let Ok(n) = input.parse::<usize>() {
            if n >= 1 && n <= choices.len() {
                return Ok(choices[n - 1].0.to_string());
            }
        }
        if let Some((key, _)) = choices.iter().find(|(k, _)| *k == input) {
            return Ok(key.to_string());
        }
        println!("  invalid choice, try again");
    }
}
