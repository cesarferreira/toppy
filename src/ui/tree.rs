use ratatui::{
    Frame,
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders},
};

use crate::{
    app::App,
    metrics::format,
    theme::{self, TREE_BORDER},
    ui::row::{command_spans, mem_pct, process_line},
};

pub fn render_tree(frame: &mut Frame, area: Rect, app: &App) {
    let block = Block::default()
        .title(" Process Tree ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(TREE_BORDER));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let visible = inner.height as usize;

    let mut lines = Vec::with_capacity(visible);
    for (row_idx, &(pid, depth, has_children)) in app
        .flat_tree
        .iter()
        .skip(app.scroll_offset)
        .take(visible)
        .enumerate()
    {
        let Some(node) = app.tree.by_pid.get(&pid) else {
            continue;
        };
        let proc = &app.processes[node.index];
        let pos = row_idx + app.scroll_offset;
        let colors = theme::row_colors(
            proc.cpu,
            mem_pct(proc, app.memory.total),
            pos.abs_diff(app.selected),
            visible,
        );
        let branch = if !has_children {
            "  "
        } else if app.tree_expanded.contains(&pid) || !app.filter.is_empty() {
            "▼ "
        } else {
            "▶ "
        };
        lines.push(render_tree_row(
            proc,
            depth,
            branch,
            pos == app.selected,
            colors,
            app.show_full_command,
            inner.width,
        ));
    }

    if lines.is_empty() {
        lines.push(Line::from(Span::styled(
            "  no processes",
            theme::dim_style(),
        )));
    }

    for (line, area) in lines.into_iter().zip(inner.rows()) {
        frame.render_widget(line, area);
    }
}

fn render_tree_row(
    proc: &crate::process::model::ProcessRow,
    depth: usize,
    branch: &str,
    selected: bool,
    colors: theme::RowColors,
    full_command: bool,
    row_width: u16,
) -> Line<'static> {
    let indent = "  ".repeat(depth);
    let mem_text = format::format_bytes(proc.mem_bytes);
    let head = format!("{indent}{branch}{:>7} ", proc.pid);
    let cpu = format!("{:>5.1}% ", proc.cpu);
    let mem = format!("{:>8} ", mem_text);
    // head contains the tree indent which is ASCII spaces, plus a branch glyph
    // (▶/▼) that is 1 char but multi-byte. Count chars only when non-ASCII present.
    let head_len = if head.is_ascii() {
        head.len()
    } else {
        head.chars().count()
    };
    let prefix_len = head_len + cpu.len() + mem.len();
    let cmd_width = row_width.saturating_sub(prefix_len as u16) as usize;

    let mut spans = vec![
        Span::styled(head, Style::default().fg(colors.base)),
        Span::styled(cpu, Style::default().fg(colors.cpu)),
        Span::styled(mem, Style::default().fg(colors.mem)),
    ];
    spans.extend(command_spans(proc, full_command, cmd_width, colors));

    process_line(spans, selected)
}
