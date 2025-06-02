use clap::{Arg, Command, ArgAction};
use colored::*;
use mid_valyrian::{run_file, ValyrianError};

fn main() {
    print_banner();

    let matches = Command::new("mid_valyrian")
        .version("0.1.0")
        .author("The Maesters of Oldtown and Oishik Biswas")
        .about("A Game of Thrones inspired interpreted programming language")
        .arg(
            Arg::new("file")
                .help("The .mv file to execute")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::new("debug")
                .short('d')
                .long("debug")
                .help("Enable debug mode (show AST and execution trace)")
                .action(ArgAction::SetTrue),
        )
        .get_matches();

    let file_path = matches.get_one::<String>("file").expect("required");
    let debug = matches.get_flag("debug");

    // Enforce .mv extension
    if !file_path.ends_with(".mv") {
        eprintln!("{}", "Error: Only files with the `.mv` extension are allowed.".bright_red());
        std::process::exit(1);
    }

    if debug {
        println!("{}", "🐉 Debug mode enabled - The Maesters will show their work".bright_yellow());
    }

    match run_file(file_path, debug) {
        Ok(()) => {
            if debug {
                println!("{}", "✅ The realm prospers! Program executed successfully.".bright_green());
            }
        }
        Err(error) => {
            eprintln!("{}", format!("{}", error).bright_red());
            std::process::exit(1);
        }
    }
}

fn print_banner() {
    println!(
        "{}",
        r#"
    ╔════════════════════════════════════════════════════════════╗
    ║                                                            ║
    ║    🐉 Welcome to Mid Valyrian - Language of Old Valyria   ║
    ║                                                            ║
    ║    "Valar morghulis" — All men must debug                  ║
    ║                                                            ║
    ╚════════════════════════════════════════════════════════════╝
    "#
        .bright_cyan()
    );
}
