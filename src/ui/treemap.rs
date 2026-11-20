use ratatui::{
    Frame,
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    widgets::{Block, Borders},
};

use crate::{
    app::App,
    metrics::format,
    process::model::{ProcessRow, SortColumn},
    theme,
};

/// Smallest tile that still shows a readable name.
const MIN_TILE_WIDTH: u16 = 6;
const MIN_TILE_HEIGHT: u16 = 2;
/// Terminal cells are about twice as tall as they are wide. Layout runs in a
/// space with doubled height so tiles look square on screen.
const CELL_ASPECT: f64 = 2.0;
/// CPU change, in percentage points, below which a process counts as steady.
pub const TREND_DEADBAND: f32 = 1.5;

/// What decides each tile's area.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SizeMetric {
    Cpu,
    Mem,
}

impl SizeMetric {
    pub fn for_sort(sort: SortColumn) -> Self {
        if sort == SortColumn::Mem {
            Self::Mem
        } else {
            Self::Cpu
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Cpu => "CPU",
            Self::Mem => "MEM",
        }
    }

    fn value(self, process: &ProcessRow) -> f64 {
        match self {
            Self::Cpu => f64::from(process.cpu.max(0.0)),
            Self::Mem => process.mem_bytes as f64,
        }
    }
}

/// Direction of a process's CPU since the previous refresh.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Trend {
    Up(f32),
    Down(f32),
    Flat(f32),
}

impl Trend {
    pub fn from_delta(delta: f32) -> Self {
        if delta >= TREND_DEADBAND {
            Self::Up(delta)
        } else if delta <= -TREND_DEADBAND {
            Self::Down(delta)
        } else {
            Self::Flat(delta)
        }
    }

    fn glyph(self) -> &'static str {
        match self {
            Self::Up(_) => "▲",
            Self::Down(_) => "▼",
            Self::Flat(_) => "■",
        }
    }

    fn delta(self) -> f32 {
        match self {
            Self::Up(d) | Self::Down(d) | Self::Flat(d) => d,
        }
    }
}

/// Render the treemap and return each tile's list position and screen area,
/// so arrow keys can move between neighboring tiles.
pub fn render_treemap(frame: &mut Frame, area: Rect, app: &App) -> Vec<(usize, Rect)> {
    let metric = SizeMetric::for_sort(app.sort);
    let filter = if app.filter.is_empty() {
        String::new()
    } else {
        format!("  filter: {}", app.filter)
    };
    let block = Block::default()
        .title(format!(
            " Treemap  size: {}  color: CPU trend{filter} ",
            metric.label()
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme::TREEMAP_BORDER));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    // One column of left padding mirrors the gap each tile leaves on its right.
    let canvas = Rect {
        x: inner.x.saturating_add(1).min(inner.right()),
        width: inner.width.saturating_sub(1),
        ..inner
    };
    if canvas.width < MIN_TILE_WIDTH || canvas.height < MIN_TILE_HEIGHT {
        return Vec::new();
    }
    if app.filtered_indices.is_empty() {
        frame.buffer_mut().set_string(
            canvas.x,
            canvas.y,
            "no matching processes",
            theme::dim_style(),
        );
        return Vec::new();
    }

    let tiles = plan_tiles(app, metric, canvas);
    let buffer = frame.buffer_mut();
    for &(position, rect) in &tiles {
        let process = &app.processes[app.filtered_indices[position]];
        draw_tile(buffer, rect, process, app, position == app.selected);
    }
    tiles
}

