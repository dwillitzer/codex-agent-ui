use super::*;
use crate::app::agent_panel::AgentPanelDetails;
use crate::chatwidget::tests::helpers::render_bottom_popup;
use codex_app_server_protocol::ThreadTokenUsage;
use codex_app_server_protocol::ThreadTokenUsageUpdatedNotification;
use codex_app_server_protocol::TokenUsageBreakdown;
use pretty_assertions::assert_eq;

fn spawn(parent: ThreadId, child: ThreadId, model: Option<&str>) -> ServerNotification {
    ServerNotification::ItemStarted(ItemStartedNotification::new(
        parent.to_string(),
        "parent-turn".into(),
        ThreadItemPayload::SubAgentActivity {
            task_summary: None,
            id: format!("spawn-{child}"),
            kind: codex_app_server_protocol::SubAgentActivityKind::Started,
            agent_thread_id: child.to_string(),
            agent_path: "/root/source-review".into(),
            model: model.map(str::to_string),
            reasoning_effort: None,
        },
        0,
    ))
}

fn usage(id: ThreadId, last: i64, total: i64, window: Option<i64>) -> ServerNotification {
    let tokens = |total_tokens| TokenUsageBreakdown {
        total_tokens,
        input_tokens: total_tokens,
        cached_input_tokens: 0,
        cache_write_input_tokens: 0,
        output_tokens: 0,
        reasoning_output_tokens: 0,
    };
    ServerNotification::ThreadTokenUsageUpdated(ThreadTokenUsageUpdatedNotification {
        thread_id: id.to_string(),
        turn_id: "child-turn".into(),
        token_usage: ThreadTokenUsage {
            last: tokens(last),
            total: tokens(total),
            model_context_window: window,
        },
    })
}

fn assignment_activity(
    root: ThreadId,
    child: ThreadId,
    summary: Option<&str>,
    kind: codex_app_server_protocol::SubAgentActivityKind,
) -> ServerNotification {
    let mut event = spawn(root, child, Some("review-model"));
    if let ServerNotification::ItemStarted(item) = &mut event
        && let ThreadItemPayload::SubAgentActivity {
            task_summary,
            kind: activity_kind,
            ..
        } = &mut item.item.payload
    {
        *task_summary = summary.map(str::to_owned);
        *activity_kind = kind;
    }
    event
}

#[tokio::test]
async fn agent_assignment_direct_cycle_stays_in_parent_family() {
    use crate::app::agent_navigation::AgentNavigationDirection::Next;
    use crate::app::agent_navigation::AgentNavigationDirection::Previous;
    let (mut app, _, _) = make_test_app_with_channels().await;
    let root = ThreadId::from_u128(1);
    let child = ThreadId::from_u128(2);
    let foreign = ThreadId::from_u128(3);
    app.primary_thread_id = Some(root);
    app.active_thread_id = Some(root);
    app.upsert_agent_picker_thread(root, None, None, false);
    app.track_agent_panel_notification(&spawn(root, child, Some("review-model")));
    app.upsert_agent_picker_thread(child, None, None, false);
    app.upsert_agent_picker_thread(foreign, None, None, false);
    assert_eq!(app.panel_adjacent_thread_id(Next), Some(child));
    assert_eq!(app.panel_adjacent_thread_id(Previous), Some(child));
    app.active_thread_id = Some(child);
    assert_eq!(app.panel_adjacent_thread_id(Next), Some(root));
    app.active_thread_id = Some(foreign);
    assert_eq!(app.panel_adjacent_thread_id(Next), Some(root));
}

