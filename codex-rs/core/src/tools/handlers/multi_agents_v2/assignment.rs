//! Human-facing assignment metadata, separate from model-reserved collaboration signatures.

use super::*;
use crate::tools::context::FunctionToolOutput;
use codex_tools::JsonSchema;
use codex_tools::ResponsesApiNamespace;
use codex_tools::ResponsesApiNamespaceTool;
use codex_tools::ResponsesApiTool;
use codex_tools::ToolSpec;
use std::collections::BTreeMap;

pub(crate) struct Handler;

impl ToolExecutor<ToolInvocation> for Handler {
    fn tool_name(&self) -> ToolName {
        ToolName::namespaced("agent_ui", "set_assignment")
    }

    fn spec(&self) -> ToolSpec {
        ToolSpec::Namespace(ResponsesApiNamespace {
            name: "agent_ui".into(),
            description: "Update the human-facing agent list without sending agent input.".into(),
            tools: vec![ResponsesApiNamespaceTool::Function(ResponsesApiTool {
                name: "set_assignment".into(),
                description: "Set a concise, single-line assignment summary shown in the user's agent list. Call after successfully spawning an agent and after assigning a new follow-up task. Use at most 80 characters describing the work, not the role or full instructions. This updates metadata only; it does not wake, interrupt, or send input to the target. Target a loaded non-root agent in this tree, or a recorded direct child.".into(),
                strict: false,
                defer_loading: None,
                parameters: JsonSchema::object(
                    BTreeMap::from([
                        ("target".into(), JsonSchema::string(Some("Agent ID or canonical task name from spawn_agent.".into()))),
                        ("summary".into(), JsonSchema::string(Some("Short current assignment, at most 80 characters.".into()))),
                    ]),
                    Some(vec!["target".into(), "summary".into()]),
                    Some(false.into()),
                ),
                output_schema: None,
            })],
        })
    }

    fn handle<'a>(&'a self, invocation: ToolInvocation) -> codex_tools::ToolExecutorFuture<'a>
    where
        ToolInvocation: 'a,
    {
        Box::pin(async move {
            let ToolInvocation {
                session,
                turn,
                payload,
                call_id,
                ..
            } = invocation;
            let args: AssignmentArgs = parse_arguments(&function_arguments(payload)?)?;
            let summary = normalize_summary(&args.summary)?;
            let target = resolve_agent_target(&session, &turn, &args.target).await?;
            if target == session.thread_id {
                return Err(FunctionCallError::RespondToModel(
                    "Assignment metadata must target another non-root agent".into(),
                ));
            }
            // Read membership without loading or resuming a task. UUID resolution alone is not
            // sufficient authority to update a different tree's presentation metadata.
            let agents = session
                .services
                .agent_control
                .list(
                    session.thread_id,
                    turn.parent_thread_id,
                    &turn.session_source,
                    None,
                )
                .await
                .map_err(collab_spawn_error)?;
            let mut path = agents
                .into_iter()
                .find(|agent| agent.thread_id == target)
                .and_then(|agent| agent.metadata.agent_path)
                .filter(|path| !path.is_root());
            // Short jobs can be unloaded before the parent's metadata call. Recorded direct
            // children still provide authoritative identity without resuming a finished task.
            if path.is_none() {
                for candidate in session
                    .services
                    .agent_control
                    .child_agent_paths(session.thread_id)
                    .await
                {
                    let name = candidate.to_string();
                    if session
                        .services
                        .agent_control
                        .resolve(
                            session.thread_id,
                            turn.parent_thread_id,
                            &turn.session_source,
                            &name,
                        )
                        .await
                        .ok()
                        == Some(target)
                    {
                        path = Some(candidate);
                        break;
                    }
                }
            }
            let path = path.ok_or_else(|| {
                FunctionCallError::RespondToModel(
                    "Assignment target must be a non-root agent in this tree".into(),
                )
            })?;
            emit_sub_agent_activity(
                &session,
                &turn,
                SubAgentActivityItem {
                    task_summary: Some(summary.clone()),
                    model: None,
                    reasoning_effort: None,
                    id: call_id,
                    agent_thread_id: target,
                    agent_path: path,
                    kind: SubAgentActivityKind::Interacted,
                },
            )
            .await;
            Ok(boxed_tool_output(FunctionToolOutput::from_text(
                serde_json::json!({"target": target, "summary": summary}).to_string(),
                Some(true),
            )))
        })
    }
}

impl CoreToolRuntime for Handler {
    fn matches_kind(&self, payload: &ToolPayload) -> bool {
        matches!(payload, ToolPayload::Function { .. })
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct AssignmentArgs {
    target: String,
    summary: String,
}

fn normalize_summary(summary: &str) -> Result<String, FunctionCallError> {
    let clean: String = summary
        .chars()
        .filter(|c| !c.is_control() || c.is_whitespace())
        .collect();
    let summary: String = clean
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .take(80)
        .collect();
    if summary.is_empty() {
        Err(FunctionCallError::RespondToModel(
            "summary must not be empty".into(),
        ))
    } else {
        Ok(summary)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn assignment_metadata_is_single_line_bounded_and_rejects_blank_controls() {
        assert_eq!(
            normalize_summary("Review\n retry\tvalidation").unwrap(),
            "Review retry validation"
        );
        assert_eq!(
            normalize_summary(&"検".repeat(100))
                .unwrap()
                .chars()
                .count(),
            80
        );
        assert!(normalize_summary("\u{1b} \u{1b}").is_err());
    }
}
