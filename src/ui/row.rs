use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
};

use crate::{process::model::ProcessRow, theme, ui::text::truncate_to_width};

/// Give selected rows a consistent high-contrast style across the full panel width.
pub fn process_line(spans: Vec<Span<'static>>, selected: bool) -> Line<'static> {
    let mut line = Line::from(spans);
    if selected {
        let style = Style::default()
            .fg(theme::SELECTED_FG)
            .bg(theme::SELECTED_BG)
            .add_modifier(Modifier::BOLD);
        for span in &mut line.spans {
            span.style = style;
        }
        line.style = style;
    }
    line
}

/// Memory use as a percentage of total RAM, for picking the MEM color.
pub fn mem_pct(proc: &ProcessRow, total_mem: u64) -> f32 {
    if total_mem == 0 {
        0.0
    } else {
        (proc.mem_bytes as f64 * 100.0 / total_mem as f64) as f32
    }
}

/// COMMAND column text is white in both display modes.
pub fn command_spans(proc: &ProcessRow, full_command: bool, width: usize) -> Vec<Span<'static>> {
    let text = truncate_to_width(proc.command_label(full_command), width.max(1));
    vec![Span::styled(text, Style::default().fg(Color::White))]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn proc(cmd: &str, name: &str) -> ProcessRow {
        ProcessRow {
            pid: 1,
            start_time: 0,
            cpu: 0.0,
            mem_bytes: 0,
            cmd: cmd.into(),
            cmd_lower: cmd.to_lowercase(),
            name: name.into(),
            name_lower: name.to_lowercase(),
            parent_pid: None,
        }
    }

    fn parts(spans: &[Span]) -> Vec<(String, Color)> {
        spans
            .iter()
            .map(|s| (s.content.to_string(), s.style.fg.unwrap()))
            .collect()
    }

    #[test]
    fn full_command_is_white() {
        let p = proc("/usr/bin/ffmpeg -v error", "ffmpeg");
        assert_eq!(
            parts(&command_spans(&p, true, 80)),
            vec![("/usr/bin/ffmpeg -v error".into(), Color::White)]
        );
    }

    #[test]
    fn bare_command_is_white() {
        let p = proc("toppy", "toppy");
        assert_eq!(
            parts(&command_spans(&p, true, 80)),
            vec![("toppy".into(), Color::White)]
        );
    }

    #[test]
    fn name_mode_is_white() {
        let p = proc("/usr/bin/ffmpeg -v error", "ffmpeg");
        assert_eq!(
            parts(&command_spans(&p, false, 80)),
            vec![("ffmpeg".into(), Color::White)]
        );
    }
}