/// Choose which processes get tiles and where they go. The selected process
/// always gets a tile, even when it ranks below the ones that fit.
fn plan_tiles(app: &App, metric: SizeMetric, canvas: Rect) -> Vec<(usize, Rect)> {
    let len = app.filtered_indices.len();
    let max_tiles = (usize::from(canvas.width) * usize::from(canvas.height)
        / (usize::from(MIN_TILE_WIDTH) * 4))
        .clamp(1, len);
    let selected = app.selected.min(len - 1);

    let mut count = max_tiles;
    loop {
        let mut positions: Vec<usize> = (0..count).collect();
        if selected >= count {
            positions.push(selected);
        }
        let values: Vec<f64> = positions
            .iter()
            .map(|&p| metric.value(&app.processes[app.filtered_indices[p]]))
            .collect();
        let rects = layout(&weights(&values), canvas);
        let too_small = rects
            .iter()
            .position(|r| r.width < MIN_TILE_WIDTH || r.height < MIN_TILE_HEIGHT);
        match too_small {
            Some(index) if positions.len() > 1 => {
                // Drop everything from the first unreadable tile on and retry.
                // When only the pinned selected tile is too small, make room
                // for it by dropping one ranked tile.
                count = if index < count { index } else { count - 1 };
            }
            _ => return positions.into_iter().zip(rects).collect(),
        }
    }
}

/// Raise tiny values so idle processes still get a visible tile.
fn weights(values: &[f64]) -> Vec<f64> {
    let total: f64 = values.iter().sum();
    let floor = if total > 0.0 { total * 0.01 } else { 1.0 };
    values.iter().map(|&v| v.max(floor)).collect()
}

#[derive(Debug, Clone, Copy)]
struct FRect {
    x: f64,
    y: f64,
    w: f64,
    h: f64,
}

/// Squarified treemap of `weights` (kept in input order) inside `area`.
pub fn layout(weights: &[f64], area: Rect) -> Vec<Rect> {
    let bounds = FRect {
        x: f64::from(area.x),
        y: f64::from(area.y) * CELL_ASPECT,
        w: f64::from(area.width),
        h: f64::from(area.height) * CELL_ASPECT,
    };
    let total: f64 = weights.iter().sum();
    if weights.is_empty() || total <= 0.0 {
        return Vec::new();
    }
    let scale = bounds.w * bounds.h / total;
    let areas: Vec<f64> = weights.iter().map(|w| w * scale).collect();

    squarify(&areas, bounds)
        .into_iter()
        .map(|r| {
            let x0 = r.x.round() as u16;
            let x1 = (r.x + r.w).round() as u16;
            let y0 = (r.y / CELL_ASPECT).round() as u16;
            let y1 = ((r.y + r.h) / CELL_ASPECT).round() as u16;
            Rect::new(x0, y0, x1.saturating_sub(x0), y1.saturating_sub(y0))
        })
        .collect()
}

fn squarify(areas: &[f64], mut rest: FRect) -> Vec<FRect> {
    let mut out = Vec::with_capacity(areas.len());
    let mut start = 0;
    while start < areas.len() {
        let side = rest.w.min(rest.h);
        let mut end = start + 1;
        let mut best = worst_ratio(&areas[start..end], side);
        while end < areas.len() {
            let next = worst_ratio(&areas[start..=end], side);
            if next > best {
                break;
            }
            best = next;
            end += 1;
        }

        let row = &areas[start..end];
        let sum: f64 = row.iter().sum();
        let last = row.len() - 1;
        if rest.w >= rest.h {
            // Lay the row as a column on the left.
            let width = if end == areas.len() {
                rest.w
            } else {
                sum / rest.h
            };
            let mut y = rest.y;
            for (i, &a) in row.iter().enumerate() {
                let h = if i == last {
                    rest.y + rest.h - y
                } else {
                    a / width
                };
                out.push(FRect {
                    x: rest.x,
                    y,
                    w: width,
                    h,
                });
                y += h;
            }
            rest.x += width;
            rest.w -= width;
        } else {
            // Lay the row along the top.
            let height = if end == areas.len() {
                rest.h
            } else {
                sum / rest.w
            };
            let mut x = rest.x;
            for (i, &a) in row.iter().enumerate() {
                let w = if i == last {
                    rest.x + rest.w - x
                } else {
                    a / height
                };
                out.push(FRect {
                    x,
                    y: rest.y,
                    w,
                    h: height,
                });
                x += w;
            }
            rest.y += height;
            rest.h -= height;
        }
        start = end;
    }
    out
}

