use crate::{
    cpp::analyze_cpp_like,
    java::analyze_java,
    javascript::analyze_javascript,
    kotlin::analyze_kotlin,
    markdown::analyze_markdown,
    model::{Language, Node, StructureView},
    objc::analyze_objc,
    rust::analyze_rust,
    swift::analyze_swift,
    util::{file_name, first_non_empty_preview},
};
use anyhow::{Context, Result};
use std::{fs, path::Path};

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
    } else if matches!(language, Language::C | Language::Cpp) {
        analyze_cpp_like(language, source, preview_len)
    } else if matches!(
        language,
        Language::JavaScript | Language::TypeScript | Language::Tsx
    ) {
        analyze_javascript(language, source, preview_len)
    } else if language == Language::Java {
        analyze_java(source, preview_len)
    } else if language == Language::Kotlin {
        analyze_kotlin(source, preview_len)
    } else if language == Language::ObjectiveC {
        analyze_objc(source, preview_len)
    } else if language == Language::Swift {
        analyze_swift(source, preview_len)
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

pub fn detect_language(path: &Path, source: &str) -> Language {
    match path.extension().and_then(|extension| extension.to_str()) {
        Some("h") if looks_like_objc(source) => Language::ObjectiveC,
        Some("c" | "h") => Language::C,
        Some("cc" | "cpp" | "cxx" | "hh" | "hpp" | "hxx") => Language::Cpp,
        Some("java") => Language::Java,
        Some("js" | "jsx") => Language::JavaScript,
        Some("kt" | "kts") => Language::Kotlin,
        Some("md" | "markdown" | "mdown") => Language::Markdown,
        Some("m" | "mm") => Language::ObjectiveC,
        Some("rs") => Language::Rust,
        Some("swift") => Language::Swift,
        Some("ts") => Language::TypeScript,
        Some("tsx") => Language::Tsx,
        _ if source.trim_start().starts_with("# ") => Language::Markdown,
        _ => Language::Unknown,
    }
}

fn looks_like_objc(source: &str) -> bool {
    let trimmed = source.trim_start();
    trimmed.starts_with("#import")
        || trimmed.contains("@interface")
        || trimmed.contains("@implementation")
        || trimmed.contains("@protocol")
}
