use mdbook::book::Chapter;
use regex::Regex;

use crate::languages;

pub fn process_chapter(chapter: &mut Chapter) {
    chapter.content = process_markdown(&chapter.content);
}

pub fn process_markdown(content: &str) -> String {
    let start_comment = "<!-- langtabs-start -->";
    let end_comment = "<!-- langtabs-end -->";

    if !content.contains(start_comment) || !content.contains(end_comment) {
        return content.to_string();
    }

    // Find all language tab blocks
    let re = Regex::new(&format!(
        r"{}([\s\S]*?){}",
        regex::escape(start_comment),
        regex::escape(end_comment)
    ))
    .unwrap();

    let mut result = content.to_string();

    for cap in re.captures_iter(content) {
        let full_match = cap.get(0).unwrap().as_str();
        let inner_content = cap.get(1).unwrap().as_str();

        let lang_sections = extract_language_sections(inner_content);
        if !lang_sections.is_empty() {
            let html = generate_tabs_html(&lang_sections);
            result = result.replace(full_match, &html);
        }
    }

    result
}

// A struct to represent a language section with its content
struct LanguageSection {
    language: languages::ProgrammingLanguage,
    content: String,
}

// Extract language sections using a simple state machine
fn extract_language_sections(content: &str) -> Vec<LanguageSection> {
    let mut sections: Vec<LanguageSection> = Vec::new();
    
    // Split the content into lines
    let lines: Vec<&str> = content.lines().collect();
    
    // Regex for detecting code block start and end
    let start_block_regex = Regex::new(r"^```([a-zA-Z0-9_+-]+)\s*$").unwrap();
    let end_block_regex = Regex::new(r"^```\s*$").unwrap();
    
    // State machine variables
    let mut in_code_block = false;
    let mut current_language = String::new();
    let mut current_content = Vec::new();
    
    for line in lines {
        if !in_code_block {
            // Check if this line starts a code block
            if let Some(captures) = start_block_regex.captures(line) {
                in_code_block = true;
                current_language = captures[1].to_lowercase();
                current_content = vec![line.to_string()];
            }
            // Otherwise, ignore text outside of code blocks
        } else {
            // We're in a code block, add the line
            current_content.push(line.to_string());
            
            // Check if this line ends the code block
            if end_block_regex.is_match(line) {
                // Add the completed section
                sections.push(LanguageSection {
                    language: languages::ProgrammingLanguage::from_str(&current_language),
                    content: current_content.join("\n"),
                });
                
                // Reset state
                in_code_block = false;
                current_language = String::new();
                current_content = Vec::new();
            }
        }
    }
    
    // In case the last block wasn't properly closed
    if in_code_block && !current_content.is_empty() {
        sections.push(LanguageSection {
            language: languages::ProgrammingLanguage::from_str(&current_language),
            content: current_content.join("\n"),
        });
    }
    
    sections
}

fn generate_tabs_html(sections: &[LanguageSection]) -> String {
    if sections.is_empty() {
        return String::new();
    }

    let mut html = format!(r#"<div class="langtabs">"#);

    // Generate tab headers
    html.push_str(r#"<div class="langtabs-header">"#);
    for (i, section) in sections.iter().enumerate() {
        let class = if i == 0 {
            "langtabs-tab active"
        } else {
            "langtabs-tab"
        };

        html.push_str(&format!(
            r#"<button class="{}" data-lang="{}-{}"><i class="langtabs-icon {}"></i>{}</button>"#,
            class, 
            section.language.to_identifier(), 
            i,
            section.language.icon_class(), 
            section.language.display_name(),
        ));
    }
    html.push_str("</div>");

    // Generate tab content with raw markdown
    html.push_str(r#"<div class="langtabs-content">"#);
    html.push_str("\n\n");
    for (i, section) in sections.iter().enumerate() {
        let class = if i == 0 {
            "langtabs-code active"
        } else {
            "langtabs-code"
        };

        // Create a div with the markdown content inside
        html.push_str("\n\n");
        html.push_str(&format!(
            r#"<div class="{}" data-lang="{}-{}">"#,
            class, section.language.to_identifier(), i
        ));
        html.push_str("\n\n");

        // Insert the raw markdown content
        html.push_str(&section.content);
        
        html.push_str("\n\n</div>");
    }
    html.push_str("</div>");
    html.push_str("</div>");

    html
}