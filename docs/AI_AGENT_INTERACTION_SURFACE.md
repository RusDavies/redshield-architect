# AI Agent Interaction Surface

## Purpose

RedShield Architect treats AI agents as collaborators that propose typed,
reviewable model operations. The workbench should make that collaboration feel
native to architecture work instead of bolting a chat box beside a diagram and
hoping the user enjoys rummaging through mystery edits.

This contract defines the first workbench surface. It is provider-agnostic and
portable across Tauri desktop and future browser-hosted deployments.

## Surface Layout

The workbench should expose four coordinated regions:

- `Agent conversation`: a focused thread for prompts, clarifying questions,
  rationale, and user instructions scoped to the current project/package/view.
- `Context and provenance`: compact references showing which requirements,
  model elements, relationships, diagrams, source notes, files, or review
  packets the agent used.
- `Proposal tray`: a durable queue of proposed operation packages with status,
  validation results, warnings, and apply/reject controls.
- `Inspector integration`: selected model elements and relationships can show
  related agent rationale, proposed edits, conflicts, and provenance without
  hiding the normal human-facing inspector details.

The diagram canvas remains a semantic model workbench. Agent output should not
own canvas state directly; it proposes operations against model, view, render,
or review package contracts.

## Primary Workflow

1. User selects a project scope, view, element, relationship, requirement, or
   review packet.
2. User asks the agent for help in the conversation region.
3. Agent reports the context it plans to use before creating operations when the
   action may read sensitive local project material.
4. Agent returns rationale, assumptions, source/provenance references, and a
   proposal package.
5. Workbench validates the proposal package against local schemas and operation
   preconditions.
6. Proposal tray groups operations by intent and risk, with clear accept,
   reject, revise, and export actions.
7. Accepted proposals are applied through the same deterministic operation
   application path as local/manual proposal drafts.
8. Accepted changes remain normal Git-reviewable project file changes.

## Proposal States

Minimum visible states:

- `draft`: agent or user is still shaping the proposal
- `validating`: schemas and preconditions are running
- `needs_context`: the agent cannot proceed without a specific user choice or
  approved source
- `ready`: proposal is valid and can be reviewed
- `warning`: proposal is structurally valid but has review warnings
- `blocked`: proposal failed validation or violates policy
- `accepted`: user accepted the proposal
- `rejected`: user rejected the proposal
- `applied`: accepted proposal was applied to canonical files
- `superseded`: newer proposal replaced this one

## Review Controls

The proposal tray should provide:

- operation summary grouped by model object, view, render rule, requirement,
  trace link, or review packet
- diff preview against canonical package files
- schema validation status
- operation precondition status
- provenance links and source snippets when explicitly allowed
- rationale and assumptions
- risk flags for destructive changes, broad rewrites, external provider use, or
  redacted/omitted context
- accept all safe operations, accept selected operations, reject, revise, export,
  and save draft

Accepting a proposal must be a visible user action. The product should not allow
silent AI mutation of canonical project files.

## Conversation Model

The conversation region is a scoped workbench tool, not the canonical project
record. Durable outputs must be converted into one of these artifacts:

- proposal transaction
- source/provenance note
- decision or review packet
- requirement/model/view/render/trace operation
- explicit user-authored note

Conversation history may be persisted by an adapter, but it should not become
the only source for model truth.

## Context And Privacy Boundary

The agent surface must show what context is in scope before sensitive reads or
provider calls. Context categories:

- selected model elements and relationships
- selected diagram or saved view
- requirements and trace links
- proposal history and validation results
- source notes and imported package metadata
- user-selected local files or snippets
- review packets and control-profile decisions

External provider calls are opt-in per project or operation. Redaction rules
apply before prompts, logs, support bundles, or exported proposal evidence leave
the local trust boundary.

## Portable API Boundary

Workbench UI components should communicate with an `agentSession` capability
rather than a concrete provider SDK.

Required capabilities:

- list available agent providers or local adapters
- start/resume a scoped agent session
- declare intended context reads
- request user approval for sensitive context expansion
- submit prompt or instruction
- stream assistant text and structured progress
- produce proposal package drafts
- validate proposal packages
- save/load/export proposal drafts
- accept/reject selected operations through the normal proposal application path

Tauri desktop can back these capabilities with local filesystem and command
adapters. Browser-hosted deployments can back the same capabilities with server
APIs. UI code should not assume either shape.

## Non-Goals For The First Surface

- direct provider integration before the surface and operation envelope are
  defined
- multi-user collaboration semantics
- autonomous background mutation of project files
- training-data or telemetry policy
- full prompt-library/productivity-suite behavior
- generated code execution

## Implementation Notes

- The existing inspector tabs remain for human inspection and editing.
- The existing proposal tray becomes the natural place for proposal status and
  apply/reject controls.
- The navigator should feed scoped selections into both inspector and agent
  context.
- Raw prompts and provider responses should be collapsible or advanced details,
  not the primary review artifact.
- The default review artifact is a typed proposal package plus validation
  evidence, because that is what can be versioned, tested, and applied.