#[tokio::test]
async fn agent_panel_status_filter_survives_widget_replacement_and_resets_for_new_parent() {
    use crate::bottom_pane::subagent_panel::AgentStatusFilter;
    let (mut app, _, _) = make_test_app_with_channels().await;
    let root = ThreadId::from_u128(1);
    let child = ThreadId::from_u128(2);
    app.primary_thread_id = Some(root);
    app.active_thread_id = Some(root);
    let event = spawn(root, child, Some("review-model"));
    app.track_agent_panel_notification(&event);
    app.cache_collab_receiver_threads_for_notification(&event);
    app.sync_subagent_panel();
    app.agent_navigation.panel_filter = AgentStatusFilter::Failed;
    app.sync_subagent_panel();
    assert!(render_bottom_popup(&app.chat_widget, 140).contains("[Failed 0]"));
    let mut replacement = make_test_app().await;
    std::mem::swap(&mut app.chat_widget, &mut replacement.chat_widget);
    app.active_thread_id = Some(child);
    app.sync_subagent_panel();
    assert!(render_bottom_popup(&app.chat_widget, 140).contains("[Failed 0]"));
    app.primary_thread_id = Some(ThreadId::from_u128(3));
    app.sync_subagent_panel();
    app.apply_agent_panel_filter(root, AgentStatusFilter::Failed);
    assert_eq!(app.agent_navigation.panel_filter, AgentStatusFilter::All);
}

#[tokio::test]
async fn agent_panel_status_filter_keeps_viewed_child_approval_active_and_globally_visible() {
    use crate::bottom_pane::subagent_panel::AgentStatusFilter;
    let (mut app, _, _) = make_test_app_with_channels().await;
    let root = ThreadId::from_u128(1);
    let child = ThreadId::from_u128(2);
    app.primary_thread_id = Some(root);
    app.active_thread_id = Some(child);
    let event = spawn(root, child, Some("review-model"));
    app.track_agent_panel_notification(&event);
    app.cache_collab_receiver_threads_for_notification(&event);
    app.ensure_thread_channel(child)
        .store
        .lock()
        .await
        .push_request(exec_approval_request(
            child,
            "approval-turn",
            "approval-call",
            None,
        ));
    app.refresh_pending_thread_approvals().await;
    assert!(!app.agent_navigation.pending_approvals.contains(&child));
    assert!(
        app.agent_navigation
            .panel_pending_approvals
            .contains(&child)
    );
    app.agent_navigation.panel_filter = AgentStatusFilter::Failed;
    app.sync_subagent_panel();
    let hidden = render_bottom_popup(&app.chat_widget, 140);
    assert!(hidden.contains("1 approval pending"), "{hidden}");
    assert!(hidden.contains("No agents match Failed."), "{hidden}");
    app.agent_navigation.panel_filter = AgentStatusFilter::Active;
    app.sync_subagent_panel();
    let active = render_bottom_popup(&app.chat_widget, 140);
    assert!(active.contains("waiting for approval"), "{active}");
    assert!(active.contains("[Active 1]"), "{active}");
    app.thread_event_channels[&child]
        .store
        .lock()
        .await
        .note_outbound_op(&Op::ExecApproval {
            id: "approval-call".into(),
            turn_id: Some("approval-turn".into()),
            decision: codex_app_server_protocol::CommandExecutionApprovalDecision::Cancel,
        });
    app.refresh_pending_thread_approvals().await;
    assert!(
        !app.agent_navigation
            .panel_pending_approvals
            .contains(&child)
    );
    assert!(!render_bottom_popup(&app.chat_widget, 140).contains("approval pending"));
}

