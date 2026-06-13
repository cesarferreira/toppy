use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::{
    app::App,
    metrics::format,
    theme::{self, TREE_BORDER, utilization_color},
    ui::text::truncate_to_width,
};

pub fn render_tree(frame: &mut Frame, area: Rect, app: &App) {
    let block = Block::default()
        .title(" Process Tree ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(TREE_BORDER));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let visible = inner.height as usize;
    let flat = app.tree.flatten_visible(&app.processes, &app.tree_expanded);

    let mut lines = Vec::new();
    for (row_idx, (pid, depth, has_children)) in flat
        .into_iter()
        .skip(app.scroll_offset)
        .take(visible)
        .enumerate()
    {
        let Some(node) = app.tree.by_pid.get(&pid) else {
            continue;
        };
        let proc = &app.processes[node.index];
        let selected = row_idx + app.scroll_offset == app.selected;
        lines.push(render_tree_row(
            proc,
            depth,
            has_children,
            app.tree_expanded.contains(&pid),
            selected,
            inner.width,
        ));
    }

    if lines.is_empty() {
        lines.push(Line::from(Span::styled(
            "  no processes",
            theme::dim_style(),
        )));
    }

    frame.render_widget(Paragraph::new(lines), inner);
}

fn render_tree_row(
    proc: &crate::process::model::ProcessRow,
    depth: usize,
    has_children: bool,
    expanded: bool,
    selected: bool,
    row_width: u16,
) -> Line<'static> {
    let indent = "  ".repeat(depth);
    let branch = if has_children {
        if expanded { "▼ " } else { "▶ " }
    } else {
        "  "
    };
    let cpu_color = utilization_color(proc.cpu);
    let mem_text = format::format_bytes(proc.mem_bytes);
    let head = format!("{indent}{branch}{:>7} ", proc.pid);
    let cpu = format!("{:>5.1}% ", proc.cpu);
    let mem = format!("{:>8} ", mem_text);
    let prefix_len = head.chars().count() + cpu.chars().count() + mem.chars().count();
    let cmd_width = row_width.saturating_sub(prefix_len as u16) as usize;
    let cmd = truncate_to_width(&proc.cmd, cmd_width.max(1));

    let mut spans = vec![
        Span::raw(head),
        Span::styled(cpu, Style::default().fg(cpu_color)),
        Span::raw(mem),
        Span::raw(cmd),
    ];

    if selected {
        for span in &mut spans {
            span.style = span.style.bg(theme::SELECTED_BG).add_modifier(Modifier::BOLD);
        }
    }

    Line::from(spans)
}
