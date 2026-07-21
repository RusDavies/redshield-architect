# Portfolio View Semantics

RedShield portfolio views are generated views over portfolio facts and solution-model links. They are not separate canonical state and they do not turn the MVP into a portfolio dashboard product. The canonical facts stay in `model/portfolio.json`, `model/elements.json`, `model/relationships.json`, requirements, trace links, and proposals.

The first package contract supports five non-UML portfolio view kinds:

- `capability_map`
- `application_landscape`
- `lifecycle_roadmap`
- `risk_heatmap`
- `dependency_map`

These view kinds may appear in `views/diagrams.json` before the workbench renders them. That lets imports, agents, CLI summaries, and future UI work agree on names and references without silently inventing private formats. Thrilling governance. Somehow useful.

## Common Rules

Portfolio views use the same `DiagramView` envelope as UML views:

- `id`: stable view ID.
- `title`: human-readable title.
- `viewKind`: one of the portfolio view kinds.
- `portfolioRefs`: portfolio objects included in the view.
- `modelRefs`: optional model elements included for solution-architecture context.
- `layout`: optional canvas layout metadata.

The historical layout field name `modelRef` is still used for nodes. In portfolio views, a layout node may refer to either a model element or a portfolio object, as long as the object is listed in `modelRefs` or `portfolioRefs`.

## Summary Filters

Portfolio summary filters are temporary local controls for narrowing the current read-only summary. The CLI search argument and workbench search/kind/lifecycle controls do not create saved package state and are not shareable links.

Do not turn these first filters into saved views yet. When repeated use justifies durable sharing, use the bounded contract in [Portfolio Saved Views And Queries](PORTFOLIO_SAVED_VIEWS.md): a named saved query with explicit fields, deterministic validation, and proposal-reviewed changes. Otherwise RedShield gets stealth dashboard configuration before the model contract deserves it, which is how simple tools grow tentacles and invoice you for them.

## Capability Map

`capability_map` shows business capabilities and the applications, services, technologies, risks, owners, or model elements that support them.

Primary refs:

- `business_capability`
- `portfolio_application`
- `portfolio_service`
- related model elements

Useful questions:

- Which capabilities does this solution support?
- Which applications and services sit under a capability?
- Which capabilities carry high architectural risk?

MVP posture: read-only summary or generated diagram only. Do not build capability-planning CRUD yet.

## Application Landscape

`application_landscape` shows portfolio applications and products represented as applications, grouped by capability, owner, lifecycle state, technology, criticality, or service boundary.

Primary refs:

- `portfolio_application`
- `portfolio_service`
- `owner` or `organization_unit`
- `technology_component`
- `technology_standard`

Useful questions:

- What applications or products exist in the estate?
- Which services do they provide or consume?
- Which technologies and standards matter for them?

MVP posture: preserve and validate view membership; defer dashboard interactions, surveys, and app-inventory workflows.

## Lifecycle Roadmap

`lifecycle_roadmap` shows lifecycle posture, target states, dates, and milestones for applications, services, and technologies.

Primary refs:

- `portfolio_application`
- `portfolio_service`
- `technology_component`
- `technology_standard`
- `lifecycle_milestone`
- `roadmap_item`

Useful questions:

- What is active, planned, deprecated, retiring, or retired?
- Which target dates and milestones are driving change?
- Which technologies need migration before support or retirement dates?

MVP posture: generated/read-only SVG rendering is supported for the first lifecycle roadmap view. Full timeline editing is later work.

## Risk Heatmap

`risk_heatmap` shows portfolio risks and controls grouped by severity, criticality, capability, application, service, technology, owner, or lifecycle state.

Primary refs:

- `risk`
- `control`
- `portfolio_application`
- `portfolio_service`
- `technology_component`
- `business_capability`
- related model elements

Useful questions:

- Which architecture areas carry the most risk?
- Which risks touch critical capabilities or retiring technologies?
- Which controls mitigate the risk and where is evidence stored?

MVP posture: no heatmap UI yet. Keep the view kind stable so later reporting can be generated from reviewed data.

## Dependency Map

`dependency_map` shows dependencies between capabilities, applications, services, technologies, and model elements.

Primary refs:

- `portfolio_application`
- `portfolio_service`
- `technology_component`
- `business_capability`
- related model elements and relationships

Useful questions:

- What depends on this service or technology?
- Which capability is affected by changing a component?
- Which solution-model relationships explain a portfolio dependency?

MVP posture: generated/read-only only until portfolio relationship semantics are stronger.

## Roadmap Layout Semantics

Lifecycle roadmap rendering now derives a small set of read-only layout semantics from portfolio object data:

- timeline buckets come from lifecycle target, retirement, end-of-support, or current-from dates and render as a visible scale
- swimlanes group objects by portfolio kind so applications, services, technologies, and milestones do not collapse into one pile
- target-state callouts render when an object has a structured `lifecycle.targetState` or `lifecycle.targetDate`
- milestone links still come from `lifecycle.milestoneRefs` and remain dashed when both endpoint objects are included

These semantics are generated view behavior, not canonical planning state. They do not edit portfolio objects, infer missing dates, persist layout, or introduce a roadmap-planning workflow.

Do not promote the first generated roadmap semantics into saved/customizable package metadata yet. Timeline bucket rules, swimlane definitions, target-state label placement, milestone-link styling, and roadmap layout presets should stay renderer-owned defaults until repeated use shows which controls architects actually need to preserve, review, share, or import/export.

When durable customization becomes necessary, it should use the bounded contract in [Portfolio Roadmap Presentation](PORTFOLIO_ROADMAP_PRESENTATION.md). That contract should be reviewable package metadata with deterministic validation and clear separation from portfolio object truth; it should not hide renderer preferences inside ad hoc diagram state.

## Validation

Current validation is deliberately small:

- view kinds are bounded
- `modelRefs` must point to model elements
- `portfolioRefs` must point to portfolio objects
- layout nodes must reference listed `modelRefs` or `portfolioRefs`
- relationship connector layout still points to canonical model relationships

The schema does not yet encode saved grouping preferences, custom swimlane definitions, heatmap buckets, dependency edge semantics, or portfolio relationship kinds. Those should be added only when the renderer, workbench, or import/export adapter needs durable user-controlled semantics.

## First Renderer

The first generated portfolio renderer is `render-lifecycle-roadmap`.

It renders `lifecycle_roadmap` views to SVG through Graphviz, using the same generated-artifact boundary as the use-case renderer. The renderer:

- colors included portfolio objects by `lifecycleState`
- derives a visible timeline scale from lifecycle dates
- derives swimlanes from portfolio object kinds
- renders target-state callouts from structured lifecycle metadata
- renders `lifecycle_milestone` objects as milestone nodes
- includes target dates from structured lifecycle metadata when present
- draws dashed milestone links from `lifecycle.milestoneRefs` when both endpoint objects are included in the view

The renderer does not edit portfolio objects, infer roadmap dates, create missing milestones, or persist layout metadata. That restraint is not glamour, but it is how the package stays deterministic.