fn worst_ratio(row: &[f64], side: f64) -> f64 {
    let sum: f64 = row.iter().sum();
    if sum <= 0.0 || side <= 0.0 {
        return f64::INFINITY;
    }
    let max = row.iter().copied().fold(f64::MIN, f64::max);
    let min = row.iter().copied().fold(f64::MAX, f64::min);
    let side2 = side * side;
    let sum2 = sum * sum;
    (side2 * max / sum2).max(sum2 / (side2 * min.max(f64::EPSILON)))
}

fn draw_tile(buffer: &mut Buffer, rect: Rect, process: &ProcessRow, app: &App, selected: bool) {
    let rect = rect.intersection(buffer.area);
    // Leave a one-cell gap on the right and bottom so tiles read as separate.
    let body = Rect {
        width: if rect.width > 3 {
            rect.width - 1
        } else {
            rect.width
        },
        height: if rect.height > 2 {
            rect.height - 1
        } else {
            rect.height
        },
        ..rect
    };
    if body.is_empty() {
        return;
    }

    let trend = Trend::from_delta(app.cpu_delta(process));
    let bg = theme::treemap_tile_bg(trend.delta());
    buffer.set_style(body, Style::default().bg(bg));

    // Padding inside the tile.
    let content = Rect {
        x: body.x + 1,
        y: body.y + u16::from(body.height >= 5),
        width: body.width.saturating_sub(2),
        height: body.height - u16::from(body.height >= 5),
    };
    if content.width == 0 || content.height == 0 {
        return;
    }
    let width = usize::from(content.width);

    let value_style = Style::default()
        .fg(theme::SELECTED_FG)
        .bg(bg)
        .add_modifier(Modifier::BOLD);
    let meta_style = Style::default().fg(theme::TREEMAP_META).bg(bg);
    // The selected tile keeps its trend color and inverts its title bar.
    let title_style = if selected {
        Style::default()
            .fg(bg)
            .bg(theme::SELECTED_FG)
            .add_modifier(Modifier::BOLD)
    } else {
        value_style
    };

    let cpu = format!("{:.1}%", process.cpu);
    let trend_text = format!("{} {:+.1}", trend.glyph(), trend.delta());
    let footer: Vec<(String, String)> = match content.height {
        0 | 1 => Vec::new(),
        2 => vec![usage_line(&cpu, &trend_text, width)],
        _ => vec![
            (id_line(process, app.show_pid, width), String::new()),
            usage_line(&cpu, &trend_text, width),
        ],
    };
    let title_rows = usize::from(content.height) - footer.len();

    // Full command lines are unreadable in small tiles, so show the program name.
    let mut label = process.name.clone();
    if selected {
        label.insert_str(0, "▶ ");
    }
    let title = wrap(&label, width, title_rows);
    if selected {
        let bar_bottom = content.y + title.len().max(1) as u16;
        let bar = Rect {
            height: bar_bottom - body.y,
            ..body
        };
        buffer.set_style(bar, title_style);
    }
    for (row, line) in title.iter().enumerate() {
        buffer.set_stringn(content.x, content.y + row as u16, line, width, title_style);
    }

    let footer_top = content.bottom() - footer.len() as u16;
    for (i, (left, right)) in footer.iter().enumerate() {
        let y = footer_top + i as u16;
        buffer.set_stringn(content.x, y, left, width, meta_style);
        let right_len = right.chars().count();
        if !right.is_empty() {
            buffer.set_stringn(
                content.right() - right_len as u16,
                y,
                right,
                right_len,
                value_style,
            );
        }
    }
}

/// `#PID · memory`, dropping parts that do not fit in `width`.
fn id_line(process: &ProcessRow, show_pid: bool, width: usize) -> String {
    let mem = format::format_bytes(process.mem_bytes);
    let pid = format!("#{}", process.pid);
    let candidates = if show_pid {
        vec![format!("{pid} · {mem}"), pid, mem]
    } else {
        vec![mem]
    };
    first_fitting(candidates, width)
}