#[tokio::test]
async fn agent_panel_status_filter_preserves_explicit_failure_over_generic_completion() {
    use codex_app_server_protocol::SubAgentActivityKind;
    let (mut app, _, _) = make_test_app_with_channels().await;
    let root = ThreadId::from_u128(1);
    let child = ThreadId::from_u128(2);
    app.primary_thread_id = Some(root);
    let event = spawn(root, child, Some("review-model"));
    app.track_agent_panel_notification(&event);
    app.cache_collab_receiver_threads_for_notification(&event);
    app.track_agent_panel_notification(&ServerNotification::TurnCompleted(
        codex_app_server_protocol::TurnCompletedNotification {
            thread_id: child.to_string(),
            turn: test_turn("failed-turn", TurnStatus::Failed, vec![]),
        },
    ));
    app.track_agent_panel_notification(&assignment_activity(
        root,
        child,
        None,
        SubAgentActivityKind::Completed,
    ));
    assert_eq!(
        app.agent_navigation.panel_details[&child].status.as_deref(),
        Some("error")
    );
    app.agent_navigation.panel_filter =
        crate::bottom_pane::subagent_panel::AgentStatusFilter::Failed;
    app.sync_subagent_panel();
    assert!(render_bottom_popup(&app.chat_widget, 140).contains("[Failed 1]"));
}

#[tokio::test]
async fn agent_assignment_uses_current_parent_summary_instead_of_initial_prompt() {
    use codex_app_server_protocol::SubAgentActivityKind;
    let (mut app, _, _) = make_test_app_with_channels().await;
    let root = ThreadId::from_u128(1);
    let child = ThreadId::from_u128(2);
    app.primary_thread_id = Some(root);
    app.active_thread_id = Some(root);
    app.upsert_agent_picker_thread(child, Some("Reviewer".into()), None, false);
    let mut thread: codex_app_server_protocol::Thread = serde_json::from_value(serde_json::json!({
        "id": child.to_string(), "sessionId": root.to_string(), "parentThreadId": root.to_string(),
        "preview": "Your initial prompt says to sleep first and then follow many detailed instructions",
        "ephemeral": false, "modelProvider": "openai", "model": "review-model",
        "createdAt": 1, "updatedAt": 2, "status": {"type":"idle"}, "cwd": "/tmp",
        "cliVersion": "0.0.0", "source": "unknown", "turns": []
    })).unwrap();
    app.remember_agent_panel_thread(root, &thread);
    app.sync_subagent_panel();
    let unknown = render_bottom_popup(&app.chat_widget, 140);
    assert!(unknown.contains("Assignment unavailable"), "{unknown}");
    assert!(!unknown.contains("initial prompt"));
    thread.name = Some("Saved audit title".into());
    app.remember_agent_panel_thread(root, &thread);
    assert_eq!(
        app.agent_navigation.panel_details[&child].assignment_text(),
        "Saved audit title"
    );
    app.track_agent_panel_notification(&assignment_activity(
        root,
        child,
        Some("Audit retry validation"),
        SubAgentActivityKind::Started,
    ));
    app.remember_agent_panel_thread(root, &thread);
    let rendered = render_bottom_popup(&app.chat_widget, 140);
    assert!(rendered.contains("Audit retry validation"), "{rendered}");
    assert!(!rendered.contains("Saved audit title"));
    app.track_agent_panel_notification(&assignment_activity(
        root,
        child,
        Some("Check null retry limits"),
        SubAgentActivityKind::Interacted,
    ));
    app.track_agent_panel_notification(&assignment_activity(
        root,
        child,
        None,
        SubAgentActivityKind::Completed,
    ));
    let rendered = render_bottom_popup(&app.chat_widget, 140);
    assert!(rendered.contains("Check null retry limits"), "{rendered}");
    assert!(!rendered.contains("Audit retry validation"));
}

