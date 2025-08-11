use colored::*;

/// Utility functions for printing messages in different colors
/// for better visibility and user experience in the command line interface.
pub fn success(message: &str) {
    println!("{}", message.green().bold());
}

pub fn info(message: &str) {
    println!("{}", message.cyan());
}

pub fn warn(message: &str) {
    eprintln!("{}", message.yellow());
}

pub fn error(message: &str) {
    eprintln!("{}", message.red().bold());
}