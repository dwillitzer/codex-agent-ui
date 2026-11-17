//! Parent-scoped agent presentation, fed by authoritative app-server notifications.
//!
//! This cache is independent of the displayed transcript. It never resumes a child, changes its
//! settings, or sends it input. Membership comes from spawn lineage, not from all loaded threads.

use super::App;
use super::app_server_event_targets::ServerNotificationThreadTarget;
use super::app_server_event_targets::server_notification_thread_target;
use crate::bottom_pane::subagent_panel::SubagentPanelRow;
use crate::status::format_tokens_compact;
use codex_app_server_protocol::ServerNotification;
use codex_app_server_protocol::Thread;
use codex_app_server_protocol::ThreadItemPayload;
use codex_app_server_protocol::ThreadStatus;
use codex_app_server_protocol::ThreadTokenUsage;
use codex_app_server_protocol::TurnStatus;
use codex_protocol::ThreadId;
use std::time::Duration;
use std::time::Instant;

fn assignment_line(text: &str) -> Option<String> {
    let text: String = text
        .chars()
        .filter(|c| !c.is_control() || c.is_whitespace())
        .collect();
    let text: String = text
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .take(80)
        .collect();
    (!text.is_empty()).then_some(text)
}

pub(super) struct AgentPanelBookmark(pub(crate) crate::transcript_view::TranscriptBookmark);

impl std::fmt::Debug for AgentPanelBookmark {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("AgentPanelBookmark")
    }
}

#[derive(Debug, Default)]
pub(super) struct AgentPanelDetails {
    pub root: Option<ThreadId>,
    pub parent: Option<ThreadId>,
    pub model: Option<String>,
    pub usage: Option<ThreadTokenUsage>,
    pub task: String,
    pub assignment: Option<String>,
    pub stale: bool,
    pub status: Option<String>,
    pub turn_id: Option<String>,
    pub started: Option<Instant>,
    pub elapsed: Option<Duration>,
}

impl AgentPanelDetails {
    pub(super) fn assignment_text(&self) -> &str {
        self.assignment
            .as_deref()
            .or_else(|| (!self.task.is_empty()).then_some(self.task.as_str()))
            .unwrap_or("Assignment unavailable")
    }

    fn set_model(&mut self, model: String) {
        if self.model.as_ref() != Some(&model) {
            self.usage = None;
            self.model = Some(model);
        }
    }

    pub(super) fn is_active(&self, running: bool, closed: bool) -> bool {
        running
            && !closed
            && !self.stale
            && !self.status.as_deref().is_some_and(|status| {
                matches!(
                    status,
                    "done" | "interrupted" | "error" | "closed" | "unavailable"
                )
            })
    }

    pub(super) fn display_state(&self, running: bool, closed: bool, approval: bool) -> &str {
        if approval {
            if self.stale {
                "waiting for approval (reconnecting)"
            } else {
                "waiting for approval"
            }
        } else if self.stale {
            "unknown (reconnecting)"
        } else if closed {
            "closed"
        } else if self.is_active(running, closed) {
            if self.status.as_deref() == Some("waiting") {
                "waiting"
            } else {
                "running"
            }
        } else if self
            .status
            .as_deref()
            .is_some_and(|status| matches!(status, "running" | "waiting"))
        {
            "idle"
        } else {
            self.status.as_deref().unwrap_or("unknown")
        }
    }

    pub(super) fn model_context(&self) -> String {
        let model = self
            .model
            .as_deref()
            .filter(|s| !s.is_empty())
            .unwrap_or("model unavailable");
        let context = self
            .usage
            .as_ref()
            .and_then(|usage| {
                let window = usage.model_context_window.filter(|window| *window > 0)?;
                // Context occupancy uses the latest response, never lifetime session usage. Use the
                // same baseline-adjusted percentage as the main Codex context indicator.
                let last = &usage.last;
                let tokens = codex_protocol::protocol::TokenUsage {
                    total_tokens: last.total_tokens.max(0),
                    input_tokens: last.input_tokens,
                    cached_input_tokens: last.cached_input_tokens,
                    cache_write_input_tokens: last.cache_write_input_tokens,
                    output_tokens: last.output_tokens,
                    reasoning_output_tokens: last.reasoning_output_tokens,
                    ..Default::default()
                };
                let used = 100 - tokens.percent_of_context_window_remaining(window);
                Some(format!(
                    "ctx {used}% used ({}/{})",
                    format_tokens_compact(last.total_tokens.max(0)),
                    format_tokens_compact(window)
                ))
            })
            .unwrap_or_else(|| "ctx unavailable".to_string());
        format!("{model} · {context}")
    }

