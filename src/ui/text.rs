pub fn truncate_to_width(text: &str, max_chars: usize) -> String {
    if max_chars == 0 {
        return String::new();
    }
    if text.chars().count() <= max_chars {
        return text.to_string();
    }
    if max_chars == 1 {
        return "…".to_string();
    }
    let truncated: String = text.chars().take(max_chars - 1).collect();
    format!("{truncated}…")
}
