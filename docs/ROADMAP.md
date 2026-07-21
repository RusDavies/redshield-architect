# Roadmap

## Milestone 0: Product Definition

- Define the product thesis and first workflow.
- Choose the initial application stack.
- Define the canonical model package shape.
- Define proposal/review mechanics for AI-suggested model changes.
- Define the first enterprise-architecture schema/UI boundary.
- Define privacy and provider-selection defaults.

## Milestone 1: Thin Prototype

- Create a model package with requirements, actors, use cases, and trace links.
- Validate model package structure from a CLI.
- Render a use-case diagram from accepted model data.
- Show traceability between a requirement and model elements.
- Save deterministic text-backed files.
- Preserve native portfolio facts in the package without requiring full portfolio UI.
- Define lifecycle fields and milestone semantics for applications, products, services, and technology components.
- Define package-level portfolio view kinds beyond UML.
- Add read-only CLI/workbench summaries for portfolio objects.
- Add generated/read-only rendering for the first portfolio lifecycle roadmap.
- Add search and filters to read-only portfolio summaries.
- Add richer generated lifecycle-roadmap layout semantics for timeline buckets, swimlanes, and target-state transitions.
- Decide that first portfolio summary filters remain temporary/local until a named saved-view or query contract is justified by real use.
- Decide that generated lifecycle-roadmap semantics remain renderer-owned defaults until a named roadmap presentation contract is justified by real use.

## Milestone 2: Proposal Review Workflow

- Store proposed model operations separately from canonical state.
- Render proposal summaries, operation diffs, validation results, and provenance.
- Apply accepted operations through the core validation path.
- Reject or defer individual operations.
- Commit accepted model changes when configured.

## Milestone 3: Workbench UX

- Build source-note, model, proposal, diagram, and traceability views.
- Support basic editing for requirements and model elements.
- Improve diagram ergonomics enough that the model does not feel like homework with a viewport.
- Add export of diagram images and readable summaries.

## Milestone 4: Compatibility And Packaging

- Evaluate XMI, PlantUML, Mermaid, Structurizr DSL, SVG, and PDF export priorities.
- Define ArchiMate import/export mapping after the native package contract is stable.
- Package for common Linux desktop environments.
- Add project templates and examples.
- Document provider integration boundaries.
