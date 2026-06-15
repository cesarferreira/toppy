pub fn truncate_to_width(text: &str, max_chars: usize) -> String {
    if max_chars == 0 {
        return String::new();
    }
    // Fast path: all-ASCII text and short enough to skip char counting.
    if text.is_ascii() && text.len() <= max_chars {
        return text.to_string();
    }

    let mut out = String::with_capacity(text.len().min(max_chars * 4));
    let mut taken = 0usize;
    let mut chars = text.chars();
    while let Some(ch) = chars.next() {
        if taken + 1 == max_chars && chars.clone().next().is_some() {
            out.push('…');
            return out;
        }
        out.push(ch);
        taken += 1;
        if taken == max_chars {
            return out;
        }
    }
    out
}
