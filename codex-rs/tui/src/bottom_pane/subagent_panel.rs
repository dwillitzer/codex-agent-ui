//! A bounded, persistent view of the current session's child agents.
//!
//! Selection uses the existing `/subagents` picker so this view never steals editor keys.
//! Unknown context and idle threads are deliberately distinct from zero usage and completion.

use crate::render::renderable::Renderable;
use crate::text_formatting::truncate_text;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Stylize;
use ratatui::text::Line;
use ratatui::widgets::Paragraph;
use std::time::Duration;
use std::time::Instant;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct SubagentPanelRow {
    pub label: String,
    pub state: String,
    pub model_context: String,
    pub task: String,
    pub active: bool,
    pub current: bool,
    pub started: Option<Instant>,
    pub elapsed: Option<Duration>,
}

#[derive(Default)]
pub(crate) struct SubagentPanel {
    rows: Vec<SubagentPanelRow>,
    collapsed: bool,
}

impl SubagentPanel {
    pub(crate) fn set(&mut self, rows: Vec<SubagentPanelRow>, collapsed: bool) -> bool {
        if self.rows == rows && self.collapsed == collapsed {
            return false;
        }
        self.rows = rows;
        self.collapsed = collapsed;
        true
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    #[cfg(test)]
    fn has_active_agents(&self) -> bool {
        self.rows.iter().any(|row| row.active)
    }

    pub(crate) fn needs_clock_refresh(&self) -> bool {
        !self.collapsed
            && self
                .rows
                .iter()
                .any(|row| row.active && row.started.is_some())
    }

    fn lines(&self, width: u16) -> Vec<Line<'static>> {
        if self.rows.is_empty() || width < 12 {
            return vec![];
        }
        let active = self.rows.iter().filter(|row| row.active).count();
        let summary = format!(
            "Agents: {active} active / {} · /subagents to inspect · /subagents {}",
            self.rows.len(),
            if self.collapsed { "expand" } else { "compact" },
        );
        let mut lines = vec![Line::from(truncate_text(&summary, width as usize)).dim()];
        if self.collapsed {
            return lines;
        }
        // Keep active agents visible even when earlier children have finished. The picker keeps
        // every row in stable spawn order; this preview has a fixed height budget.
        let mut rows: Vec<_> = self.rows.iter().collect();
        rows.sort_by_key(|row| (!row.current, !row.active));
        for row in rows.iter().take(3) {
            let elapsed = row.elapsed.or_else(|| {
                row.started
                    .filter(|_| row.active)
                    .map(|start| start.elapsed())
            });
            let elapsed = elapsed
                .map(|elapsed| {
                    let secs = elapsed.as_secs();
                    format!(" · {}m{:02}s", secs / 60, secs % 60)
                })
                .unwrap_or_default();
            let text = format!(
                "{} {} · {}{}",
                if row.current { ">" } else { "·" },
                row.label,
                row.state,
                elapsed,
            );
            lines.push(Line::from(truncate_text(&text, width as usize)));
            if width >= 40 {
                let details = if row.task.is_empty() || width < 100 {
                    format!("  {}", row.model_context)
                } else {
                    format!("  {} · {}", row.model_context, row.task)
                };
                lines.push(Line::from(truncate_text(&details, width as usize)).dim());
            }
        }
        if self.rows.len() > 3 {
            lines.push(Line::from(format!("  +{} more in /subagents", self.rows.len() - 3)).dim());
        }
        lines
    }
}

impl Renderable for SubagentPanel {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        ratatui::widgets::Widget::render(Paragraph::new(self.lines(area.width)), area, buf);
    }

    fn desired_height(&self, width: u16) -> u16 {
        self.lines(width).len() as u16
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    fn row(name: &str, active: bool) -> SubagentPanelRow {
        SubagentPanelRow {
            label: name.into(),
            state: if active { "running" } else { "done" }.into(),
            model_context: "gpt-child · ctx 50% used (60K/108K)".into(),
            task: "Review source and report findings".into(),
            active,
            current: false,
            started: None,
            elapsed: Some(Duration::from_secs(74)),
        }
    }

    fn render(panel: &SubagentPanel, width: u16) -> String {
        let height = panel.desired_height(width);
        let mut buffer = Buffer::empty(Rect::new(0, 0, width, height));
        panel.render(buffer.area, &mut buffer);
        (0..height)
            .map(|y| {
                (0..width)
                    .map(|x| buffer[(x, y)].symbol())
                    .collect::<String>()
                    .trim_end()
                    .to_string()
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    #[test]
    fn empty_panel_uses_no_rows() {
        assert_eq!(SubagentPanel::default().desired_height(100), 0);
    }

    #[test]
    fn persistent_panel_snapshot() {
        let mut panel = SubagentPanel::default();
        panel.set(
            vec![row("/root/source-review", true), row("/root/tests", false)],
            false,
        );
        insta::assert_snapshot!(render(&panel, 110), @"
        Agents: 1 active / 2 · /subagents to inspect · /subagents compact
        · /root/source-review · running · 1m14s
          gpt-child · ctx 50% used (60K/108K) · Review source and report findings
        · /root/tests · done · 1m14s
          gpt-child · ctx 50% used (60K/108K) · Review source and report findings
        ");
        assert!(panel.has_active_agents());
    }

    #[test]
    fn preview_is_bounded_and_prioritizes_current_and_active_agents() {
        let mut rows: Vec<_> = (0..20).map(|i| row(&format!("done-{i}"), false)).collect();
        rows.push(row("running-agent", true));
        let mut current = row("current-agent", false);
        current.current = true;
        rows.push(current);
        let mut panel = SubagentPanel::default();
        panel.set(rows, false);
        let text = render(&panel, 100);
        assert!(text.contains("> current-agent"));
        assert!(text.contains("running-agent"));
        assert!(text.contains("+19 more in /subagents"));
        assert_eq!(panel.desired_height(100), 8);
        // Narrow layouts use one line per child; no task wrapping or unbounded height.
        assert_eq!(panel.desired_height(20), 5);
        assert_eq!(panel.desired_height(0), 0);
    }

    #[test]
    fn compact_keeps_completion_visible_and_expands_without_losing_rows() {
        let mut panel = SubagentPanel::default();
        let rows = vec![row("finished-agent", false)];
        panel.set(rows.clone(), true);
        assert_eq!(panel.desired_height(100), 1);
        assert!(render(&panel, 100).contains("Agents: 0 active / 1"));
        assert!(!panel.has_active_agents());
        panel.set(rows, false);
        assert!(render(&panel, 100).contains("finished-agent"));
    }

    #[test]
    fn rendering_clips_to_the_supplied_viewport() {
        let mut panel = SubagentPanel::default();
        panel.set(vec![row("agent", true)], false);
        let mut buffer = Buffer::empty(Rect::new(0, 0, 100, 8));
        for cell in buffer.content.iter_mut() {
            cell.set_symbol("x");
        }
        panel.render(Rect::new(5, 2, 30, 2), &mut buffer);
        assert_eq!(buffer[(0, 0)].symbol(), "x");
        assert_eq!(buffer[(35, 2)].symbol(), "x");
        assert_eq!(buffer[(5, 4)].symbol(), "x");
    }
}
