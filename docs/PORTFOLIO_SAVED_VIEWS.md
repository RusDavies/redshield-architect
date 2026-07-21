# Portfolio Saved Views And Queries

This document defines the first named portfolio saved-view/query contract for a future package version. It is a design target, not implemented prototype schema yet. Current CLI and workbench filters remain temporary local controls until implementation work deliberately adds this contract to validation and proposal application.

The contract exists so saved summary filters do not grow into invisible dashboard state. A saved portfolio view should be a named, reviewable package object with bounded query fields, stable output expectations, and clear provenance.

## Storage Shape

When implemented, saved portfolio views should live in a dedicated package file:

```text
redshield/
  views/
    portfolio-views.json
```

The file should use the normal package version envelope:

```json
{
  "schemaVersion": "0.1.0",
  "views": []
}
```

Do not store saved portfolio summary filters inside `views/diagrams.json`. Diagram views answer "what should render on this canvas or export?" Saved portfolio queries answer "which portfolio facts should this named summary include?" They may feed each other later, but they are not the same object. Small mercy, enormous future diff reduction.

## Saved View Object

A saved portfolio view should contain:

- `id`: stable package-local ID such as `portfolio-view.active-critical-apps`.
- `title`: human-readable name.
- `description`: optional short purpose.
- `scope`: one of `portfolio_summary`, `portfolio_view_source`, or `export_set`.
- `resultKinds`: bounded portfolio object kinds the view may return.
- `query`: bounded filter expression.
- `sort`: optional deterministic sort order.
- `columns`: optional summary fields to show.
- `presentation`: optional display hints that do not change query truth.
- `provenance`: optional source references and creator/review notes.

The first implementation should support only `scope: "portfolio_summary"` unless a renderer or exporter needs the others.

## Query Fields

The initial `query` object should be deliberately boring:

- `text`: case-insensitive search across ID, name, description, lifecycle state, criticality, standard state, tags, and external-reference labels.
- `kinds`: portfolio object kinds.
- `statuses`: portfolio object statuses.
- `lifecycleStates`: lifecycle states.
- `criticalities`: `low`, `medium`, `high`, or `critical`.
- `standardStates`: technology standard states.
- `tags`: tags that must be present.
- `ownerRefs`: referenced owners.
- `capabilityRefs`: referenced capabilities.
- `technologyRefs`: referenced technologies.
- `riskRefs`: referenced risks.
- `relatedElementRefs`: linked solution-model elements.

Every populated array uses OR semantics within the field. Different fields combine with AND semantics. This keeps saved views explainable in review and cheap to validate. More advanced boolean groups can wait until someone has evidence they are worth the complexity and not just trying to smuggle a query language into a modeling tool.

## Sorting And Columns

Supported `sort` fields should start with:

- `name`
- `kind`
- `status`
- `lifecycleState`
- `criticality`
- `standardState`

Supported `columns` should start with:

- `id`
- `kind`
- `name`
- `status`
- `lifecycleState`
- `criticality`
- `standardState`
- `ownerRefs`
- `capabilityRefs`
- `technologyRefs`
- `riskRefs`
- `relatedElementRefs`
- `tags`

Sorts and columns are presentation/query-output metadata. They do not alter portfolio object truth.

## Presentation Hints

The first `presentation` object may include:

- `density`: `compact`, `comfortable`, or `detailed`.
- `groupBy`: one of `kind`, `status`, `lifecycleState`, `criticality`, `standardState`, `owner`, or `capability`.
- `showCounts`: boolean.

Presentation hints are intentionally less powerful than render profiles. If a view needs real diagram or export styling, use a diagram view, render profile, or future roadmap presentation contract instead.

## Proposal Operations

Saved portfolio views should be created and changed through typed proposal operations, not silent UI writes:

- `create_portfolio_saved_view`
- `update_portfolio_saved_view`
- `remove_portfolio_saved_view`

The validator should reject unknown query fields, unsupported enum values, duplicate IDs, empty titles, no-op updates, and references to missing owners, capabilities, technologies, risks, or model elements when those reference targets are local package objects.

## Boundaries

Saved portfolio views should not:

- materialize hidden copies of matching portfolio objects
- bypass package validation
- become a general dashboard builder
- store provider prompts, AI chat history, or private source text
- imply that portfolio filters are globally shareable outside the project repository

Sharing means the saved view is committed as package metadata and can be reviewed, diffed, imported, or exported by RedShield-aware tooling.
