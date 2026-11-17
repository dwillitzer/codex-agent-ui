//! A bounded, persistent view of the current session's child agents.
//!
//! Explicit focus separates the selected row from the transcript being viewed. Outside focus,
//! editor keys remain with the composer. Selection follows stable thread IDs through updates.
//! Unknown context and idle threads are deliberately distinct from zero usage and completion.

use crate::render::renderable::Renderable;
use crate::text_formatting::truncate_text;
use codex_protocol::ThreadId;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Stylize;
use ratatui::text::Line;
use ratatui::widgets::Paragraph;
use std::time::Duration;
use std::time::Instant;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct SubagentPanelRow {
    pub id: ThreadId,
    pub parent: bool,
    pub input_target: String,
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
    selected: Option<ThreadId>,
}

impl SubagentPanel {
    pub(crate) fn set(&mut self, rows: Vec<SubagentPanelRow>, collapsed: bool) -> bool {
        if self.rows == rows && self.collapsed == collapsed {
            return false;
        }
        self.rows = rows;
        self.collapsed = collapsed;
        if self.is_empty() {
            self.selected = None;
        }
        if self
            .selected
            .is_some_and(|id| !self.rows.iter().any(|row| row.id == id))
        {
            self.selected = self.rows.iter().find(|row| row.current).map(|row| row.id);
        }
        true
    }

    pub(crate) fn focus(&mut self) -> bool {
        if self.is_empty() {
            return false;
        }
        self.selected = self
            .rows
            .iter()
            .find(|row| row.current)
            .or_else(|| self.rows.first())
            .map(|row| row.id);
        true
    }

    pub(crate) fn focused(&self) -> bool {
        self.selected.is_some()
    }

    pub(crate) fn blur(&mut self) {
        self.selected = None;
    }

    pub(crate) fn selected(&self) -> Option<ThreadId> {
        self.selected
    }

    pub(crate) fn move_selection(&mut self, forward: bool) {
        let Some(index) = self
            .rows
            .iter()
            .position(|row| Some(row.id) == self.selected)
        else {
            return;
        };
        let len = self.rows.len();
        self.selected = Some(
            self.rows[if forward {
                (index + 1) % len
            } else {
                (index + len - 1) % len
            }]
            .id,
        );
    }

    pub(crate) fn is_empty(&self) -> bool {
        !self.rows.iter().any(|row| !row.parent)
    }

    #[cfg(test)]
    fn has_active_agents(&self) -> bool {
        self.rows.iter().any(|row| row.active)
    }

    pub(crate) fn needs_clock_refresh(&self) -> bool {
        (!self.collapsed || self.focused())
            && self
                .rows
                .iter()
                .any(|row| row.active && row.started.is_some())
    }

    fn lines(&self, width: u16) -> Vec<Line<'static>> {
        if self.is_empty() || width < 12 {
            return vec![];
        }
        let active = self
            .rows
            .iter()
            .filter(|row| !row.parent && row.active)
            .count();
        let children = self.rows.iter().filter(|row| !row.parent).count();
        let summary = format!(
            "Agents: {active} active / {} · F6 select · /subagents {}",
            children,
            if self.collapsed { "expand" } else { "compact" },
        );
        let mut lines = vec![Line::from(truncate_text(&summary, width as usize)).dim()];
        if let Some(viewed) = self.rows.iter().find(|row| row.current && !row.parent) {
            lines.push(
                Line::from(truncate_text(
                    &format!("Viewing {} · Input: {}", viewed.label, viewed.input_target),
                    width as usize,
                ))
                .cyan(),
            );
        }
        if self.focused() {
            lines.push(
                Line::from(truncate_text(
                    "Select: ↑/↓ · Enter view · a stop turn · Esc return to input",
                    width as usize,
                ))
                .cyan(),
            );
        }
        if self.collapsed && !self.focused() {
            return lines;
        }
        // Keep active agents visible even when earlier children have finished. The picker keeps
        // every row in stable spawn order; this preview has a fixed height budget.
        let mut rows: Vec<_> = self
            .rows
            .iter()
            .filter(|row| self.focused() || !row.parent)
            .collect();
        let start = if self.focused() {
            rows.iter()
                .position(|row| Some(row.id) == self.selected)
                .unwrap_or(0)
                .saturating_sub(2)
        } else {
            rows.sort_by_key(|row| (!row.current, !row.active));
            0
        };
        for row in rows.iter().skip(start).take(3) {
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
                "{}{} {} · {}{}",
                if Some(row.id) == self.selected {
                    ">"
                } else {
                    " "
                },
                if row.current { "●" } else { "·" },
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
        if rows.len() > 3 {
            lines.push(Line::from(format!("  {} more · /subagents for all", rows.len() - 3)).dim());
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
            id: ThreadId::new(),
            parent: false,
            input_target: "agent".into(),
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
        Agents: 1 active / 2 · F6 select · /subagents compact
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
        assert!(text.contains("● current-agent"));
        assert!(text.contains("running-agent"));
        assert!(text.contains("19 more · /subagents"));
        assert_eq!(panel.desired_height(100), 9);
        // Narrow layouts use one line per child; no task wrapping or unbounded height.
        assert_eq!(panel.desired_height(20), 6);
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

    #[test]
    fn focus_tracks_identity_across_completion_updates_and_windows_selection() {
        let mut rows: Vec<_> = (0..8).map(|i| row(&format!("agent-{i}"), true)).collect();
        rows[0].parent = true;
        rows[0].label = "Main".into();
        rows[0].current = true;
        let mut panel = SubagentPanel::default();
        panel.set(rows.clone(), true);
        assert!(panel.focus());
        for _ in 0..7 {
            panel.move_selection(true);
        }
        let selected = panel.selected();
        rows[7].active = false;
        rows[7].state = "done".into();
        panel.set(rows.clone(), true);
        assert_eq!(panel.selected(), selected);
        let text = render(&panel, 100);
        assert!(text.contains(">· agent-7 · done"), "{text}");
        assert!(!text.contains("agent-1"));
        panel.move_selection(true);
        assert_eq!(panel.selected(), Some(rows[0].id));
        panel.move_selection(false);
        assert_eq!(panel.selected(), selected);
        rows.pop();
        panel.set(rows, true);
        assert!(render(&panel, 100).contains(">● Main"));
        panel.blur();
        assert!(!panel.focused());
        assert_eq!(panel.desired_height(100), 1);
    }
}