    fn status_changed(&mut self, status: &ThreadStatus) {
        // A list/status response can race a terminal turn event. Releasing a viewed thread's
        // subscription changes residency, not its last turn outcome. Only an observed new turn
        // or a refresh after lost events may replace explicit completion with generic activity.
        if !self.stale
            && matches!(
                status,
                ThreadStatus::Active { .. } | ThreadStatus::Idle | ThreadStatus::NotLoaded
            )
            && self
                .status
                .as_deref()
                .is_some_and(|s| matches!(s, "done" | "interrupted" | "error"))
        {
            return;
        }
        self.stale = false;
        self.status = match status {
            ThreadStatus::Active { active_flags } if !active_flags.is_empty() => {
                Some("waiting".into())
            }
            ThreadStatus::Active { .. } => Some("running".into()),
            ThreadStatus::SystemError => Some("error".into()),
            ThreadStatus::NotLoaded => Some("unloaded".into()),
            // Idle is not proof of completion.
            ThreadStatus::Idle => Some("idle".into()),
        };
    }
}

impl App {
    pub(super) fn apply_agent_panel_filter(
        &mut self,
        parent: ThreadId,
        filter: crate::bottom_pane::subagent_panel::AgentStatusFilter,
    ) {
        if self.primary_thread_id == Some(parent) {
            self.agent_navigation.panel_filter = filter;
            self.sync_subagent_panel();
        }
    }

    /// Direct cycling follows the same parent-scoped, stable order as the visible panel.
    pub(super) fn panel_adjacent_thread_id(
        &self,
        direction: super::agent_navigation::AgentNavigationDirection,
    ) -> Option<ThreadId> {
        let root = self.primary_thread_id?;
        let current = self.current_displayed_thread_id()?;
        let ids: Vec<_> = self
            .agent_navigation
            .visible_threads()
            .into_iter()
            .map(|(id, _)| id)
            .filter(|id| {
                *id == root
                    || self
                        .agent_navigation
                        .panel_details
                        .get(id)
                        .is_some_and(|d| d.root == Some(root))
            })
            .collect();
        if ids.len() <= 1 {
            return None;
        }
        let Some(index) = ids.iter().position(|id| *id == current) else {
            return Some(root);
        };
        let index = match direction {
            super::agent_navigation::AgentNavigationDirection::Next => (index + 1) % ids.len(),
            super::agent_navigation::AgentNavigationDirection::Previous => {
                (index + ids.len() - 1) % ids.len()
            }
        };
        Some(ids[index])
    }

    /// Rehydrate the latest available assignment from retained parent history, without replaying
    /// lifecycle state or replacing newer live metadata with an older page.
    pub(super) fn restore_agent_assignments(
        &mut self,
        source: ThreadId,
        turns: &[codex_app_server_protocol::Turn],
    ) {
        let Some(root) = self.primary_thread_id else {
            return;
        };
        if source != root
            && !self
                .agent_navigation
                .panel_details
                .get(&source)
                .is_some_and(|d| d.root == Some(root))
        {
            return;
        }
        let mut latest = std::collections::HashMap::new();
        for item in turns
            .iter()
            .flat_map(codex_app_server_protocol::Turn::item_payloads)
        {
            if let ThreadItemPayload::SubAgentActivity {
                agent_thread_id,
                task_summary: Some(summary),
                ..
            } = item
                && let Ok(child) = ThreadId::from_string(agent_thread_id)
                && child != root
                && let Some(summary) = assignment_line(summary)
            {
                latest.insert(child, summary);
            }
        }
        for (child, summary) in latest {
            let details = self
                .agent_navigation
                .panel_details
                .entry(child)
                .or_default();
            if details.root.is_none_or(|known| known == root) {
                // Forked history can contain another parent's activities. Only live lineage or
                // ancestor-filtered discovery may establish panel membership.
                details.assignment.get_or_insert(summary);
            }
        }
    }

    pub(super) fn can_stop_agent_panel_turn(
        &self,
        thread_id: ThreadId,
        backend: crate::backend_id::BackendId,
    ) -> bool {
        self.primary_thread_id.is_some()
            && Some(thread_id) != self.primary_thread_id
            && self
                .agent_navigation
                .panel_details
                .get(&thread_id)
                .is_some_and(|details| details.root == self.primary_thread_id)
            && self.backend_registry.owner(thread_id).or_else(|| {
                self.primary_thread_id
                    .and_then(|root| self.backend_registry.owner(root))
            }) == Some(backend)
    }