#[tokio::test]
async fn agent_assignment_history_restores_latest_without_overwriting_live_updates() {
    use codex_app_server_protocol::SubAgentActivityKind;
    let (mut app, _, _) = make_test_app_with_channels().await;
    let root = ThreadId::from_u128(1);
    let child = ThreadId::from_u128(2);
    app.primary_thread_id = Some(root);
    let turn = |id: &str, summary: &str| {
        let ServerNotification::ItemStarted(item) =
            assignment_activity(root, child, Some(summary), SubAgentActivityKind::Interacted)
        else {
            unreachable!()
        };
        test_turn(id, TurnStatus::Completed, vec![item.item.payload])
    };
    let older = turn("older", "Audit retries");
    let recent = turn("recent", "Check null limits");
    app.restore_agent_assignments(root, &[older.clone(), recent]);
    assert_eq!(
        app.agent_navigation.panel_details[&child].assignment_text(),
        "Check null limits"
    );
    assert!(app.agent_navigation.panel_details[&child].status.is_none());
    assert!(app.agent_navigation.panel_details[&child].root.is_none());
    app.track_agent_panel_notification(&assignment_activity(
        root,
        child,
        Some("Review final patch"),
        SubAgentActivityKind::Interacted,
    ));
    app.restore_agent_assignments(root, &[older]);
    assert_eq!(
        app.agent_navigation.panel_details[&child].assignment_text(),
        "Review final patch"
    );
    let sibling = ThreadId::from_u128(3);
    // Discovering the child's actual parent is independent of assignment-message senders.
    app.agent_navigation
        .panel_details
        .get_mut(&child)
        .unwrap()
        .parent = Some(root);
    app.track_agent_panel_notification(&assignment_activity(
        root,
        sibling,
        Some("Review tests"),
        SubAgentActivityKind::Started,
    ));
    app.track_agent_panel_notification(&assignment_activity(
        sibling,
        child,
        Some("Review edge cases"),
        SubAgentActivityKind::Interacted,
    ));
    assert_eq!(
        app.agent_navigation.panel_details[&child].parent,
        Some(root)
    );
    assert_eq!(
        app.agent_navigation.panel_details[&child].assignment_text(),
        "Review edge cases"
    );
}

#[tokio::test]
async fn agent_panel_is_root_scoped_and_uses_latest_child_context() {
    let (mut app, _, _) = make_test_app_with_channels().await;
    let root = ThreadId::from_u128(1);
    let child = ThreadId::from_u128(2);
    let grandchild = ThreadId::from_u128(3);
    let unrelated = ThreadId::from_u128(4);
    app.primary_thread_id = Some(root);
    app.active_thread_id = Some(root);
    app.upsert_agent_picker_thread(root, None, None, false);
    for event in [
        spawn(root, child, Some("child-model")),
        spawn(child, grandchild, None),
    ] {
        app.track_agent_panel_notification(&event);
        app.cache_collab_receiver_threads_for_notification(&event);
    }
    // A different root and its children must never enter this parent's panel.
    let foreign = spawn(unrelated, ThreadId::from_u128(5), Some("foreign-model"));
    app.track_agent_panel_notification(&foreign);
    app.cache_collab_receiver_threads_for_notification(&foreign);
    app.track_agent_panel_notification(&usage(child, 60_000, 900_000, Some(108_000)));
    app.sync_subagent_panel();
    let rendered = render_bottom_popup(&app.chat_widget, 140);
    assert!(rendered.contains("Agents: 2 active / 2"), "{rendered}");
    assert!(
        rendered.contains("child-model · ctx 50% used (60K/108K)"),
        "{rendered}"
    );
    assert!(
        rendered.contains("model unavailable · ctx unavailable"),
        "{rendered}"
    );
    assert!(!rendered.contains("foreign-model"));
    // Compaction lowers current occupancy even while lifetime usage increases.
    app.track_agent_panel_notification(&usage(child, 12_000, 1_000_000, Some(108_000)));
    let rendered = render_bottom_popup(&app.chat_widget, 140);
    assert!(rendered.contains("ctx 0% used (12K/108K)"), "{rendered}");
    app.agent_navigation.clear();
    app.sync_subagent_panel();
    assert!(!render_bottom_popup(&app.chat_widget, 140).contains("Agents:"));
}

