use clap::{Arg, Command};
use mdbook::errors::Error;
use mdbook::preprocess::{CmdPreprocessor, Preprocessor};
use mdbook_langtabs::LangTabsPreprocessor;
use std::fs;
use std::io;
use std::path::PathBuf;
use std::process;

/// Preprocessor command
const MDBOOK_LANGTABS: &str = "mdbook-langtabs";
/// Preprocessor short name
const LANGTABS: &str = "langtabs";

fn main() {
    let matches = Command::new(MDBOOK_LANGTABS)
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
                        .default_value(".")
                        .required(false),
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
        let dir_path = sub_args
            .get_one::<String>("dir")
            .map(PathBuf::from)
            .unwrap_or(PathBuf::from("."));

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

/// Install preprocessor to be used by book in dir.
///
/// Inspired by mdbook-mermaid.
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
    let css_path = dir.join("langtabs.css");
    let js_path = dir.join("langtabs.js");

    if !css_path.exists() {
        let css_content = include_str!("../assets/langtabs.css");
        fs::write(&css_path, css_content)?;
        println!("Created langtabs.css in book root directory");
    }

    if !js_path.exists() {
        let js_content = include_str!("../assets/langtabs.js");
        fs::write(&js_path, js_content)?;
        println!("Created langtabs.js in book root directory");
    }

    // 3. Update book.toml configuration
    let mut toml_doc = fs::read_to_string(&toml_path)?.parse::<toml_edit::DocumentMut>()?;

    // Check if the langtabs preprocessor is already configured
    let preprocessor_installed = toml_doc.get("preprocessor")
        .and_then(|p| p.as_table())
        .and_then(|t| t.get(LANGTABS))
        .is_some();

    if !preprocessor_installed {
        // Get or create the preprocessor table
        let preprocessor_table = toml_doc.entry("preprocessor")
            .or_insert_with(|| {
                let mut table = toml_edit::Table::new();
                table.set_implicit(true);
                toml_edit::Item::Table(table)
            })
            .as_table_mut()
            .unwrap();
        
        // Get or create the langtabs table inside the preprocessor table
        let langtabs_table = preprocessor_table.entry(LANGTABS)
            .or_insert_with(|| {
                let mut table = toml_edit::Table::new();
                table.set_implicit(true);
                toml_edit::Item::Table(table)
            })
            .as_table_mut()
            .unwrap();
        
        langtabs_table["command"] = toml_edit::value(MDBOOK_LANGTABS);
        println!("Added preprocessor config to book.toml");
    } else {
        println!("Preprocessor config already exists in book.toml");
    }

    // Add additional-css and additional-js of needed

    let html_section = toml_doc
        .entry("output")
        .or_insert_with(|| {
            let mut table = toml_edit::Table::new();
            table.set_implicit(true);
            toml_edit::Item::Table(table)
        })
        .as_table_mut()
        .unwrap()
        .entry("html")
        .or_insert_with(|| {
            let mut table = toml_edit::Table::new();
            table.set_implicit(true);
            toml_edit::Item::Table(table)
        })
        .as_table_mut()
        .unwrap();

    // Check and update additional-css
    let css_configured = if let Some(css_array) = html_section.get("additional-css") {
        if let Some(arr) = css_array.as_array() {
            // Check if langtabs.css is already in the array
            arr.iter()
                .any(|item| item.as_str().map(|s| s == "langtabs.css").unwrap_or(false))
        } else {
            false
        }
    } else {
        false
    };

    if !css_configured {
        let css_entry = html_section.entry("additional-css");
        if let toml_edit::Entry::Occupied(mut o) = css_entry {
            // Array exists, add to it
            if let Some(arr) = o.get_mut().as_array_mut() {
                arr.push("langtabs.css");
                println!("Added CSS to existing additional-css list");
            }
        } else {
            // Create new array with our value
            let mut arr = toml_edit::Array::new();
            arr.push("langtabs.css");
            html_section["additional-css"] = toml_edit::value(arr);
            println!("Added CSS configuration to book.toml");
        }
    }

    // Check and update additional-js
    let js_configured = if let Some(js_array) = html_section.get("additional-js") {
        if let Some(arr) = js_array.as_array() {
            // Check if langtabs.js is already in the array
            arr.iter()
                .any(|item| item.as_str().map(|s| s == "langtabs.js").unwrap_or(false))
        } else {
            false
        }
    } else {
        false
    };

    if !js_configured {
        let js_entry = html_section.entry("additional-js");
        if let toml_edit::Entry::Occupied(mut o) = js_entry {
            // Array exists, add to it
            if let Some(arr) = o.get_mut().as_array_mut() {
                arr.push("langtabs.js");
                println!("Added JS to existing additional-js list");
            }
        } else {
            // Create new array with our value
            let mut arr = toml_edit::Array::new();
            arr.push("langtabs.js");
            html_section["additional-js"] = toml_edit::value(arr);
            println!("Added JS configuration to book.toml");
        }
    }

    fs::write(toml_path, toml_doc.to_string())?;

    println!("mdbook-langtabs installation complete!");
    println!("You can now use language tabs in your book by enclosing code blocks in:");
    println!("<!-- langtabs-start -->");
    println!("```rust");
    println!("// Rust code here");
    println!("```");
    println!("```js");
    println!("// JavaScript code here");
    println!("```");
    println!("<!-- langtabs-end -->");

    Ok(())
}
