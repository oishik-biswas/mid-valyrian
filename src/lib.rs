//! 🐉 Mid Valyrian Language Core
//!
//! This crate provides parsing, interpreting, and error handling for the
//! Mid Valyrian programming language, inspired by *Game of Thrones*.
//!
//! Use `run_file` to execute a `.valyrian` source file,
//! or `run_code` to interpret Valyrian code from a string.

pub mod ast;
pub mod parser;
pub mod interpreter;
pub mod error;

pub use ast::*;
pub use parser::*;
pub use interpreter::*;
pub use error::*;

use std::fs;
use std::path::Path;

/// Runs a Mid Valyrian source file.
///
/// # Arguments
///
/// * `path` - Path to the `.valyrian` source file.
/// * `debug` - Enables verbose AST and execution output if `true`.
///
/// # Errors
///
/// Returns `ValyrianError` if file reading, parsing, or interpretation fails.
pub fn run_file<P: AsRef<Path>>(path: P, debug: bool) -> Result<(), ValyrianError> {
    let path_ref = path.as_ref();

    if !path_ref.ends_with(".mv") {
        return Err(ValyrianError::ParseError("File must end with .mv".to_string()));
    }
    
    let contents = fs::read_to_string(path_ref)
        .map_err(|e| ValyrianError::IoError(format!(
            "Failed to read file '{}': {}",
            path_ref.display(),
            e
        )))?;

    run_code(&contents, debug)
}

/// Runs Mid Valyrian code from a string.
///
/// # Arguments
///
/// * `code` - The source code as a string.
/// * `debug` - Enables verbose AST and execution output if `true`.
///
/// # Errors
///
/// Returns `ValyrianError` if parsing or interpretation fails.
pub fn run_code(code: &str, debug: bool) -> Result<(), ValyrianError> {
    let program = parse_program(code)?;
    let mut interpreter = Interpreter::new(debug);
    interpreter.interpret(&program)
}
