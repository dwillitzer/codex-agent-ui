//! A bounded, persistent view of the current session's child agents.
//!
//! Explicit focus separates the selected row from the transcript being viewed. Outside focus,
//! editor keys remain with the composer. Selection follows stable thread IDs through updates.
//! Unknown context and idle threads are deliberately distinct from zero usage and completion.
//! Focused `f` cycles a session-local status filter. Main and pending-approval awareness remain
//! available even when no child matches; filtering never changes the viewed transcript.

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

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum AgentStatusFilter {
    #[default]
    All,
    Active,
    Done,
    Failed,
}

impl AgentStatusFilter {
    const ALL: [Self; 4] = [Self::All, Self::Active, Self::Done, Self::Failed];

    pub(crate) fn next(self) -> Self {
        match self {
            Self::All => Self::Active,
            Self::Active => Self::Done,
            Self::Done => Self::Failed,
            Self::Failed => Self::All,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::All => "All",
            Self::Active => "Active",
            Self::Done => "Done",
            Self::Failed => "Failed",
        }
    }
}

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
    pub approval_pending: bool,
    pub current: bool,
    pub started: Option<Instant>,
    pub elapsed: Option<Duration>,
}

impl SubagentPanelRow {
    fn matches(&self, filter: AgentStatusFilter) -> bool {
        if self.parent || filter == AgentStatusFilter::All {
            return true;
        }
        let category = if self.approval_pending {
            Some(AgentStatusFilter::Active)
        } else {
            match self.state.as_str() {
                "done" => Some(AgentStatusFilter::Done),
                "error" => Some(AgentStatusFilter::Failed),
                _ if self.active => Some(AgentStatusFilter::Active),
                _ => None,
            }
        };
        category == Some(filter)
    }
}

#[derive(Default)]
pub(crate) struct SubagentPanel {
    rows: Vec<SubagentPanelRow>,
    collapsed: bool,
    selected: Option<ThreadId>,
    focused: bool,
    filter: AgentStatusFilter,
}

impl SubagentPanel {
    pub(crate) fn set(&mut self, rows: Vec<SubagentPanelRow>, collapsed: bool) -> bool {
        if self.rows == rows && self.collapsed == collapsed {
            return false;
        }
        let old_index = self
            .selected
            .and_then(|id| self.rows.iter().position(|row| row.id == id));
        let old_root = self.rows.iter().find(|row| row.parent).map(|row| row.id);
        let new_root = rows.iter().find(|row| row.parent).map(|row| row.id);
        if old_root.is_some() && old_root != new_root {
            self.blur();
            self.filter = AgentStatusFilter::All;
        }
        self.rows = rows;
        self.collapsed = collapsed;
        if self.is_empty() {
            self.blur();
        }
        self.reconcile_selection(old_index);
        true
    }

    fn reconcile_selection(&mut self, old_index: Option<usize>) {
        if !self.focused {
            self.selected = None;
            return;
        }
        if self
            .rows
            .iter()
            .any(|row| Some(row.id) == self.selected && row.matches(self.filter))
        {
            return;
        }
        let index = old_index.unwrap_or(0);
        self.selected = self
            .rows
            .iter()
            .skip(index)
            .chain(self.rows.iter().take(index))
            .find(|row| row.matches(self.filter))
            .map(|row| row.id);
    }

    pub(crate) fn set_filter(&mut self, filter: AgentStatusFilter) -> bool {
        if self.filter == filter {
            return false;
        }
        let old_index = self
            .selected
            .and_then(|id| self.rows.iter().position(|row| row.id == id));
        self.filter = filter;
        self.reconcile_selection(old_index);
        true
    }

    pub(crate) fn filter(&self) -> AgentStatusFilter {
        self.filter
    }

    pub(crate) fn parent_id(&self) -> Option<ThreadId> {
        self.rows.iter().find(|row| row.parent).map(|row| row.id)
    }

    fn count(&self, filter: AgentStatusFilter) -> usize {
        self.rows
            .iter()
            .filter(|row| !row.parent && row.matches(filter))
            .count()
    }

