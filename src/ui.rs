//! UI utilities for consistent CLI output.

use colored::Colorize;

/// Print a success message with green color.
pub fn success(message: &str) {
    println!("{}", message.green().bold());
}

/// Print a hint/suggestion with dimmed style.
pub fn hint(message: &str) {
    println!("{}", message.dimmed());
}

/// Print next step suggestions.
pub fn next_steps(steps: &[(&str, &str)]) {
    println!();
    println!("{}", "Next steps:".cyan().bold());
    for (cmd, desc) in steps {
        println!("  {}  {}", cmd.cyan(), format!("# {}", desc).dimmed());
    }
}
