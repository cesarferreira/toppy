use ratatui::{
    Frame,
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use crate::{
    app::App,
    metrics::format,
    process::model::ProcessRow,
    theme::{self, PROC_BORDER},
    ui::row::{command_spans, mem_pct, process_line},
};

pub fn render_processes(frame: &mut Frame, area: Rect, app: &App) {
    let title = build_title(app);
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(PROC_BORDER));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.height < 2 {
        return;
    }

    let header = Line::from(vec![
        Span::styled("  PID", theme::header_style()),
        Span::styled("    CPU%", theme::header_style()),
        Span::styled("      MEM", theme::header_style()),
        Span::styled("  COMMAND", theme::header_style()),
    ]);

    let header_area = Rect { height: 1, ..inner };
    frame.render_widget(Paragraph::new(header), header_area);

    let list_area = Rect {
        y: inner.y + 1,
        height: inner.height.saturating_sub(1),
        ..inner
    };

    let visible = list_area.height as usize;
    let mut lines = Vec::new();

    for (row_idx, &proc_idx) in app
        .filtered_indices
        .iter()
        .skip(app.scroll_offset)
        .take(visible)
        .enumerate()
    {
        let proc = &app.processes[proc_idx];
        let pos = row_idx + app.scroll_offset;
        let colors = theme::row_colors(
            proc.cpu,
            mem_pct(proc, app.memory.total),
            pos.abs_diff(app.selected),
            visible,
        );
        lines.push(render_row(
            proc,
            pos == app.selected,
            colors,
            app.show_full_command,
            list_area.width,
        ));
    }

    if lines.is_empty() {
        lines.push(Line::from(Span::styled(
            "  no matching processes",
            theme::dim_style(),
        )));
    }

    for (line, area) in lines.into_iter().zip(list_area.rows()) {
        frame.render_widget(line, area);
    }
}

fn build_title(app: &App) -> String {
    let dir = if app.sort_desc { "desc" } else { "asc" };
    let filter = if app.filter.is_empty() {
        String::new()
    } else {
        format!("  filter: {}", app.filter)
    };
    format!(" Processes  sort: {} {dir}{filter} ", app.sort.label())
}

fn render_row(
    proc: &ProcessRow,
    selected: bool,
    colors: theme::RowColors,
    full_command: bool,
    row_width: u16,
) -> Line<'static> {
    let mem_text = format::format_bytes(proc.mem_bytes);
    let pid = format!("{:>7} ", proc.pid);
    let cpu = format!("{:>6.1}% ", proc.cpu);
    let mem = format!("{:>8} ", mem_text);
    let prefix_len = pid.len() + cpu.len() + mem.len();
    let cmd_width = row_width.saturating_sub(prefix_len as u16) as usize;

    let mut spans = vec![
        Span::styled(pid, Style::default().fg(colors.base)),
        Span::styled(cpu, Style::default().fg(colors.cpu)),
        Span::styled(mem, Style::default().fg(colors.mem)),
    ];
    spans.extend(command_spans(proc, full_command, cmd_width, colors));

    process_line(spans, selected)
}
