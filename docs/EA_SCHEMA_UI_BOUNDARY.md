# Enterprise Architecture Schema/UI Boundary

RedShield's first enterprise architecture layer is a schema capability before it is a full workbench UI. The package should be able to preserve reviewable EA facts early, while the MVP UI stays focused on requirements, model elements, diagrams, traceability, and proposal review.

This keeps the model honest without turning the first prototype into a portfolio-management product. Ambition is good. Accidental suite-building is how software goes to die in a procurement portal.

## First-Schema Concepts

These concepts belong in the first schema because they are common architecture facts that need stable IDs, provenance, review, and links back to solution architecture:

- `business_capability`: the business or operating ability supported by model elements, services, applications, or initiatives.
- `portfolio_application`: an application or product in the estate, distinct from a UML component used inside a solution model.
- `portfolio_service`: a provided or consumed service that can connect capabilities, applications, components, and implementation work.
- `technology_component`: a concrete runtime, framework, platform, database, tool, protocol, library family, or infrastructure component.
- `technology_standard`: an approved, tolerated, discouraged, banned, or emerging technology policy/standard.
- `organization_unit`: a team, department, vendor group, or accountable organizational area.
- `owner`: a durable ownership role or accountable party reference.
- `lifecycle_milestone`: a dated or named lifecycle event such as launch, deprecation, end of support, migration, or retirement.
- `roadmap_item`: planned change that moves architecture from current state toward target state.
- `risk`: architecture, security, operational, delivery, obsolescence, compliance, or continuity risk.
- `control`: a required mitigation, governance rule, review gate, policy control, or compliance control.
- `governance_decision`: durable approval, waiver, exception, standardization, or direction-setting decision.
- `data_source`: source system or evidence feed for discovered/imported architecture facts.

These are deliberately metamodel-neutral. RedShield can map to ArchiMate, UML profiles, Structurizr, or customer vocabulary later without making those standards the internal source of truth.

The ArchiMate posture is documented in [ArchiMate Alignment Decision](ARCHIMATE_ALIGNMENT.md): RedShield remains native internally and treats ArchiMate as a later mapping/export target.

Lifecycle field semantics are documented in [Portfolio Lifecycle Semantics](PORTFOLIO_LIFECYCLE.md): lifecycle data is native RedShield package state for applications, products represented as applications, services, and technologies.

Portfolio view semantics are documented in [Portfolio View Semantics](PORTFOLIO_VIEWS.md): capability maps, application landscapes, lifecycle roadmaps, risk heatmaps, and dependency maps are package-level view kinds before they are full MVP UI surfaces.

## MVP UI Boundary

The MVP UI should not try to provide full CRUD, landscape dashboards, survey workflows, import wizards, or enterprise reporting for every portfolio object. The first visible UI should stay close to the core RedShield workflow:

- requirements list and detail editing
- semantic model element inspection/editing
- diagram view editing
- proposal operation review
- traceability inspection
- basic validation output

Portfolio facts may appear in MVP only where they clarify a selected model element or proposal:

- show linked capabilities, services, technologies, risks, and owners in an inspector summary
- show lifecycle/criticality badges for selected model elements when present
- include portfolio refs in proposal summaries and validation messages
- allow download/apply of portfolio proposal operations through the same proposal path as other model changes

Full portfolio-object browsing, editing, relationship graphing, import management, governance workflow, dashboards, heatmaps, surveys, and reporting are explicitly deferred.

## First-Schema But Not MVP UI

These concepts should be schema-supported now but mostly hidden from the first UI:

- `portfolio_application`: preserve/import app inventory facts, but do not build an application portfolio dashboard yet.
- `organization_unit` and `owner`: support references and provenance, but defer org charts, ownership workflows, and responsibility matrices.
- `lifecycle_milestone`: preserve lifecycle dates/events, but defer roadmap/timeline UI.
- `roadmap_item`: preserve target-state planning facts, but defer roadmap planning UI.
- `control`: preserve governance/compliance/control facts, but defer control management.
- `governance_decision`: preserve decisions and waivers, but defer decision-board workflow.
- `data_source`: preserve provenance/source identity, but defer import/source-management UI.

These concepts may still be authored by proposal JSON, imports, CLI tools, or future adapters. The lack of MVP UI is a product-scope decision, not a schema rejection.

## MVP-Visible Subset

These concepts can be visible early because they help solution architects immediately:

- `business_capability`, as a linked label or inspector section
- `portfolio_service`, as provided/consumed service context
- `technology_component` and `technology_standard`, as technology context and standard-state hints
- `risk`, as selected-element risk context
- `owner`, as accountable/technical/business ownership context

The UI should treat these as contextual facts attached to model elements and proposals, not as a separate enterprise-architecture workspace.

## Validation Direction

The first validation boundary should remain conservative:

- object IDs are globally unique across the package
- portfolio object kind/status/lifecycle/criticality/standard-state values are bounded
- `relatedElementRefs` must point to existing model elements
- local `ownerRefs`, `capabilityRefs`, `technologyRefs`, `riskRefs`, and lifecycle milestone refs must point to portfolio objects of the expected kind
- `package:<projectId>#<portfolioObjectId>` refs resolve locally when `<projectId>` matches the current manifest, resolve against `imports/imports.json` when that package is declared, and otherwise warn as unresolved external portfolio refs
- `source:<sourceId>#<externalObjectId>` refs warn as unresolved external portfolio refs for imported/source-system identity
- unqualified missing portfolio refs fail validation as local package mistakes
- malformed qualified refs fail validation
- proposal operations reject no-op portfolio updates
- proposal application writes portfolio changes through package validation

The same validation applies to model-element `architecture` mappings for owners, technologies, risks, capabilities, services, and lifecycle milestones. Imports, cross-package references, and external estate systems may still need legitimate references that are not locally materialized, but they must now use the explicit `package:` or `source:` qualifier instead of looking like a misspelled local ID.

## Deferred Work

Deferred work belongs in backlog, not the first UI:

- read-only CLI/workbench summaries for portfolio objects
- stricter reference validation once import and cross-package semantics are defined
- richer portfolio view rendering for capability maps, application landscapes, dependency maps, lifecycle roadmaps, risk heatmaps, and target-state transition views
- ArchiMate alignment/mapping decision
- import/export strategy for portfolio objects and discovered facts
- governance/control workflow and enterprise reporting
