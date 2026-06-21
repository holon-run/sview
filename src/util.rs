use std::path::Path;

pub(crate) fn file_name(path: &str) -> String {
    Path::new(path)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(path)
        .to_string()
}

pub(crate) fn first_non_empty_preview(source: &str, preview_len: usize) -> Option<String> {
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