/// CPU usage on the left and the trend on the right, shortened to fit `width`.
fn usage_line(cpu: &str, trend: &str, width: usize) -> (String, String) {
    let long = format!("{cpu} CPU");
    for (left, right) in [
        (long.as_str(), trend),
        (cpu, trend),
        (long.as_str(), ""),
        (cpu, ""),
    ] {
        let gap = usize::from(!right.is_empty());
        if left.chars().count() + gap + right.chars().count() <= width {
            return (left.to_string(), right.to_string());
        }
    }
    (cpu.to_string(), String::new())
}

fn first_fitting(candidates: Vec<String>, width: usize) -> String {
    let fallback = candidates.last().cloned().unwrap_or_default();
    candidates
        .into_iter()
        .find(|text| text.chars().count() <= width)
        .unwrap_or(fallback)
}

/// Greedy word wrap into at most `rows` lines of `width` chars. The last line
/// ends with `…` when text is cut.
fn wrap(text: &str, width: usize, rows: usize) -> Vec<String> {
    if width == 0 || rows == 0 {
        return Vec::new();
    }
    let mut lines: Vec<String> = Vec::new();
    let mut current = String::new();
    let mut current_len = 0;
    for word in text.split_whitespace() {
        let mut word: Vec<char> = word.chars().collect();
        loop {
            let gap = usize::from(current_len > 0);
            if current_len + gap + word.len() <= width {
                if gap == 1 {
                    current.push(' ');
                }
                current.extend(word.iter());
                current_len += gap + word.len();
                break;
            }
            if current_len > 0 {
                lines.push(std::mem::take(&mut current));
                current_len = 0;
                continue;
            }
            // A single word longer than the line: hard split.
            let rest = word.split_off(width);
            lines.push(word.into_iter().collect());
            word = rest;
        }
    }
    if current_len > 0 {
        lines.push(current);
    }
    if lines.len() > rows {
        lines.truncate(rows);
        let last = &mut lines[rows - 1];
        let mut chars: Vec<char> = last.chars().collect();
        chars.truncate(width.saturating_sub(1));
        while chars.last() == Some(&' ') {
            chars.pop();
        }
        chars.push('…');
        *last = chars.into_iter().collect();
    }
    lines
}