#[tokio::test]
async fn agent_panel_unloading_preserves_observed_turn_outcomes_and_filter_membership() {
    let (mut app, _, _) = make_test_app_with_channels().await;
    let root = ThreadId::new();
    let child = ThreadId::new();
    app.primary_thread_id = Some(root);
    app.active_thread_id = Some(root);
    let event = spawn(root, child, Some("review-model"));
    app.track_agent_panel_notification(&event);
    app.cache_collab_receiver_threads_for_notification(&event);
    for (outcome, state, count) in [
        (TurnStatus::Completed, "done", "Done 1"),
        (TurnStatus::Failed, "error", "Failed 1"),
        (TurnStatus::Interrupted, "interrupted", "Done 0"),
    ] {
        app.track_agent_panel_notification(&ServerNotification::TurnStarted(
            codex_app_server_protocol::TurnStartedNotification {
                thread_id: child.to_string(),
                turn: test_turn("child-turn", TurnStatus::InProgress, vec![]),
            },
        ));
        assert_eq!(
            app.agent_navigation.panel_details[&child].status.as_deref(),
            Some("running")
        );
        app.track_agent_panel_notification(&ServerNotification::TurnCompleted(
            codex_app_server_protocol::TurnCompletedNotification {
                thread_id: child.to_string(),
                turn: test_turn("child-turn", outcome, vec![]),
            },
        ));
        app.agent_navigation.mark_stopped(child);
        app.track_agent_panel_notification(&ServerNotification::ThreadStatusChanged(
            codex_app_server_protocol::ThreadStatusChangedNotification {
                thread_id: child.to_string(),
                status: codex_app_server_protocol::ThreadStatus::NotLoaded,
            },
        ));
        app.sync_subagent_panel();
        assert_eq!(
            app.agent_navigation.panel_details[&child].status.as_deref(),
            Some(state)
        );
        let text = render_bottom_popup(&app.chat_widget, 140);
        assert!(text.contains(count), "{text}");
    }
    let unknown = ThreadId::new();
    let event = spawn(root, unknown, None);
    app.track_agent_panel_notification(&event);
    app.cache_collab_receiver_threads_for_notification(&event);
    app.track_agent_panel_notification(&ServerNotification::ThreadStatusChanged(
        codex_app_server_protocol::ThreadStatusChangedNotification {
            thread_id: unknown.to_string(),
            status: codex_app_server_protocol::ThreadStatus::NotLoaded,
        },
    ));
    assert_eq!(
        app.agent_navigation.panel_details[&unknown]
            .status
            .as_deref(),
        Some("unloaded")
    );
}

#[tokio::test]
async fn agent_panel_ignores_old_completion_and_preserves_model_on_metadata_upsert() {
    let (mut app, _, _) = make_test_app_with_channels().await;
    let root = ThreadId::from_u128(1);
    let child = ThreadId::from_u128(2);
    app.primary_thread_id = Some(root);
    app.active_thread_id = Some(root);
    let event = spawn(root, child, Some("review-model"));
    app.track_agent_panel_notification(&event);
    app.cache_collab_receiver_threads_for_notification(&event);
    app.track_agent_panel_notification(&ServerNotification::TurnStarted(
        codex_app_server_protocol::TurnStartedNotification {
            thread_id: child.to_string(),
            turn: test_turn("new-turn", TurnStatus::InProgress, vec![]),
        },
    ));
    app.track_agent_panel_notification(&ServerNotification::TurnCompleted(
        codex_app_server_protocol::TurnCompletedNotification {
            thread_id: child.to_string(),
            turn: test_turn("old-turn", TurnStatus::Completed, vec![]),
        },
    ));
    assert_eq!(
        app.agent_navigation.panel_details[&child].status.as_deref(),
        Some("running")
    );
    app.upsert_agent_picker_thread(
        child,
        Some("Reviewer".into()),
        Some("explorer".into()),
        false,
    );
    assert_eq!(
        app.agent_navigation.panel_details[&child].model.as_deref(),
        Some("review-model")
    );
    app.track_agent_panel_notification(&ServerNotification::TurnCompleted(
        codex_app_server_protocol::TurnCompletedNotification {
            thread_id: child.to_string(),
            turn: test_turn("new-turn", TurnStatus::Completed, vec![]),
        },
    ));
    app.agent_navigation.mark_stopped(child);
    app.sync_subagent_panel();
    let rendered = render_bottom_popup(&app.chat_widget, 120);
    assert!(
        rendered.contains("/root/source-review · done"),
        "{rendered}"
    );
    assert!(rendered.contains("Agents: 0 active / 1"), "{rendered}");
    app.agent_navigation.panel_collapsed = true;
    app.sync_subagent_panel();
    let compact = render_bottom_popup(&app.chat_widget, 120);
    assert!(compact.contains("Agents: 0 active / 1"));
    assert!(!compact.contains("/root/source-review"));
}

