use mdbook::book::Chapter;
use pulldown_cmark::{CodeBlockKind, Event, Options, Parser, Tag, TagEnd};
use regex::Regex;
use uuid::Uuid;

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

        let lang_blocks = extract_code_blocks(inner_content);
        if !lang_blocks.is_empty() {
            let html = generate_tabs_html(&lang_blocks);
            result = result.replace(full_match, &html);
        }
    }

    result
}

fn extract_code_blocks(content: &str) -> Vec<(String, String)> {
    let mut code_blocks = Vec::new();
    let mut current_lang = String::new();
    let mut current_code = String::new();
    let mut in_code_block = false;

    let parser = Parser::new_ext(content, Options::all());

    for event in parser {
        match event {
            Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(lang))) => {
                in_code_block = true;
                current_lang = lang.to_string();
                current_code.clear();
            }
            Event::End(TagEnd::CodeBlock) => {
                if in_code_block && !current_lang.is_empty() {
                    code_blocks.push((current_lang.clone(), current_code.clone()));
                }
                in_code_block = false;
            }
            Event::Text(text) => {
                if in_code_block {
                    current_code.push_str(&text);
                }
            }
            Event::Code(text) => {
                if in_code_block {
                    current_code.push_str(&text);
                }
            }
            _ => {}
        }
    }
    code_blocks
}