    pub(super) fn remember_agent_panel_thread(&mut self, root: ThreadId, thread: &Thread) {
        let Ok(id) = ThreadId::from_string(&thread.id) else {
            return;
        };
        if id == root {
            return;
        }
        let details = self.agent_navigation.panel_details.entry(id).or_default();
        details.root = Some(root);
        details.parent = thread
            .parent_thread_id
            .as_deref()
            .and_then(|id| ThreadId::from_string(id).ok());
        if let Some(model) = &thread.model {
            details.set_model(model.clone());
        }
        details.task = thread
            .name
            .as_deref()
            .and_then(assignment_line)
            .unwrap_or_default();
        details.status_changed(&thread.status);
    }

    pub(super) fn track_agent_panel_notification(&mut self, notification: &ServerNotification) {
        let item = match notification {
            ServerNotification::ItemStarted(item) => Some(&item.item.payload),
            ServerNotification::ItemCompleted(item) => Some(&item.item.payload),
            _ => None,
        };
        // Text/tool deltas are hot-path traffic. They do not alter this panel and must not clone
        // every agent row or trigger picker refreshes on each streamed token.
        match notification {
            ServerNotification::ThreadStarted(_)
            | ServerNotification::ThreadTokenUsageUpdated(_)
            | ServerNotification::ThreadSettingsUpdated(_)
            | ServerNotification::ModelRerouted(_)
            | ServerNotification::ThreadStatusChanged(_)
            | ServerNotification::TurnStarted(_)
            | ServerNotification::TurnCompleted(_)
            | ServerNotification::ThreadClosed(_)
            | ServerNotification::ThreadReverted(_) => {}
            ServerNotification::ThreadNameUpdated(_) => {}
            _ if matches!(
                item,
                Some(
                    ThreadItemPayload::SubAgentActivity { .. }
                        | ThreadItemPayload::CollabAgentToolCall { .. }
                )
            ) => {}
            _ => return,
        }
        let Some(root) = self.primary_thread_id else {
            return;
        };
        let ServerNotificationThreadTarget::Thread(id) =
            server_notification_thread_target(notification)
        else {
            return;
        };
        if let ServerNotification::ThreadStarted(started) = notification {
            let parent = started
                .thread
                .parent_thread_id
                .as_deref()
                .and_then(|id| ThreadId::from_string(id).ok());
            let belongs = parent.is_some_and(|parent| {
                parent == root
                    || self
                        .agent_navigation
                        .panel_details
                        .get(&parent)
                        .is_some_and(|d| d.root == Some(root))
            });
            if belongs {
                self.remember_agent_panel_thread(root, &started.thread);
            } else if id != root {
                // Notifications from separate child streams can arrive before the parent's
                // spawn event. Retain lineage without displaying it in the wrong root.
                let details = self.agent_navigation.panel_details.entry(id).or_default();
                details.parent = parent;
                details.root = parent.is_none().then_some(id);
                if let Some(model) = &started.thread.model {
                    details.set_model(model.clone());
                }
                details.task = started
                    .thread
                    .name
                    .as_deref()
                    .and_then(assignment_line)
                    .unwrap_or_default();
                details.status_changed(&started.thread.status);
            }
            self.resolve_agent_panel_lineage(root);
        }
        let source_belongs = id == root
            || self
                .agent_navigation
                .panel_details
                .get(&id)
                .is_some_and(|details| details.root == Some(root));
        if !source_belongs {
            return;
        }
        if let Some(ThreadItemPayload::SubAgentActivity {
            agent_thread_id,
            task_summary,
            model,
            kind,
            ..
        }) = item
            && let Ok(child) = ThreadId::from_string(agent_thread_id)
        {
            let details = self
                .agent_navigation
                .panel_details
                .entry(child)
                .or_default();
            details.root = Some(root);
            // A follow-up sender need not be the agent's original spawning parent.
            if matches!(
                kind,
                codex_app_server_protocol::SubAgentActivityKind::Started
            ) {
                details.parent.get_or_insert(id);
            }
            if let Some(summary) = task_summary.as_deref().and_then(assignment_line) {
                details.assignment = Some(summary);
            }
            if let Some(model) = model
                && details.model.is_none()
            {
                details.set_model(model.clone());
            }
            use codex_app_server_protocol::SubAgentActivityKind;
            match kind {
                SubAgentActivityKind::Completed if details.status.as_deref() == Some("error") => {}
                SubAgentActivityKind::Completed | SubAgentActivityKind::Interrupted => {
                    details.status = Some(
                        if matches!(kind, SubAgentActivityKind::Completed) {
                            "done"
                        } else {
                            "interrupted"
                        }
                        .into(),
                    );
                    details.stale = false;
                    details.elapsed = details
                        .elapsed
                        .or_else(|| details.started.map(|start| start.elapsed()));
                    details.started = None;
                }
                SubAgentActivityKind::Started | SubAgentActivityKind::Interacted => {}
            }
        }
        if let Some(ThreadItemPayload::CollabAgentToolCall {
            receiver_thread_ids,
            model,
            tool,
            agents_states,
            ..
        }) = item
        {
            for child in receiver_thread_ids
                .iter()
                .filter_map(|id| ThreadId::from_string(id).ok())
            {
                if !matches!(tool, codex_app_server_protocol::CollabAgentTool::SpawnAgent)
                    && !self
                        .agent_navigation
                        .panel_details
                        .get(&child)
                        .is_some_and(|details| details.root == Some(root))
                {
                    continue;
                }
                let details = self
                    .agent_navigation
                    .panel_details
                    .entry(child)
                    .or_default();
                details.root = Some(root);
                details.parent = Some(id);
                if details.model.is_none()
                    && matches!(tool, codex_app_server_protocol::CollabAgentTool::SpawnAgent)
                    && let Some(model) = model
                {
                    details.set_model(model.clone());
                }
                if let Some(state) = agents_states.get(&child.to_string()) {
                    use codex_app_server_protocol::CollabAgentStatus;
                    let status = match state.status {
                        CollabAgentStatus::PendingInit => "starting",
                        CollabAgentStatus::Running => "running",
                        CollabAgentStatus::Completed => "done",
                        CollabAgentStatus::Interrupted => "interrupted",
                        CollabAgentStatus::Errored => "error",
                        CollabAgentStatus::Shutdown => "closed",
                        CollabAgentStatus::NotFound => "unavailable",
                    };
                    details.status = Some(status.into());
                    details.stale = false;
                    if !matches!(
                        state.status,
                        CollabAgentStatus::Running | CollabAgentStatus::PendingInit
                    ) {
                        details.elapsed = details
                            .elapsed
                            .or_else(|| details.started.map(|start| start.elapsed()));
                        details.started = None;
                    }
                }
            }
        }
        if item.is_some() {
            self.resolve_agent_panel_lineage(root);
        }
        if id != root {
            let details = self.agent_navigation.panel_details.entry(id).or_default();
            match notification {
                ServerNotification::ThreadTokenUsageUpdated(usage) => {
                    details.usage = Some(usage.token_usage.clone())
                }
                ServerNotification::ThreadSettingsUpdated(settings) => {
                    details.set_model(settings.thread_settings.model.clone());
                }
                ServerNotification::ThreadNameUpdated(name) => {
                    details.task = name
                        .thread_name
                        .as_deref()
                        .and_then(assignment_line)
                        .unwrap_or_default();
                }
                ServerNotification::ModelRerouted(rerouted) => {
                    details.set_model(rerouted.to_model.clone());
                    details.usage = None;
                }
                ServerNotification::ThreadStatusChanged(status) => {
                    details.status_changed(&status.status)
                }
                ServerNotification::TurnStarted(turn) => {
                    details.stale = false;
                    details.turn_id = Some(turn.turn.id.clone());
                    details.status = Some("running".into());
                    details.started = Some(Instant::now());
                    details.elapsed = None;
                }
                ServerNotification::TurnCompleted(turn)
                    if details
                        .turn_id
                        .as_deref()
                        .is_none_or(|id| id == turn.turn.id.as_str()) =>
                {
                    details.stale = false;
                    details.status = Some(
                        match turn.turn.status {
                            TurnStatus::Completed => "done",
                            TurnStatus::Interrupted => "interrupted",
                            TurnStatus::Failed => "error",
                            TurnStatus::InProgress => "running",
                        }
                        .into(),
                    );
                    details.elapsed = turn
                        .turn
                        .duration_ms
                        .filter(|duration| *duration >= 0)
                        .map(|duration| Duration::from_millis(duration as u64))
                        .or_else(|| details.started.map(|start| start.elapsed()));
                    details.started = None;
                }
                ServerNotification::ThreadClosed(_) => {
                    details.status = Some("closed".into());
                    details.elapsed = details
                        .elapsed
                        .or_else(|| details.started.map(|start| start.elapsed()));
                    details.started = None;
                }
                ServerNotification::ThreadReverted(_) => details.usage = None,
                _ => {}
            }
        }
        self.refresh_agent_picker();
    }

