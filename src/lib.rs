use anyhow::{Context, Result};
use serde::Serialize;
use std::{fs, path::Path};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct StructureView {
    pub path: String,
    pub language: Language,
    pub nodes: Vec<Node>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Language {
    Markdown,
    Rust,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Node {
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub level: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub start_line: usize,
    pub end_line: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preview: Option<String>,
    pub children: Vec<Node>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RenderOptions {
    pub max_depth: Option<usize>,
    pub max_nodes: usize,
}

pub fn analyze_file(path: &Path, preview_len: usize) -> Result<StructureView> {
    let source =
        fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?;
    let language = detect_language(path, &source);
    Ok(analyze_source(
        path.to_string_lossy().as_ref(),
        language,
        &source,
        preview_len,
    ))
}

pub fn analyze_source(
    path: &str,
    language: Language,
    source: &str,
    preview_len: usize,
) -> StructureView {
    let line_count = source.lines().count().max(1);
    let nodes = if source.trim().is_empty() {
        Vec::new()
    } else if language == Language::Markdown {
        analyze_markdown(source, preview_len)
    } else if language == Language::Rust {
        analyze_rust(source, preview_len)
    } else {
        vec![Node {
            kind: "file".to_string(),
            level: None,
            name: Some(file_name(path)),
            start_line: 1,
            end_line: line_count,
            preview: first_non_empty_preview(source, preview_len),
            children: Vec::new(),
        }]
    };

    StructureView {
        path: path.to_string(),
        language,
        nodes,
    }
}

pub fn render_text(view: &StructureView, options: &RenderOptions) -> String {
    let mut out = format!("{} ({:?})\n", view.path, view.language);
    let mut remaining = options.max_nodes;

    for node in &view.nodes {
        render_node(node, 0, options, &mut remaining, &mut out);
        if remaining == 0 {
            out.push_str("...\n");
            break;
        }
    }

    out
}

pub fn detect_language(path: &Path, source: &str) -> Language {
    match path.extension().and_then(|extension| extension.to_str()) {
        Some("md" | "markdown" | "mdown") => Language::Markdown,
        Some("rs") => Language::Rust,
        _ if source.trim_start().starts_with("# ") => Language::Markdown,
        _ => Language::Unknown,
    }
}

fn render_node(
    node: &Node,
    depth: usize,
    options: &RenderOptions,
    remaining: &mut usize,
    out: &mut String,
) {
    if *remaining == 0 || options.max_depth.is_some_and(|max_depth| depth > max_depth) {
        return;
    }

    *remaining -= 1;
    let indent = "  ".repeat(depth);
    let name = node.name.as_deref().unwrap_or("<anonymous>");
    out.push_str(&format!(
        "{indent}- {} {name} [{}-{}]",
        node.kind, node.start_line, node.end_line
    ));
    if let Some(preview) = &node.preview {
        out.push_str(&format!(" :: {preview}"));
    }
    out.push('\n');

    for child in &node.children {
        render_node(child, depth + 1, options, remaining, out);
        if *remaining == 0 {
            break;
        }
    }
}

fn analyze_markdown(source: &str, preview_len: usize) -> Vec<Node> {
    let lines = source.lines().collect::<Vec<_>>();
    let mut items = Vec::<MarkdownItem>::new();
    let mut heading_stack = Vec::<(usize, usize)>::new();
    let mut line_index = 0;

    if lines.first().is_some_and(|line| line.trim() == "---") {
        if let Some(end_index) = lines.iter().skip(1).position(|line| line.trim() == "---") {
            let end_line = end_index + 2;
            items.push(MarkdownItem {
                parent: None,
                heading_level: None,
                node: Node {
                    kind: "frontmatter".to_string(),
                    level: None,
                    name: Some("frontmatter".to_string()),
                    start_line: 1,
                    end_line,
                    preview: Some("---".to_string()),
                    children: Vec::new(),
                },
            });
            line_index = end_line;
        }
    }

    while line_index < lines.len() {
        let line = lines[line_index];
        let line_number = line_index + 1;

        if let Some((level, name)) = parse_heading(line) {
            while heading_stack
                .last()
                .is_some_and(|(stack_level, _)| *stack_level >= level)
            {
                heading_stack.pop();
            }
            let parent = heading_stack.last().map(|(_, index)| *index);
            let index = items.len();
            items.push(MarkdownItem {
                parent,
                heading_level: Some(level),
                node: Node {
                    kind: "heading".to_string(),
                    level: Some(level),
                    name: Some(name),
                    start_line: line_number,
                    end_line: lines.len(),
                    preview: Some(truncate_preview(line.trim(), preview_len)),
                    children: Vec::new(),
                },
            });
            heading_stack.push((level, index));
            line_index += 1;
            continue;
        }

        if line.trim_start().starts_with("```") || line.trim_start().starts_with("~~~") {
            let fence = &line.trim_start()[..3];
            let start_line = line_number;
            line_index += 1;
            while line_index < lines.len() && !lines[line_index].trim_start().starts_with(fence) {
                line_index += 1;
            }
            let end_line = if line_index < lines.len() {
                line_index += 1;
                line_index
            } else {
                lines.len()
            };
            items.push(MarkdownItem {
                parent: heading_stack.last().map(|(_, index)| *index),
                heading_level: None,
                node: Node {
                    kind: "code_block".to_string(),
                    level: None,
                    name: code_block_name(line),
                    start_line,
                    end_line,
                    preview: Some(truncate_preview(line.trim(), preview_len)),
                    children: Vec::new(),
                },
            });
            continue;
        }

        if is_list_item(line) {
            let start_line = line_number;
            line_index += 1;
            while line_index < lines.len()
                && (is_list_item(lines[line_index]) || lines[line_index].trim().is_empty())
            {
                line_index += 1;
            }
            items.push(MarkdownItem {
                parent: heading_stack.last().map(|(_, index)| *index),
                heading_level: None,
                node: Node {
                    kind: "list".to_string(),
                    level: None,
                    name: Some("list".to_string()),
                    start_line,
                    end_line: line_index,
                    preview: Some(truncate_preview(line.trim(), preview_len)),
                    children: Vec::new(),
                },
            });
            continue;
        }

        line_index += 1;
    }

    for index in 0..items.len() {
        if let Some(level) = items[index].heading_level {
            if let Some(next_heading) = items[index + 1..].iter().find(|item| {
                item.heading_level
                    .is_some_and(|next_level| next_level <= level)
            }) {
                items[index].node.end_line = next_heading.node.start_line.saturating_sub(1);
            }
        }
    }

    build_markdown_tree(&items, None)
}

#[derive(Debug, Clone)]
struct MarkdownItem {
    parent: Option<usize>,
    heading_level: Option<usize>,
    node: Node,
}

fn build_markdown_tree(items: &[MarkdownItem], parent: Option<usize>) -> Vec<Node> {
    items
        .iter()
        .enumerate()
        .filter(|(_, item)| item.parent == parent)
        .map(|(index, item)| {
            let mut node = item.node.clone();
            node.children = build_markdown_tree(items, Some(index));
            node
        })
        .collect()
}

fn parse_heading(line: &str) -> Option<(usize, String)> {
    let trimmed = line.trim_start();
    let level = trimmed
        .chars()
        .take_while(|character| *character == '#')
        .count();
    if !(1..=6).contains(&level) || !trimmed[level..].starts_with(' ') {
        return None;
    }

    let name = trimmed[level..].trim().trim_end_matches('#').trim();
    if name.is_empty() {
        None
    } else {
        Some((level, name.to_string()))
    }
}

fn code_block_name(line: &str) -> Option<String> {
    let info = line.trim_start()[3..].trim();
    (!info.is_empty()).then(|| info.to_string())
}

fn is_list_item(line: &str) -> bool {
    let trimmed = line.trim_start();
    if trimmed.starts_with("- ") || trimmed.starts_with("* ") || trimmed.starts_with("+ ") {
        return true;
    }

    let Some((digits, rest)) = trimmed.split_once('.') else {
        return false;
    };
    !digits.is_empty()
        && digits.chars().all(|character| character.is_ascii_digit())
        && rest.starts_with(' ')
}

fn analyze_rust(source: &str, preview_len: usize) -> Vec<Node> {
    let mut items = Vec::<RustItem>::new();
    let mut stack = Vec::<RustStackEntry>::new();
    let mut brace_depth = 0usize;
    let mut pending_attributes = Vec::<String>::new();
    let mut pending_block = None::<RustStackEntry>;

    for (line_index, line) in source.lines().enumerate() {
        let line_number = line_index + 1;
        let trimmed = line.trim_start();

        if trimmed.starts_with("#[") {
            pending_attributes.push(trimmed.to_string());
        } else if let Some((kind, name)) = parse_rust_item(trimmed, &pending_attributes) {
            let parent = stack.last().map(|entry| entry.item_index);
            let index = items.len();
            let opens_block = line_contains_code_char(trimmed, '{');
            items.push(RustItem {
                parent,
                node: Node {
                    kind,
                    level: None,
                    name: Some(name),
                    start_line: line_number,
                    end_line: line_number,
                    preview: Some(truncate_preview(trimmed, preview_len)),
                    children: Vec::new(),
                },
            });

            if opens_block {
                stack.push(RustStackEntry {
                    item_index: index,
                    close_depth: brace_depth,
                });
            } else if !trimmed.ends_with(';') {
                pending_block = Some(RustStackEntry {
                    item_index: index,
                    close_depth: brace_depth,
                });
            }

            pending_attributes.clear();
        } else if !trimmed.is_empty() && !trimmed.starts_with("//") {
            pending_attributes.clear();
        }

        if let Some(entry) = pending_block {
            if line_contains_code_char(trimmed, '{') {
                stack.push(entry);
                pending_block = None;
            } else if line_contains_code_char(trimmed, ';') {
                items[entry.item_index].node.end_line = line_number;
                pending_block = None;
            }
        }

        brace_depth = update_brace_depth(brace_depth, line);

        while stack
            .last()
            .is_some_and(|entry| brace_depth <= entry.close_depth)
        {
            let entry = stack.pop().expect("stack entry exists");
            items[entry.item_index].node.end_line = line_number;
        }
    }

    let final_line = source.lines().count().max(1);
    if let Some(entry) = pending_block {
        items[entry.item_index].node.end_line = final_line;
    }
    for entry in stack {
        items[entry.item_index].node.end_line = final_line;
    }

    build_rust_tree(&items, None)
}

#[derive(Debug, Clone)]
struct RustItem {
    parent: Option<usize>,
    node: Node,
}

#[derive(Debug, Clone, Copy)]
struct RustStackEntry {
    item_index: usize,
    close_depth: usize,
}

fn build_rust_tree(items: &[RustItem], parent: Option<usize>) -> Vec<Node> {
    items
        .iter()
        .enumerate()
        .filter(|(_, item)| item.parent == parent)
        .map(|(index, item)| {
            let mut node = item.node.clone();
            node.children = build_rust_tree(items, Some(index));
            node
        })
        .collect()
}

fn parse_rust_item(line: &str, attributes: &[String]) -> Option<(String, String)> {
    let line = strip_rust_visibility_and_modifiers(line);
    if let Some(name) = rust_name_after_keyword(line, "mod") {
        return Some(("module".to_string(), name));
    }
    if let Some(name) = rust_name_after_keyword(line, "struct") {
        return Some(("struct".to_string(), name));
    }
    if let Some(name) = rust_name_after_keyword(line, "enum") {
        return Some(("enum".to_string(), name));
    }
    if let Some(name) = rust_name_after_keyword(line, "trait") {
        return Some(("trait".to_string(), name));
    }
    if line.starts_with("impl") && keyword_boundary(line, "impl") {
        return Some(("impl".to_string(), rust_impl_name(line)));
    }
    if let Some(name) = rust_name_after_keyword(line, "fn") {
        let kind = if attributes.iter().any(|attribute| {
            attribute.starts_with("#[test") || attribute.starts_with("#[tokio::test")
        }) {
            "test"
        } else {
            "function"
        };
        return Some((kind.to_string(), name));
    }

    None
}

fn strip_rust_visibility_and_modifiers(mut line: &str) -> &str {
    loop {
        let trimmed = line.trim_start();
        if let Some(rest) = trimmed.strip_prefix("pub ") {
            line = rest;
        } else if trimmed.starts_with("pub(") {
            if let Some(end) = trimmed.find(") ") {
                line = &trimmed[end + 2..];
            } else {
                return trimmed;
            }
        } else if let Some(rest) = trimmed.strip_prefix("async ") {
            line = rest;
        } else if let Some(rest) = trimmed.strip_prefix("const ") {
            line = rest;
        } else if let Some(rest) = trimmed.strip_prefix("unsafe ") {
            line = rest;
        } else {
            return trimmed;
        }
    }
}

fn rust_name_after_keyword(line: &str, keyword: &str) -> Option<String> {
    if !line.starts_with(keyword) || !keyword_boundary(line, keyword) {
        return None;
    }
    let rest = line[keyword.len()..].trim_start();
    let name = rest
        .chars()
        .take_while(|character| {
            character.is_ascii_alphanumeric() || *character == '_' || *character == '!'
        })
        .collect::<String>();
    (!name.is_empty()).then_some(name)
}

fn keyword_boundary(line: &str, keyword: &str) -> bool {
    line[keyword.len()..]
        .chars()
        .next()
        .is_some_and(|character| character.is_whitespace() || character == '<')
}

fn rust_impl_name(line: &str) -> String {
    let signature = line
        .split('{')
        .next()
        .unwrap_or(line)
        .trim()
        .trim_end_matches(';')
        .trim();
    truncate_preview(signature, 80)
}

fn update_brace_depth(mut depth: usize, line: &str) -> usize {
    let mut chars = line.chars().peekable();
    let mut in_string = false;
    let mut in_char = false;
    let mut escaped = false;

    while let Some(character) = chars.next() {
        if !in_string && !in_char && character == '/' && chars.peek() == Some(&'/') {
            break;
        }

        if escaped {
            escaped = false;
            continue;
        }
        if in_string || in_char {
            if character == '\\' {
                escaped = true;
            } else if in_string && character == '"' {
                in_string = false;
            } else if in_char && character == '\'' {
                in_char = false;
            }
            continue;
        }

        if character == '"' {
            in_string = true;
            continue;
        }
        if character == '\'' {
            in_char = true;
            continue;
        }

        match character {
            '{' => depth += 1,
            '}' => depth = depth.saturating_sub(1),
            _ => {}
        }
    }

    depth
}

fn line_contains_code_char(line: &str, expected: char) -> bool {
    line.split("//")
        .next()
        .unwrap_or(line)
        .chars()
        .any(|character| character == expected)
}

fn file_name(path: &str) -> String {
    Path::new(path)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(path)
        .to_string()
}

fn first_non_empty_preview(source: &str, preview_len: usize) -> Option<String> {
    source
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .map(|line| truncate_preview(line, preview_len))
}

pub fn truncate_preview(value: &str, max_len: usize) -> String {
    if value.chars().count() <= max_len {
        return value.to_string();
    }

    let mut truncated = value
        .chars()
        .take(max_len.saturating_sub(1))
        .collect::<String>();
    truncated.push('…');
    truncated
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_language_from_extension() {
        assert_eq!(
            detect_language(Path::new("README.md"), ""),
            Language::Markdown
        );
        assert_eq!(detect_language(Path::new("src/lib.rs"), ""), Language::Rust);
        assert_eq!(detect_language(Path::new("LICENSE"), ""), Language::Unknown);
    }

    #[test]
    fn creates_file_node_for_non_empty_source() {
        let view = analyze_source("example.txt", Language::Unknown, "\nhello\nworld\n", 20);

        assert_eq!(view.nodes.len(), 1);
        assert_eq!(view.nodes[0].kind, "file");
        assert_eq!(view.nodes[0].start_line, 1);
        assert_eq!(view.nodes[0].end_line, 3);
        assert_eq!(view.nodes[0].preview.as_deref(), Some("hello"));
    }

    #[test]
    fn truncates_preview_on_character_boundary() {
        assert_eq!(truncate_preview("abcdef", 4), "abc…");
        assert_eq!(truncate_preview("ab", 4), "ab");
    }

    #[test]
    fn extracts_markdown_sections() {
        let source = "---\ntitle: Test\n---\n# Intro\n\n- one\n- two\n\n## Details\n\n```rust\nfn main() {}\n```\n# Next\n";
        let view = analyze_source("README.md", Language::Markdown, source, 40);

        assert_eq!(view.nodes[0].kind, "frontmatter");
        assert_eq!(view.nodes[0].start_line, 1);
        assert_eq!(view.nodes[0].end_line, 3);

        let intro = &view.nodes[1];
        assert_eq!(intro.kind, "heading");
        assert_eq!(intro.name.as_deref(), Some("Intro"));
        assert_eq!(intro.start_line, 4);
        assert_eq!(intro.end_line, 13);
        assert_eq!(intro.children[0].kind, "list");
        assert_eq!(intro.children[1].name.as_deref(), Some("Details"));
        assert_eq!(intro.children[1].children[0].kind, "code_block");
        assert_eq!(view.nodes[2].name.as_deref(), Some("Next"));
    }

    #[test]
    fn extracts_rust_items() {
        let source = "pub mod api {\n    pub struct Client;\n\n    impl Client {\n        pub fn new() -> Self {\n            Self\n        }\n    }\n}\n\n#[cfg(test)]\nmod tests {\n    #[test]\n    fn creates_client() {}\n}\n";
        let view = analyze_source("src/lib.rs", Language::Rust, source, 80);

        let api = &view.nodes[0];
        assert_eq!(api.kind, "module");
        assert_eq!(api.name.as_deref(), Some("api"));
        assert_eq!(api.start_line, 1);
        assert_eq!(api.end_line, 9);
        assert_eq!(api.children[0].kind, "struct");
        assert_eq!(api.children[1].kind, "impl");
        assert_eq!(api.children[1].children[0].kind, "function");
        assert_eq!(api.children[1].children[0].name.as_deref(), Some("new"));
        assert_eq!(api.children[1].children[0].end_line, 7);

        let tests = &view.nodes[1];
        assert_eq!(tests.name.as_deref(), Some("tests"));
        assert_eq!(tests.children[0].kind, "test");
        assert_eq!(tests.children[0].name.as_deref(), Some("creates_client"));
    }

    #[test]
    fn tracks_multiline_rust_signatures() {
        let source = "pub fn build(\n    input: String,\n) -> String {\n    input\n}\n";
        let view = analyze_source("src/lib.rs", Language::Rust, source, 80);

        assert_eq!(view.nodes[0].kind, "function");
        assert_eq!(view.nodes[0].name.as_deref(), Some("build"));
        assert_eq!(view.nodes[0].start_line, 1);
        assert_eq!(view.nodes[0].end_line, 5);
    }

    #[test]
    fn braces_in_char_literals_do_not_hide_closing_blocks() {
        let source = "fn contains_quote() {\n    let quote = '\"';\n}\nfn next() {}\n";
        let view = analyze_source("src/lib.rs", Language::Rust, source, 80);

        assert_eq!(view.nodes[0].name.as_deref(), Some("contains_quote"));
        assert_eq!(view.nodes[0].end_line, 3);
        assert_eq!(view.nodes[1].name.as_deref(), Some("next"));
    }
}
