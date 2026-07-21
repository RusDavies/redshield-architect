# Product Brief

## Product

RedShield Architect is a Linux-strong and web-ready requirements and architecture modeling workbench for technical teams that want text-backed models, useful UML views, traceability, and AI-assisted review without turning the model into opaque generated sludge.

## Audience

Initial users:

- software architects and senior engineers designing new systems
- small technical teams that already use Git for project artifacts
- teams that need browser access for review, collaboration, onboarding, or managed-device use
- consultants who need reviewable requirements, model, and decision evidence
- engineers who want AI assistance without losing provenance or approval control

Later users may include enterprise architecture, regulated engineering, security review, and modernization teams.

## Problem

Architecture tools often split into two unsatisfying camps:

- diagram-first tools that are pleasant for drawing but weak as durable semantic models
- heavyweight modeling suites that are powerful but cumbersome, proprietary, often awkward on Linux, and not always pleasant for lightweight web collaboration

AI adds another problem: generated suggestions can be useful, but they are risky when they mutate canonical project artifacts without typed operations, validation, provenance, or human review.

## Thesis

RedShield Architect should treat architecture work as a model lifecycle:

- requirements are structured objects
- UML elements are structured objects
- diagrams are views over those objects
- traceability is first-class
- AI output is a proposal, not project truth
- Git history can carry model evolution, review, and accountability
- the workbench can run locally on Linux first while keeping its UI and model-operation boundary portable to Web/SaaS

## Prototype Scope

The first prototype should support one end-to-end local path while avoiding architecture choices that would force a separate web product:

1. Create or import a small requirements set.
2. Propose model elements from approved source notes.
3. Review and accept or reject proposed operations.
4. Render at least one diagram view from accepted model data.
5. Show traceability from requirements to model elements and implementation tasks.
6. Export deterministic text-backed project files.

## Differentiators

- Linux-first local workflow
- Web/SaaS-ready workbench architecture
- deterministic model package suitable for Git
- practical UML rather than ceremony for its own sake
- AI proposal review as a core feature
- provenance and validation attached to model changes
- export-oriented design rather than storage lock-in

## Risks

- scope creep toward a full ALM suite
- weak diagram ergonomics despite strong model semantics
- overfitting to one AI provider or agent runtime
- local-only assumptions that make browser-hosted or SaaS deployment a rewrite
- insufficient import/export compatibility
- unclear packaging choices