/// Pick the nearest tile from `current` in direction (`dx`, `dy`).
pub fn neighbor(tiles: &[(usize, Rect)], current: usize, dx: i32, dy: i32) -> Option<usize> {
    let &(_, from) = tiles.iter().find(|(p, _)| *p == current)?;
    let center = |r: Rect| {
        (
            i32::from(r.x) * 2 + i32::from(r.width),
            i32::from(r.y) * 2 + i32::from(r.height),
        )
    };
    let (fx, fy) = center(from);
    tiles
        .iter()
        .filter(|(p, _)| *p != current)
        .filter_map(|&(position, r)| {
            // Distance along the move, then the offset across it.
            let (along, across) = match (dx, dy) {
                (1, _) if r.x >= from.right() => {
                    (i32::from(r.x - from.right()), (center(r).1 - fy).abs())
                }
                (-1, _) if r.right() <= from.x => {
                    (i32::from(from.x - r.right()), (center(r).1 - fy).abs())
                }
                (_, 1) if r.y >= from.bottom() => {
                    (i32::from(r.y - from.bottom()), (center(r).0 - fx).abs())
                }
                (_, -1) if r.bottom() <= from.y => {
                    (i32::from(from.y - r.bottom()), (center(r).0 - fx).abs())
                }
                _ => return None,
            };
            Some((along * 4 + across, position))
        })
        .min()
        .map(|(_, position)| position)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn covered(rects: &[Rect], area: Rect) -> usize {
        let mut seen = vec![false; usize::from(area.width) * usize::from(area.height)];
        for r in rects {
            for y in r.top()..r.bottom() {
                for x in r.left()..r.right() {
                    let i =
                        usize::from(y - area.y) * usize::from(area.width) + usize::from(x - area.x);
                    assert!(!seen[i], "overlap at ({x}, {y})");
                    seen[i] = true;
                }
            }
        }
        seen.iter().filter(|&&s| s).count()
    }

    #[test]
    fn layout_tiles_the_whole_area_without_overlap() {
        let area = Rect::new(3, 2, 97, 31);
        for weights in [
            vec![1.0],
            vec![50.0, 30.0, 10.0, 5.0, 5.0],
            vec![9.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0],
            vec![1.0, 2.0, 3.0, 40.0],
        ] {
            let rects = layout(&weights, area);
            assert_eq!(rects.len(), weights.len());
            assert_eq!(
                covered(&rects, area),
                usize::from(area.width) * usize::from(area.height),
                "{weights:?}"
            );
        }
    }

    #[test]
    fn larger_weights_get_larger_tiles() {
        let rects = layout(&[60.0, 30.0, 10.0], Rect::new(0, 0, 80, 20));
        let areas: Vec<u32> = rects.iter().map(|r| r.area()).collect();
        assert!(areas[0] > areas[1] && areas[1] > areas[2], "{areas:?}");
    }

    #[test]
    fn idle_processes_get_a_floor_weight() {
        assert_eq!(weights(&[0.0, 0.0]), vec![1.0, 1.0]);
        assert_eq!(weights(&[100.0, 0.0]), vec![100.0, 1.0]);
    }

    #[test]
    fn trend_ignores_small_changes() {
        assert!(matches!(Trend::from_delta(1.0), Trend::Flat(_)));
        assert!(matches!(Trend::from_delta(-1.0), Trend::Flat(_)));
        assert!(matches!(Trend::from_delta(3.0), Trend::Up(_)));
        assert!(matches!(Trend::from_delta(-3.0), Trend::Down(_)));
    }

    #[test]
    fn wrap_breaks_words_and_marks_cut_text() {
        assert_eq!(
            wrap("Google Chrome Helper", 10, 3),
            ["Google", "Chrome", "Helper"]
        );
        assert_eq!(wrap("Google Chrome Helper", 10, 2), ["Google", "Chrome…"]);
        assert_eq!(wrap("abcdefghij", 4, 3), ["abcd", "efgh", "ij"]);
        assert_eq!(wrap("anything", 0, 3), Vec::<String>::new());
    }

    #[test]
    fn footer_text_shrinks_to_fit() {
        assert_eq!(
            usage_line("2.0%", "▲ +3.0", 20),
            ("2.0% CPU".into(), "▲ +3.0".into())
        );
        assert_eq!(
            usage_line("2.0%", "▲ +3.0", 12),
            ("2.0%".into(), "▲ +3.0".into())
        );
        assert_eq!(
            usage_line("2.0%", "▲ +3.0", 8),
            ("2.0% CPU".into(), String::new())
        );
        assert_eq!(
            usage_line("2.0%", "▲ +3.0", 5),
            ("2.0%".into(), String::new())
        );
        let mut process = crate::app::tests::row(2117831, 0.0, None);
        process.mem_bytes = 6 * 1024 * 1024;
        assert_eq!(id_line(&process, true, 30), "#2117831 · 6.0 MB");
        assert_eq!(id_line(&process, true, 10), "#2117831");
        assert_eq!(id_line(&process, false, 10), "6.0 MB");
    }

    #[test]
    fn neighbor_moves_toward_the_adjacent_tile() {
        // 0 | 1
        // --+--
        // 2 | 3
        let tiles = [
            (0, Rect::new(0, 0, 10, 5)),
            (1, Rect::new(10, 0, 10, 5)),
            (2, Rect::new(0, 5, 10, 5)),
            (3, Rect::new(10, 5, 10, 5)),
        ];
        assert_eq!(neighbor(&tiles, 0, 1, 0), Some(1));
        assert_eq!(neighbor(&tiles, 0, 0, 1), Some(2));
        assert_eq!(neighbor(&tiles, 3, -1, 0), Some(2));
        assert_eq!(neighbor(&tiles, 3, 0, -1), Some(1));
        assert_eq!(neighbor(&tiles, 0, -1, 0), None);
    }
}
