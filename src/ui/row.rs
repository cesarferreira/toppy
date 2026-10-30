use ratatui::{style::Style, text::Span};

use crate::{process::model::ProcessRow, theme::RowColors, ui::text::truncate_to_width};

/// Memory use as a percentage of total RAM, for picking the MEM color.
pub fn mem_pct(proc: &ProcessRow, total_mem: u64) -> f32 {
    if total_mem == 0 {
        0.0
    } else {
        (proc.mem_bytes as f64 * 100.0 / total_mem as f64) as f32
    }
}

/// COMMAND column spans. The program name gets the CPU color; in full-command
/// mode only the basename of argv[0] does, and the path and args keep the base color.
pub fn command_spans(
    proc: &ProcessRow,
    full_command: bool,
    width: usize,
    colors: RowColors,
) -> Vec<Span<'static>> {
    let text = truncate_to_width(proc.command_label(full_command), width.max(1));
    if !full_command {
        return vec![Span::styled(text, Style::default().fg(colors.cpu))];
    }

    let argv0_end = text.find(' ').unwrap_or(text.len());
    let name_start = text[..argv0_end].rfind('/').map_or(0, |i| i + 1);
    let mut spans = Vec::with_capacity(3);
    if name_start > 0 {
        spans.push(Span::styled(
            text[..name_start].to_string(),
            Style::default().fg(colors.base),
        ));
    }
    spans.push(Span::styled(
        text[name_start..argv0_end].to_string(),
        Style::default().fg(colors.cpu),
    ));
    if argv0_end < text.len() {
        spans.push(Span::styled(
            text[argv0_end..].to_string(),
            Style::default().fg(colors.base),
        ));
    }
    spans
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::style::Color;

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

    const COLORS: RowColors = RowColors {
        base: Color::White,
        cpu: Color::Green,
        mem: Color::Blue,
    };

    fn parts(spans: &[Span]) -> Vec<(String, Color)> {
        spans
            .iter()
            .map(|s| (s.content.to_string(), s.style.fg.unwrap()))
            .collect()
    }

    #[test]
    fn full_command_highlights_basename_only() {
        let p = proc("/usr/bin/ffmpeg -v error", "ffmpeg");
        assert_eq!(
            parts(&command_spans(&p, true, 80, COLORS)),
            vec![
                ("/usr/bin/".into(), Color::White),
                ("ffmpeg".into(), Color::Green),
                (" -v error".into(), Color::White),
            ]
        );
    }

    #[test]
    fn bare_command_is_all_program_color() {
        let p = proc("toppy", "toppy");
        assert_eq!(
            parts(&command_spans(&p, true, 80, COLORS)),
            vec![("toppy".into(), Color::Green)]
        );
    }

    #[test]
    fn name_mode_uses_program_color() {
        let p = proc("/usr/bin/ffmpeg -v error", "ffmpeg");
        assert_eq!(
            parts(&command_spans(&p, false, 80, COLORS)),
            vec![("ffmpeg".into(), Color::Green)]
        );
    }
}
