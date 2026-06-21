use crate::model::{Language, Node, RenderOptions, StructureView};

pub fn render_text(view: &StructureView, options: &RenderOptions) -> String {
    let mut out = format!("{} ({})\n", view.path, language_label(view.language));
    let mut remaining = options.max_nodes;

    for (index, node) in view.nodes.iter().enumerate() {
        render_node(
            node,
            "",
            index + 1 == view.nodes.len(),
            0,
            options,
            &mut remaining,
            &mut out,
        );
        if remaining == 0 {
            out.push_str("...\n");
            break;
        }
    }

    out
}

fn language_label(language: Language) -> &'static str {
    match language {
        Language::Markdown => "markdown",
        Language::Rust => "rust",
        Language::Unknown => "unknown",
    }
}

fn render_node(
    node: &Node,
    prefix: &str,
    is_last: bool,
    depth: usize,
    options: &RenderOptions,
    remaining: &mut usize,
    out: &mut String,
) {
    if *remaining == 0 || options.max_depth.is_some_and(|max_depth| depth > max_depth) {
        return;
    }

    *remaining -= 1;
    let name = node.name.as_deref().unwrap_or("<anonymous>");
    let connector = if is_last { "└─" } else { "├─" };
    out.push_str(&format!(
        "{prefix}{connector} {} {name} L{}-{}",
        node.kind, node.start_line, node.end_line
    ));
    if let Some(preview) = text_preview(node) {
        out.push_str(&format!(" — {preview}"));
    }
    out.push('\n');

    let child_prefix = format!("{prefix}{}", if is_last { "   " } else { "│  " });
    for (index, child) in node.children.iter().enumerate() {
        render_node(
            child,
            &child_prefix,
            index + 1 == node.children.len(),
            depth + 1,
            options,
            remaining,
            out,
        );
        if *remaining == 0 {
            break;
        }
    }
}

fn text_preview(node: &Node) -> Option<&str> {
    let preview = node.preview.as_deref()?.trim();
    let name = node.name.as_deref().unwrap_or_default();
    if preview.is_empty()
        || preview == name
        || preview
            .trim_start_matches('#')
            .trim()
            .trim_end_matches('#')
            .trim()
            == name
    {
        None
    } else {
        Some(preview)
    }
}
