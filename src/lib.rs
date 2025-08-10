mod cli;
mod db;
mod error;

pub use cli::{Cli, Commands};
use error::AppError;

// The main logic function, which takes the parsed CLI commands
pub fn run(cli: Cli) -> Result<(), AppError> {
    // Open the database
    let db = db::open()?;

    match cli.command {
        Commands::New { key } => {
            // Call the appropriate function from the `db` module
            db::create_note(&db, &key)?;
            println!("Successfully created note: '{}'", key);
        }
        Commands::Edit { key } => {
            // TODO: Call db::edit_note and print a success message
        }
        Commands::Get { key } => {
            // TODO: Call db::get_note and print the result
        }
        Commands::List => {
            // TODO: Call a db::list_notes function and print the keys
        }
        Commands::Delete { key } => {
            // TODO: Call db::delete_note and print a success message
        }
        // The import/export commands can be implemented later
        Commands::Import { .. } => {
            println!("'import' is not implemented yet.");
        }
        Commands::Export { .. } => {
            println!("'export' is not implemented yet.");
        }
    }
    Ok(())
}