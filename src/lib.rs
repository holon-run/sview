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
}