fn generate_tabs_html(lang_blocks: &[(String, String)]) -> String {
    if lang_blocks.is_empty() {
        return String::new();
    }

    let tab_id = format!("langtabs-{}", Uuid::new_v4());

    let mut html = format!(r#"<div class="langtabs" id="{}">"#, tab_id);

    html.push_str(r#"<div class="langtabs-header">"#);

    for (i, (lang, _)) in lang_blocks.iter().enumerate() {
        let clean_lang = lang.split_whitespace().next().unwrap_or(lang);
        let class = if i == 0 {
            "langtabs-tab active"
        } else {
            "langtabs-tab"
        };
        let lang_display = get_language_display_name(clean_lang);
        let icon_html = get_language_icon(clean_lang);

        html.push_str(&format!(
            r#"<button class="{}" data-lang="{}">{}{}</button>"#,
            class, clean_lang, icon_html, lang_display
        ));
    }

    html.push_str("</div>");

    html.push_str(r#"<div class="langtabs-content">"#);

    for (i, (lang, code)) in lang_blocks.iter().enumerate() {
        let clean_lang = lang.split_whitespace().next().unwrap_or(lang);
        let class = if i == 0 {
            "langtabs-code active"
        } else {
            "langtabs-code"
        };

        html.push_str(&format!(
            r#"<div class="{}" data-lang="{}">"#,
            class, clean_lang
        ));

        html.push_str(&format!(
            r#"<pre><code class="language-{}">{}</code></pre>"#,
            clean_lang,
            html_escape(code)
        ));

        html.push_str("</div>");
    }

    html.push_str("</div>");
    html.push_str("</div>");

    html
}

fn get_language_display_name(lang: &str) -> String {
    match lang.to_lowercase().as_str() {
        "apache" => "Apache".to_string(),
        "armasm" => "ARM Assembly".to_string(),
        "bash" => "Bash".to_string(),
        "c" => "C".to_string(),
        "coffeescript" => "CoffeeScript".to_string(),
        "cpp" => "C++".to_string(),
        "csharp" | "cs" => "C#".to_string(),
        "css" => "CSS".to_string(),
        "d" => "D".to_string(),
        "diff" => "Diff".to_string(),
        "go" => "Go".to_string(),
        "handlebars" | "hbs" => "Handlebars".to_string(),
        "haskell" | "hs" => "Haskell".to_string(),
        "http" => "HTTP".to_string(),
        "ini" => "INI".to_string(),
        "java" => "Java".to_string(),
        "javascript" | "js" => "JavaScript".to_string(),
        "json" => "JSON".to_string(),
        "julia" => "Julia".to_string(),
        "kotlin" | "kt" => "Kotlin".to_string(),
        "less" => "Less".to_string(),
        "lua" => "Lua".to_string(),
        "makefile" | "make" => "Makefile".to_string(),
        "markdown" | "md" => "Markdown".to_string(),
        "nginx" => "Nginx".to_string(),
        "nim" => "Nim".to_string(),
        "nix" => "Nix".to_string(),
        "objectivec" | "objc" => "Objective-C".to_string(),
        "perl" | "pl" => "Perl".to_string(),
        "php" => "PHP".to_string(),
        "plaintext" | "text" | "txt" => "Plain Text".to_string(),
        "properties" | "props" => "Properties".to_string(),
        "python" | "py" => "Python".to_string(),
        "r" => "R".to_string(),
        "ruby" | "rb" => "Ruby".to_string(),
        "rust" | "rs" => "Rust".to_string(),
        "scala" => "Scala".to_string(),
        "scss" => "SCSS".to_string(),
        "shell" | "sh" => "Shell".to_string(),
        "sql" => "SQL".to_string(),
        "swift" => "Swift".to_string(),
        "typescript" | "ts" => "TypeScript".to_string(),
        "x86asm" => "x86 Assembly".to_string(),
        "xml" => "XML".to_string(),
        "yaml" | "yml" => "YAML".to_string(),
        _ => lang.to_string(),
    }
}

/// Map language identifiers to Devicon class names
fn get_language_icon(lang: &str) -> String {
    let icon_class = match lang.to_lowercase().as_str() {
        "apache" => "devicon-apache-plain",
        "armasm" | "x86asm" => "devicon-devicon-plain", // Generic icon fallback
        "bash" | "shell" | "sh" => "devicon-bash-plain",
        "c" => "devicon-c-plain",
        "coffeescript" => "devicon-coffeescript-plain",
        "cpp" => "devicon-cplusplus-plain",
        "csharp" | "cs" => "devicon-csharp-plain",
        "css" => "devicon-css3-plain",
        "d" => "devicon-d3js-plain",
        "diff" => "devicon-git-plain",
        "go" => "devicon-go-plain",
        "handlebars" | "hbs" => "devicon-handlebars-plain",
        "haskell" | "hs" => "devicon-haskell-plain",
        "html" => "devicon-html5-plain",
        "http" => "devicon-chrome-plain",
        "ini" | "properties" | "props" => "devicon-devicon-plain",
        "java" => "devicon-java-plain",
        "javascript" | "js" => "devicon-javascript-plain",
        "json" => "devicon-javascript-plain",
        "julia" => "devicon-julia-plain",
        "kotlin" | "kt" => "devicon-kotlin-plain",
        "less" => "devicon-less-plain-wordmark",
        "lua" => "devicon-lua-plain",
        "makefile" | "make" => "devicon-linux-plain",
        "markdown" | "md" => "devicon-markdown-plain",
        "nginx" => "devicon-nginx-plain",
        "nim" => "devicon-nim-plain",
        "nix" => "devicon-nixos-plain",
        "objectivec" | "objc" => "devicon-apple-plain",
        "perl" | "pl" => "devicon-perl-plain",
        "php" => "devicon-php-plain",
        "plaintext" | "text" | "txt" => "devicon-devicon-plain",
        "python" | "py" => "devicon-python-plain",
        "r" => "devicon-r-plain",
        "ruby" | "rb" => "devicon-ruby-plain",
        "rust" | "rs" => "devicon-rust-plain",
        "scala" => "devicon-scala-plain",
        "scss" => "devicon-sass-plain",
        "sql" => "devicon-mysql-plain",
        "swift" => "devicon-swift-plain",
        "typescript" | "ts" => "devicon-typescript-plain",
        "xml" => "devicon-html5-plain",
        "yaml" | "yml" => "devicon-devicon-plain",
        _ => "devicon-devicon-plain",
    };

    format!(r#"<i class="langtabs-icon {}"></i>"#, icon_class)
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