    pub(super) fn sync_subagent_panel(&mut self) {
        let root = self.primary_thread_id;
        if self.agent_navigation.panel_filter_root != root {
            self.agent_navigation.panel_filter_root = root;
            self.agent_navigation.panel_filter = Default::default();
        }
        // Discover existing children after resume as well as new live spawns. This uses the
        // existing asynchronous, ancestor-filtered picker refresh; no transcript replay or poll.
        if let Some(root) = root
            && self.agent_navigation.panel_discovery_root != Some(root)
            && let Some(owner) = self.backend_registry.owner(root)
            && let Some(handle) = self.backend_registry.request_handle(owner)
        {
            self.agent_navigation.panel_discovery_root = Some(root);
            self.refresh_agent_picker_with_handle(handle, root);
        }
        let current = self.current_displayed_thread_id();
        let mut rows: Vec<_> = self
            .agent_navigation
            .visible_threads()
            .into_iter()
            .filter_map(|(id, entry)| {
                let details = self.agent_navigation.panel_details.get(&id)?;
                if root.is_none() || details.root != root || Some(id) == root {
                    return None;
                }
                let approval_pending = self.agent_navigation.panel_pending_approvals.contains(&id)
                    || self.agent_navigation.pending_approvals.contains(&id);
                let active = approval_pending
                    || details.is_active(entry.is_running, entry.is_closed)
                    || (!details.stale
                        && !entry.is_closed
                        && details.status.as_deref() == Some("starting"));
                let state =
                    details.display_state(entry.is_running, entry.is_closed, approval_pending);
                Some(SubagentPanelRow {
                    id,
                    parent: false,
                    input_target: if self.agent_navigation.is_parent_owned(id) {
                        "disabled (parent-managed)".into()
                    } else {
                        self.thread_label(id)
                    },
                    label: self.thread_label(id),
                    state: state.into(),
                    model_context: details.model_context(),
                    task: details.assignment_text().to_owned(),
                    active,
                    approval_pending,
                    current: current == Some(id),
                    started: details.started,
                    elapsed: details.elapsed,
                })
            })
            .collect();
        if let Some(root) = root {
            rows.insert(
                0,
                SubagentPanelRow {
                    id: root,
                    parent: true,
                    input_target: "Main".into(),
                    label: "Main".into(),
                    state: "parent session".into(),
                    model_context: "Return to the parent transcript".into(),
                    task: String::new(),
                    active: false,
                    approval_pending: false,
                    current: current == Some(root),
                    started: None,
                    elapsed: None,
                },
            );
        }
        self.chat_widget
            .set_subagent_panel(rows, self.agent_navigation.panel_collapsed);
        self.chat_widget
            .set_subagent_panel_filter(self.agent_navigation.panel_filter);
    }

    fn resolve_agent_panel_lineage(&mut self, root: ThreadId) {
        // Each successful pass assigns at least one previously unresolved node. Cyclic or foreign
        // lineage stays unresolved, so this cannot loop on malformed metadata.
        loop {
            let ready: Vec<_> = self
                .agent_navigation
                .panel_details
                .iter()
                .filter_map(|(id, details)| {
                    if details.root == Some(root) {
                        return None;
                    }
                    details
                        .parent
                        .filter(|parent| {
                            *parent == root
                                || self
                                    .agent_navigation
                                    .panel_details
                                    .get(parent)
                                    .is_some_and(|parent| parent.root == Some(root))
                        })
                        .map(|_| *id)
                })
                .collect();
            if ready.is_empty() {
                break;
            }
            for id in ready {
                if let Some(details) = self.agent_navigation.panel_details.get_mut(&id) {
                    details.root = Some(root);
                }
            }
        }
    }

    pub(super) fn invalidate_agent_panel(&mut self) {
        for details in self.agent_navigation.panel_details.values_mut() {
            details.stale = true;
            details.usage = None;
            details.started = None;
            details.elapsed = None;
        }
        self.sync_subagent_panel();
    }
}
