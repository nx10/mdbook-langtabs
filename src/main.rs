use clap::{Arg, Command};
use mdbook::errors::Error;
use mdbook::preprocess::{CmdPreprocessor, Preprocessor};
use mdbook_langtabs::LangTabsPreprocessor;
use std::fs;
use std::io;
use std::path::PathBuf;
use std::process;

fn main() {
    let matches = Command::new("mdbook-langtabs")
        .about("An mdbook preprocessor that adds language tabs for code blocks")
        .subcommand(
            Command::new("supports")
                .arg(Arg::new("renderer").required(true))
                .about("Check whether a renderer is supported by this preprocessor"),
        )
        .subcommand(
            Command::new("install")
                .arg(
                    Arg::new("dir")
                        .help("Directory of the book (where the book.toml file is located)")
                        .required(true),
                )
                .about("Install the preprocessor and required assets"),
        )
        .get_matches();

    let preprocessor = LangTabsPreprocessor::new();

    if let Some(sub_args) = matches.subcommand_matches("supports") {
        let renderer = sub_args
            .get_one::<String>("renderer")
            .expect("Required argument");
        let supported = preprocessor.supports_renderer(renderer);

        if supported {
            process::exit(0);
        } else {
            process::exit(1);
        }
    } else if let Some(sub_args) = matches.subcommand_matches("install") {
        let dir = sub_args
            .get_one::<String>("dir")
            .expect("Required argument");
        let dir_path = PathBuf::from(dir);

        if let Err(e) = install_preprocessor(&dir_path) {
            eprintln!("Error installing preprocessor: {}", e);
            process::exit(1);
        }
    } else {
        // Normal preprocessing mode
        let (ctx, book) = CmdPreprocessor::parse_input(io::stdin()).expect("Failed to parse stdin");

        let processed_book = preprocessor
            .run(&ctx, book)
            .expect("Failed to process book");

        serde_json::to_writer(io::stdout(), &processed_book)
            .expect("Failed to write processed book");
    }
}

fn install_preprocessor(dir: &PathBuf) -> Result<(), Error> {
    println!("Installing mdbook-langtabs in {:?}", dir);

    // 1. Check if book.toml exists
    let toml_path = dir.join("book.toml");
    if !toml_path.exists() {
        return Err(Error::msg(format!(
            "Could not find book.toml in {}",
            dir.display()
        )));
    }

    // 2. Copy assets to the root of the book directory
    let css_content = include_str!("../assets/langtabs.css");
    let js_content = include_str!("../assets/langtabs.js");

    fs::write(dir.join("langtabs.css"), css_content)?;
    fs::write(dir.join("langtabs.js"), js_content)?;

    println!("Created asset files in book root directory");

    // 3. Update book.toml configuration
    let mut toml_content = fs::read_to_string(&toml_path)?;

    // Check if the langtabs preprocessor is already configured
    if !toml_content.contains("[preprocessor.langtabs]") {
        toml_content.push_str("\n[preprocessor.langtabs]\ncommand = \"mdbook-langtabs\"\n");
        println!("Added preprocessor config to book.toml");
    }

    // Check if additional-css and additional-js are already configured
    let mut css_configured = false;
    let mut js_configured = false;

    if let Some(html_section) = toml_content.find("[output.html]") {
        if toml_content[html_section..].contains("additional-css") {
            if toml_content.contains("langtabs.css") {
                css_configured = true;
            } else {
                toml_content = toml_content
                    .replace("additional-css = [", "additional-css = [\"langtabs.css\", ");
                println!("Added CSS to existing additional-css list");
            }
        }

        if toml_content[html_section..].contains("additional-js") {
            if toml_content.contains("langtabs.js") {
                js_configured = true;
            } else {
                toml_content =
                    toml_content.replace("additional-js = [", "additional-js = [\"langtabs.js\", ");
                println!("Added JS to existing additional-js list");
            }
        }
    }

    // If [output.html] section doesn't exist or doesn't have our entries, add it
    if !css_configured || !js_configured {
        if !toml_content.contains("[output.html]") {
            toml_content.push_str("\n[output.html]\n");
        }

        if !css_configured {
            toml_content.push_str("additional-css = [\"langtabs.css\"]\n");
            println!("Added CSS configuration to book.toml");
        }

        if !js_configured {
            toml_content.push_str("additional-js = [\"langtabs.js\"]\n");
            println!("Added JS configuration to book.toml");
        }
    }

    fs::write(toml_path, toml_content)?;

    println!("mdbook-langtabs installation complete!");
    println!("You can now use language tabs in your book by enclosing code blocks in:");
    println!("<!-- langtabs-start -->");
    println!("```language");
    println!("code here");
    println!("```");
    println!("<!-- langtabs-end -->");

    Ok(())
}
