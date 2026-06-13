use ratatui::{
    layout::Rect,
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::{app::App, theme};

pub fn render_status(frame: &mut Frame, area: Rect, app: &App) {
    let mut hint = String::from("F1/? help  +/- refresh  P/C/M/T sort  / filter  t tree  k kill  r refresh  q quit");

    if let Some(msg) = &app.status_message {
        hint = format!("{msg}  |  {hint}");
    }

    if let Some(proc) = app.selected_process() {
        hint = format!("{}  |  cmd: {}", hint, proc.cmd);
    }

    let cpu = if app.warmed_up {
        format!("{:.1}%", app.cpu.global)
    } else {
        "…".to_string()
    };

    let line = Line::from(vec![
        Span::styled(format!("CPU {cpu}  refresh {}ms  ", app.refresh_ms), theme::dim_style()),
        Span::raw(hint),
    ]);

    frame.render_widget(Paragraph::new(line), area);
}