    pub(crate) fn focus(&mut self) -> bool {
        if self.is_empty() {
            return false;
        }
        self.focused = true;
        self.selected = self
            .rows
            .iter()
            .find(|row| row.current && row.matches(self.filter))
            .map(|row| row.id);
        self.reconcile_selection(None);
        true
    }

    pub(crate) fn focused(&self) -> bool {
        self.focused
    }

    pub(crate) fn blur(&mut self) {
        self.selected = None;
        self.focused = false;
    }

    pub(crate) fn selected(&self) -> Option<ThreadId> {
        self.selected
    }

    pub(crate) fn move_selection(&mut self, forward: bool) {
        let rows: Vec<_> = self
            .rows
            .iter()
            .filter(|row| row.matches(self.filter))
            .collect();
        let Some(index) = rows.iter().position(|row| Some(row.id) == self.selected) else {
            return;
        };
        let len = rows.len();
        self.selected = Some(
            rows[if forward {
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
                .any(|row| row.matches(self.filter) && row.active && row.started.is_some())
    }

    fn lines(&self, width: u16) -> Vec<Line<'static>> {
        if self.is_empty() || width < 12 {
            return vec![];
        }
        let active = self.count(AgentStatusFilter::Active);
        let children = self.count(AgentStatusFilter::All);
        let matching = self.count(self.filter);
        let approvals = self
            .rows
            .iter()
            .filter(|row| !row.parent && row.approval_pending)
            .count();
        let approval_notice = if approvals == 0 {
            String::new()
        } else {
            format!("{approvals} approval pending · ")
        };
        let compact_filter = if self.collapsed {
            format!(" · {} {matching}/{children}", self.filter.label())
        } else {
            String::new()
        };
        let summary = format!(
            "{approval_notice}Agents: {active} active / {}{compact_filter} · Alt+←/→ switch (empty prompt) · F6 list{}",
            children,
            if self.collapsed {
                " · /subagents expand"
            } else {
                ""
            },
        );
        let mut lines = vec![Line::from(truncate_text(&summary, width as usize)).dim()];
        if !self.collapsed || self.focused() {
            let tabs = AgentStatusFilter::ALL
                .iter()
                .map(|filter| {
                    let label = format!("{} {}", filter.label(), self.count(*filter));
                    if *filter == self.filter {
                        format!("[{label}]")
                    } else {
                        label
                    }
                })
                .collect::<Vec<_>>()
                .join(" · ");
            let tabs = if width < 40 {
                format!("{} {matching}/{children} · f filter", self.filter.label())
            } else {
                format!("{tabs} · f filter")
            };
            lines.push(Line::from(truncate_text(&tabs, width as usize)).cyan());
        }
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
                    "Select: ↑/↓ · Enter view · f filter · a stop turn · Esc return to input",
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
            .filter(|row| row.matches(self.filter))
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
            let mut text = format!(
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
            if !row.parent && width >= 40 && !row.task.is_empty() {
                text.push_str(" — ");
                text.push_str(&row.task);
            }
            lines.push(Line::from(truncate_text(&text, width as usize)));
            if width >= 40 {
                let details = format!("  {}", row.model_context);
                lines.push(Line::from(truncate_text(&details, width as usize)).dim());
            }
        }
        if matching == 0 {
            lines.push(
                Line::from(truncate_text(
                    &format!("No agents match {}.", self.filter.label()),
                    width as usize,
                ))
                .dim(),
            );
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
            approval_pending: false,
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
        Agents: 1 active / 2 · Alt+←/→ switch (empty prompt) · F6 list
        [All 2] · Active 1 · Done 1 · Failed 0 · f filter
         · /root/source-review · running · 1m14s — Review source and report findings
          gpt-child · ctx 50% used (60K/108K)
         · /root/tests · done · 1m14s — Review source and report findings
          gpt-child · ctx 50% used (60K/108K)
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
        assert_eq!(panel.desired_height(100), 10);
        // Narrow layouts use one line per child; no task wrapping or unbounded height.
        assert_eq!(panel.desired_height(20), 7);
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

    fn parent() -> SubagentPanelRow {
        let mut parent = row("Main", false);
        parent.parent = true;
        parent.current = true;
        parent
    }

    #[test]
    fn status_filters_use_explicit_outcomes_and_keep_approval_awareness() {
        let mut waiting = row("waiting-agent", true);
        waiting.state = "waiting".into();
        let mut approval = row("approval-agent", false);
        approval.state = "waiting for approval".into();
        approval.approval_pending = true;
        let mut failed = row("failed-agent", false);
        failed.state = "error".into();
        let mut interrupted = row("interrupted-agent", false);
        interrupted.state = "interrupted".into();
        let mut unknown = row("unknown-agent", false);
        unknown.state = "unknown (reconnecting)".into();
        let mut idle = row("idle-agent", false);
        idle.state = "idle".into();
        let mut panel = SubagentPanel::default();
        panel.set(
            vec![
                parent(),
                row("running-agent", true),
                waiting,
                approval,
                row("finished-agent", false),
                failed,
                interrupted,
                unknown,
                idle,
            ],
            false,
        );
        assert_eq!(panel.count(AgentStatusFilter::All), 8);
        assert_eq!(panel.count(AgentStatusFilter::Active), 3);
        assert_eq!(panel.count(AgentStatusFilter::Done), 1);
        assert_eq!(panel.count(AgentStatusFilter::Failed), 1);
        panel.focus();
        panel.set_filter(AgentStatusFilter::Active);
        let mut visited = Vec::new();
        for _ in 0..4 {
            panel.move_selection(true);
            visited.push(panel.selected().unwrap());
        }
        assert_eq!(
            visited,
            vec![
                panel.rows[1].id,
                panel.rows[2].id,
                panel.rows[3].id,
                panel.rows[0].id
            ]
        );
        panel.set_filter(AgentStatusFilter::Failed);
        let text = render(&panel, 120);
        assert!(text.contains("1 approval pending"), "{text}");
        assert!(text.contains("failed-agent · error"), "{text}");
        assert!(!text.contains("approval-agent"));
        assert!(!text.contains("interrupted-agent"));
        panel.blur();
        panel.set(panel.rows.clone(), true);
        assert_eq!(panel.desired_height(120), 1);
        assert!(render(&panel, 120).contains("Failed 1/8"));
        let narrow = render(&panel, 20);
        assert!(narrow.starts_with("1 approval"), "{narrow}");
    }

    #[test]
    fn filtered_selection_reconciles_completion_without_changing_view() {
        let mut rows = vec![
            parent(),
            row("first-agent", true),
            row("second-agent", true),
        ];
        let mut panel = SubagentPanel::default();
        panel.set(rows.clone(), false);
        panel.focus();
        panel.move_selection(true);
        panel.set_filter(AgentStatusFilter::Active);
        assert_eq!(panel.selected(), Some(rows[1].id));
        rows[1].active = false;
        rows[1].state = "done".into();
        panel.set(rows.clone(), false);
        assert_eq!(panel.selected(), Some(rows[2].id));
        assert_eq!(
            panel.rows.iter().find(|r| r.current).unwrap().id,
            rows[0].id
        );
        rows.pop();
        panel.set(rows.clone(), false);
        assert_eq!(panel.selected(), Some(rows[0].id));
        assert!(panel.focused());
        assert!(render(&panel, 100).contains("No agents match Active."));
        panel.set_filter(AgentStatusFilter::Done);
        panel.move_selection(true);
        assert_eq!(panel.selected(), Some(rows[1].id));
        let mut other_parent = parent();
        other_parent.label = "Other Main".into();
        panel.set(vec![other_parent, row("other-child", true)], false);
        assert_eq!(panel.filter(), AgentStatusFilter::All);
        assert!(!panel.focused());
    }

    #[test]
    fn empty_filtered_results_without_parent_do_not_lose_keyboard_focus() {
        let mut panel = SubagentPanel::default();
        panel.set(vec![row("running-agent", true)], false);
        panel.focus();
        panel.set_filter(AgentStatusFilter::Failed);
        assert!(panel.focused());
        assert_eq!(panel.selected(), None);
        assert!(render(&panel, 100).contains("No agents match Failed."));
        panel.set_filter(panel.filter().next());
        assert_eq!(panel.filter(), AgentStatusFilter::All);
        assert!(panel.selected().is_some());
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