#[tokio::test]
async fn agent_picker_metadata_keeps_existing_watch_action() {
    let (mut app, mut rx, _) = make_test_app_with_channels().await;
    let root = ThreadId::from_u128(1);
    let child = ThreadId::from_u128(2);
    app.primary_thread_id = Some(root);
    app.active_thread_id = Some(root);
    app.upsert_agent_picker_thread(root, None, None, false);
    let event = spawn(root, child, Some("worker-model"));
    app.track_agent_panel_notification(&event);
    app.cache_collab_receiver_threads_for_notification(&event);
    app.track_agent_panel_notification(&usage(child, 60_000, 500_000, Some(108_000)));
    let params = app.agent_picker_selection_view_params(Some(child));
    let index = app
        .agent_navigation
        .picker_row_ids
        .iter()
        .position(|id| *id == child)
        .unwrap();
    let row = &params.items[index];
    assert!(
        row.description
            .as_deref()
            .unwrap()
            .contains("worker-model · ctx 50% used")
    );
    assert_eq!(params.initial_selected_idx, Some(index));
    // Selecting the row still watches the existing thread; it never spawns another child.
    while rx.try_recv().is_ok() {}
    (row.actions[0])(&app.app_event_tx);
    assert!(matches!(rx.try_recv().unwrap(), AppEvent::SelectAgentThread(id) if id == child));
}

#[test]
fn agent_context_unknown_is_not_zero_and_percent_is_clamped() {
    let mut details = AgentPanelDetails::default();
    assert_eq!(
        details.model_context(),
        "model unavailable · ctx unavailable"
    );
    for window in [None, Some(0), Some(-1)] {
        if let ServerNotification::ThreadTokenUsageUpdated(update) =
            usage(ThreadId::new(), 0, 0, window)
        {
            details.usage = Some(update.token_usage);
        }
        assert!(details.model_context().ends_with("ctx unavailable"));
    }
    for (tokens, expected) in [(-1, "ctx 0% used"), (500_000, "ctx 100% used")] {
        if let ServerNotification::ThreadTokenUsageUpdated(update) =
            usage(ThreadId::new(), tokens, tokens, Some(108_000))
        {
            details.usage = Some(update.token_usage);
        }
        assert!(details.model_context().contains(expected));
    }
}

#[test]
fn agent_panel_state_distinguishes_idle_waiting_completed_and_disconnected() {
    let mut details = AgentPanelDetails::default();
    assert_eq!(details.display_state(false, false, false), "unknown");
    details.status = Some("idle".into());
    assert_eq!(details.display_state(false, false, false), "idle");
    details.status = Some("waiting".into());
    assert_eq!(details.display_state(true, false, false), "waiting");
    assert!(details.is_active(true, false));
    assert_eq!(
        details.display_state(true, false, true),
        "waiting for approval"
    );
    details.status = Some("done".into());
    assert_eq!(details.display_state(true, false, false), "done");
    assert!(!details.is_active(true, false));
    details.stale = true;
    assert_eq!(
        details.display_state(true, false, false),
        "unknown (reconnecting)"
    );
    assert!(!details.is_active(true, false));
}

