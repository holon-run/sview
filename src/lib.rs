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
}
