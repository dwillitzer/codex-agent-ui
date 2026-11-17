use super::*;
use codex_protocol::ThreadId;
use pretty_assertions::assert_eq;

fn rows() -> Vec<subagent_panel::SubagentPanelRow> {
    (1..=2)
        .map(|id| subagent_panel::SubagentPanelRow {
            id: ThreadId::from_u128(id),
            parent: id == 1,
            input_target: "Main".into(),
            label: if id == 1 { "Main" } else { "reviewer" }.into(),
            state: "running".into(),
            model_context: "model · ctx unavailable".into(),
            task: String::new(),
            active: id == 2,
            current: id == 1,
            started: None,
            elapsed: None,
        })
        .collect()
}

#[test]
fn subagent_panel_enter_inspects_selected_thread_without_submitting_input() {
    let (tx, mut rx) = unbounded_channel();
    let mut pane = test_pane(AppEventSender::new(tx));
    pane.set_subagent_panel(rows(), false);
    assert!(pane.handle_subagent_panel_key(KeyCode::Down.into()));
    assert!(pane.handle_subagent_panel_key(KeyCode::Down.into()));
    assert_eq!(
        pane.handle_key_event(KeyCode::Enter.into()),
        InputResult::None
    );
    assert!(
        matches!(rx.try_recv(), Ok(AppEvent::SelectAgentThread(id)) if id == ThreadId::from_u128(2))
    );
    assert!(rx.try_recv().is_err());
    assert!(!pane.subagent_panel.focused());
}

#[test]
fn subagent_panel_escape_preserves_draft_and_does_not_interrupt() {
    let (tx, mut rx) = unbounded_channel();
    let mut pane = test_pane(AppEventSender::new(tx));
    pane.set_subagent_panel(rows(), false);
    pane.composer.insert_str("unfinished draft");
    pane.is_task_running = true;
    assert!(!pane.handle_subagent_panel_key(KeyCode::Down.into()));
    assert!(pane.handle_subagent_panel_key(KeyCode::F(6).into()));
    pane.handle_key_event(KeyCode::Esc.into());
    pane.handle_key_event(KeyEvent::new_with_kind(
        KeyCode::Esc,
        KeyModifiers::NONE,
        KeyEventKind::Repeat,
    ));
    assert_eq!(pane.composer.current_text(), "unfinished draft");
    assert!(!pane.subagent_panel.focused());
    assert!(rx.try_recv().is_err());
}

#[test]
fn subagent_panel_enter_repeat_cannot_submit_the_draft() {
    let (tx, mut rx) = unbounded_channel();
    let mut pane = test_pane(AppEventSender::new(tx));
    pane.set_subagent_panel(rows(), false);
    pane.composer.insert_str("keep this draft");
    assert!(pane.handle_subagent_panel_key(KeyCode::F(6).into()));
    assert!(pane.cursor_pos(Rect::new(0, 0, 100, 20)).is_none());
    pane.handle_key_event(KeyCode::Enter.into());
    assert!(matches!(rx.try_recv(), Ok(AppEvent::SelectAgentThread(_))));
    assert_eq!(
        pane.handle_key_event(KeyEvent::new_with_kind(
            KeyCode::Enter,
            KeyModifiers::NONE,
            KeyEventKind::Repeat
        )),
        InputResult::None
    );
    assert_eq!(pane.composer.current_text(), "keep this draft");
    assert!(rx.try_recv().is_err());
}

#[test]
fn subagent_panel_editor_remap_and_popup_take_precedence() {
    use codex_config::types::KeybindingSpec;
    use codex_config::types::KeybindingsSpec;
    use codex_config::types::TuiKeymap;
    let (tx, _rx) = unbounded_channel();
    let mut pane = test_pane(AppEventSender::new(tx));
    pane.set_subagent_panel(rows(), false);
    let mut config = TuiKeymap::default();
    config.editor.move_down = Some(KeybindingsSpec::One(KeybindingSpec("ctrl-n".into())));
    let keymap = RuntimeKeymap::from_config(&config).unwrap();
    pane.composer.set_keymap_bindings(&keymap);
    assert!(!pane.handle_subagent_panel_key(KeyCode::Down.into()));
    pane.composer
        .set_text_content("/".into(), Vec::new(), Vec::new());
    assert!(!pane.handle_subagent_panel_key(KeyCode::F(6).into()));
}

#[test]
fn subagent_panel_navigation_works_when_direct_input_is_disabled() {
    let (tx, mut rx) = unbounded_channel();
    let mut pane = test_pane(AppEventSender::new(tx));
    let mut rows = rows();
    rows[0].current = false;
    rows[1].current = true;
    pane.set_subagent_panel(rows, false);
    pane.composer.set_input_enabled(false, None);
    assert!(pane.handle_subagent_panel_navigation_key(KeyCode::F(6).into()));
    assert!(pane.handle_subagent_panel_navigation_key(KeyCode::Up.into()));
    assert!(pane.handle_subagent_panel_navigation_key(KeyCode::Enter.into()));
    assert!(
        matches!(rx.try_recv(), Ok(AppEvent::SelectAgentThread(id)) if id == ThreadId::from_u128(1))
    );
}

#[test]
fn subagent_panel_f6_respects_custom_editor_binding() {
    use codex_config::types::KeybindingSpec;
    use codex_config::types::KeybindingsSpec;
    use codex_config::types::TuiKeymap;
    let (tx, _rx) = unbounded_channel();
    let mut pane = test_pane(AppEventSender::new(tx));
    pane.set_subagent_panel(rows(), false);
    let mut config = TuiKeymap::default();
    config.editor.move_up = Some(KeybindingsSpec::One(KeybindingSpec("f6".into())));
    pane.set_keymap_bindings(&RuntimeKeymap::from_config(&config).unwrap());
    assert!(!pane.handle_subagent_panel_navigation_key(KeyCode::F(6).into()));
}

#[test]
fn subagent_panel_stop_requests_confirmation_for_selected_id() {
    let (tx, mut rx) = unbounded_channel();
    let mut pane = test_pane(AppEventSender::new(tx));
    pane.set_subagent_panel(rows(), false);
    pane.focus_subagent_panel();
    pane.handle_subagent_panel_navigation_key(KeyCode::Down.into());
    pane.handle_subagent_panel_navigation_key(KeyCode::Char('a').into());
    assert!(
        matches!(rx.try_recv(), Ok(AppEvent::AgentPanelStopTurn {thread_id, confirmed: false}) if thread_id == ThreadId::from_u128(2))
    );
    assert!(rx.try_recv().is_err());
}