#[tokio::test]
async fn agent_panel_stop_is_limited_to_this_parents_children_and_backend() {
    let (mut app, _, _) = make_test_app_with_channels().await;
    let root = ThreadId::from_u128(1);
    let child = ThreadId::from_u128(2);
    let foreign = ThreadId::from_u128(3);
    let backend = crate::backend_id::BackendId::new();
    let other_backend = crate::backend_id::BackendId::new();
    app.primary_thread_id = Some(root);
    app.track_agent_panel_notification(&spawn(root, child, Some("worker")));
    app.track_agent_panel_notification(&spawn(foreign, ThreadId::from_u128(4), None));
    app.backend_registry.observe(backend, root);
    // A child discovered through the root-scoped list may not yet have its own event subscription.
    assert!(app.can_stop_agent_panel_turn(child, backend));
    app.backend_registry.observe(backend, child);
    app.backend_registry.observe(backend, foreign);
    assert!(app.can_stop_agent_panel_turn(child, backend));
    assert!(!app.can_stop_agent_panel_turn(root, backend));
    assert!(!app.can_stop_agent_panel_turn(foreign, backend));
    assert!(!app.can_stop_agent_panel_turn(child, other_backend));
    app.primary_thread_id = Some(foreign);
    assert!(!app.can_stop_agent_panel_turn(child, backend));
}

#[tokio::test]
async fn agent_panel_focus_owns_enter_and_escape_while_reading_history() -> Result<()> {
    let (mut app, mut events, _) = make_test_app_with_channels().await;
    let mut server = start_config_write_test_app_server(&app).await?;
    let mut tui = crate::tui::test_support::make_test_tui()?;
    let root = ThreadId::from_u128(1);
    let child = ThreadId::from_u128(2);
    app.primary_thread_id = Some(root);
    app.active_thread_id = Some(root);
    app.upsert_agent_picker_thread(root, None, None, false);
    let event = spawn(root, child, Some("worker"));
    app.track_agent_panel_notification(&event);
    app.cache_collab_receiver_threads_for_notification(&event);
    app.sync_subagent_panel();
    read_side_history(&mut app, &mut tui, "panel-reading")?;
    assert!(!app.transcript_view.is_following());
    assert!(
        app.chat_widget
            .handle_subagent_panel_navigation_key(KeyCode::F(6).into())
    );
    app.handle_tui_event(&mut tui, &mut server, TuiEvent::Key(KeyCode::Esc.into()))
        .await?;
    assert!(!app.transcript_view.is_following());
    assert!(!app.chat_widget.subagent_panel_focused());
    assert!(
        app.chat_widget
            .handle_subagent_panel_navigation_key(KeyCode::F(6).into())
    );
    app.chat_widget
        .handle_subagent_panel_navigation_key(KeyCode::Down.into());
    while events.try_recv().is_ok() {}
    app.handle_tui_event(&mut tui, &mut server, TuiEvent::Key(KeyCode::Enter.into()))
        .await?;
    assert!(
        std::iter::from_fn(|| events.try_recv().ok())
            .any(|event| matches!(event, AppEvent::SelectAgentThread(id) if id == child))
    );
    assert!(!app.transcript_view.is_following());
    Ok(())
}

#[tokio::test]
async fn agent_panel_disconnect_discards_stale_context_but_retains_membership() {
    let (mut app, _, _) = make_test_app_with_channels().await;
    let root = ThreadId::from_u128(1);
    let child = ThreadId::from_u128(2);
    app.primary_thread_id = Some(root);
    let event = spawn(root, child, Some("child-model"));
    app.track_agent_panel_notification(&event);
    app.cache_collab_receiver_threads_for_notification(&event);
    app.track_agent_panel_notification(&usage(child, 60_000, 500_000, Some(108_000)));
    app.invalidate_agent_panel();
    let rendered = render_bottom_popup(&app.chat_widget, 140);
    assert!(rendered.contains("Agents: 0 active / 1"), "{rendered}");
    assert!(rendered.contains("unknown (reconnecting)"), "{rendered}");
    assert!(rendered.contains("ctx unavailable"), "{rendered}");
    assert_eq!(app.agent_navigation.panel_details[&child].root, Some(root));
}

