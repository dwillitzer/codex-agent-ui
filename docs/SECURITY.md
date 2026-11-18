# Agent UI fork security boundaries

Version: **0.170.0-agent-ui.1** | Updated: 2026-11-18

## Trust model

The existing subscription/API authentication remains authoritative. A client-version query
affects catalog compatibility, not account identity or permission to execute a model request.
The backend still authorizes requests; the TUI still validates selections against its catalog.
No credential, endpoint, approval, sandbox, or account-access setting is widened by this change.

```mermaid
flowchart LR
    UI[Model selection] --> Catalog[Identity-scoped catalog]
    Catalog --> Request[Authenticated request]
    Request --> Server[Backend authorization]
```

## Data handling

Credentials stay in existing credential files or stores. Local evaluation scripts load them
in process without printing them, and publish only model names, statuses, and hashes.
Demo credentials, raw request/session logs, and unredacted recordings are excluded from Git.
The shareable recording crops unrelated application content and masks host notifications.

SHA-256 evidence hashes identify source, executable, and media artifacts; they are integrity
references, not signatures or proof of production certification. This fork introduces no new
cryptographic scheme and makes no compliance-certification claim.

Child approvals remain subject to existing user/policy decisions. Inspecting a parent-managed
child does not grant direct-input permission. Turn interruption is distinct from process or
agent-lifetime termination.

## Version history

0.170.0-agent-ui.1 labels the fork explicitly and corrects the existing catalog version input.
See the repository's root SECURITY.md for upstream vulnerability-reporting policy.
