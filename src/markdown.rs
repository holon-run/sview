use crate::{
    analyzer::{build_tree, AnalyzerItem},
    model::Node,
    util::truncate_preview,
};

pub(crate) fn analyze_markdown(source: &str, preview_len: usize) -> Vec<Node> {
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

    let analyzer_items: Vec<AnalyzerItem> = items
        .into_iter()
        .map(|item| AnalyzerItem {
            parent: item.parent,
            node: item.node,
        })
        .collect();
    build_tree(&analyzer_items, None)
}

#[derive(Debug, Clone)]
struct MarkdownItem {
    parent: Option<usize>,
    heading_level: Option<usize>,
    node: Node,
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
