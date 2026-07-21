# Portfolio Lifecycle Semantics

RedShield portfolio lifecycle fields describe architecture posture, not project-task workflow. They apply to portfolio applications, products represented as portfolio applications, portfolio services, technology components, and technology standards.

The model keeps lifecycle facts deterministic and proposal-reviewed so architects can answer ordinary questions without inventing a separate portfolio suite: what exists, what is planned, what is aging out, what is replacing it, and which milestone makes the change real.

## Lifecycle Fields

Portfolio objects keep the existing flat `lifecycleState` for simple filtering and compatibility. New lifecycle detail lives in `lifecycle`:

- `state`: current architectural posture: `idea`, `planned`, `active`, `deprecated`, `retiring`, or `retired`.
- `phase`: human-facing local phase label, such as `thin prototype`, `pilot`, `production`, `standardizing`, or `sunset`.
- `currentFrom`: date the current lifecycle posture became true.
- `targetState`: intended next posture: `planned`, `active`, `deprecated`, `retiring`, or `retired`.
- `targetDate`: expected date for the target posture.
- `endOfSupportDate`: date support is expected to stop.
- `retirementDate`: date the object is expected to leave the estate or supported model.
- `milestoneRefs`: references to `lifecycle_milestone` portfolio objects.
- `notes`: short lifecycle context that does not deserve a new object.

Dates use `YYYY-MM-DD` so package diffs stay stable and sortable. The schema does not try to solve calendars, confidence, partial dates, or dependency scheduling yet. Tiny mercy.

## State Semantics

Use `idea` for a concept that is being discussed but has no approved delivery intent.

Use `planned` when there is a real intent to introduce, migrate to, replace, or standardize something, but it is not yet operating as accepted architecture.

Use `active` when the application, product, service, or technology component is part of the accepted current architecture.

Use `deprecated` when the object is still present but should not be selected for new work.

Use `retiring` when there is an active removal, replacement, or migration path.

Use `retired` when the object is no longer part of the supported current architecture. Keep retired facts if they explain decisions, migrations, risks, or trace history.

## Milestone Semantics

Use `lifecycle_milestone` objects for reviewable lifecycle events, not for every delivery task. Good milestone examples:

- launch or first production use
- pilot start or end
- standard approval
- deprecation notice
- end of support
- migration complete
- retirement complete

Milestone objects should have stable IDs and source references. Other portfolio objects refer to them through `lifecycle.milestoneRefs`. This keeps timeline facts reusable across an application, its provided service, and the technology components supporting it.

## Product And Application Boundary

RedShield does not add a separate `product` kind yet. Products are represented as `portfolio_application` objects when they are things an architecture estate owns, operates, buys, or delivers. If a later adapter needs to distinguish commercial product, internal application, SaaS tenant, or platform product, that should be a profile or mapping decision rather than a new MVP axis.

## Technology Boundary

Use `technology_component` for concrete technology used by architecture: frameworks, runtimes, libraries, infrastructure services, protocols, databases, or tools.

Use `technology_standard` for policy posture around a technology, such as approved, tolerated, discouraged, banned, or emerging. Lifecycle describes the standard's posture over time; `standardState` describes the governance stance.

## Validation

Current validation is intentionally conservative:

- lifecycle states and target states are bounded
- lifecycle date fields must use `YYYY-MM-DD`
- milestone refs must be non-empty strings
- referenced milestone objects are not required to be local yet

Cross-package imports and external portfolio systems may legitimately provide milestone refs before the local package materializes those milestones. Stricter reference validation belongs after import semantics are clearer.
