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
    assert_eq!(details.display_state(true, false, true), "needs approval");
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