#[tokio::test]
async fn agent_panel_model_reroute_invalidates_previous_model_context() {
    let (mut app, _, _) = make_test_app_with_channels().await;
    let root = ThreadId::from_u128(1);
    let child = ThreadId::from_u128(2);
    app.primary_thread_id = Some(root);
    let event = spawn(root, child, Some("old-model"));
    app.track_agent_panel_notification(&event);
    app.cache_collab_receiver_threads_for_notification(&event);
    app.track_agent_panel_notification(&usage(child, 60_000, 500_000, Some(108_000)));
    app.track_agent_panel_notification(&ServerNotification::ModelRerouted(
        codex_app_server_protocol::ModelReroutedNotification {
            thread_id: child.to_string(),
            turn_id: "child-turn".into(),
            from_model: "old-model".into(),
            to_model: "new-model".into(),
            reason: codex_app_server_protocol::ModelRerouteReason::HighRiskCyberActivity,
        },
    ));
    let rendered = render_bottom_popup(&app.chat_widget, 140);
    assert!(
        rendered.contains("new-model · ctx unavailable"),
        "{rendered}"
    );
    assert!(!rendered.contains("old-model"), "{rendered}");
}

#[tokio::test]
async fn agent_panel_resolves_out_of_order_nested_lineage_without_showing_foreign_threads() {
    let (mut app, _, _) = make_test_app_with_channels().await;
    let root = ThreadId::from_u128(1);
    let parent = ThreadId::from_u128(2);
    let child = ThreadId::from_u128(3);
    let foreign = ThreadId::from_u128(4);
    app.primary_thread_id = Some(root);
    let started = |id: ThreadId, parent: ThreadId| {
        ServerNotification::ThreadStarted(ThreadStartedNotification {
            thread: serde_json::from_value(serde_json::json!({
                "id": id.to_string(), "sessionId": root.to_string(),
                "parentThreadId": parent.to_string(), "preview": "Review nested work",
                "ephemeral": false, "modelProvider": "openai", "model": "nested-model",
                "createdAt": 1, "updatedAt": 2, "status": {"type": "idle"},
                "cwd": "/tmp", "cliVersion": "0.0.0", "source": "unknown", "turns": []
            }))
            .unwrap(),
        })
    };
    app.track_agent_panel_notification(&started(child, parent));
    app.upsert_agent_picker_thread(child, Some("Nested reviewer".into()), None, false);
    app.track_agent_panel_notification(&started(foreign, ThreadId::from_u128(99)));
    app.upsert_agent_picker_thread(foreign, Some("Foreign reviewer".into()), None, false);
    app.sync_subagent_panel();
    assert!(!render_bottom_popup(&app.chat_widget, 120).contains("Agents:"));
    let event = spawn(root, parent, Some("parent-model"));
    app.track_agent_panel_notification(&event);
    app.cache_collab_receiver_threads_for_notification(&event);
    let rendered = render_bottom_popup(&app.chat_widget, 120);
    assert!(rendered.contains("Nested reviewer"), "{rendered}");
    assert!(rendered.contains("nested-model"), "{rendered}");
    assert!(!rendered.contains("Foreign reviewer"), "{rendered}");
    assert_eq!(app.agent_navigation.panel_details[&child].root, Some(root));
    assert_eq!(app.agent_navigation.panel_details[&foreign].root, None);
    app.agent_picker_selection_view_params(None);
    assert!(app.agent_navigation.picker_row_ids.contains(&child));
    assert!(!app.agent_navigation.picker_row_ids.contains(&foreign));
}
