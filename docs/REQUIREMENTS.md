# Prototype Requirements

## Functional Requirements

| ID | Requirement | Acceptance Criteria | Priority |
| --- | --- | --- | --- |
| FR-1 | Create and edit requirements. | A user can create, update, delete, and persist requirements with stable IDs, title, body, status, and acceptance criteria. | Must |
| FR-2 | Create practical UML model elements. | A user can create at least use-case, actor, class, component, activity, and sequence-oriented model objects in the canonical model. | Must |
| FR-3 | Render a diagram view from model data. | The prototype can render at least one use-case diagram from accepted model objects. | Must |
| FR-4 | Store canonical data in deterministic text files. | Re-saving unchanged model data produces stable diffs and readable Git changes. | Must |
| FR-5 | Maintain trace links. | A user can link requirements to model elements and implementation tasks. | Must |
| FR-6 | Validate model consistency. | The app reports missing references, duplicate IDs, invalid trace links, and unsupported object shapes. | Must |
| FR-7 | Review AI-suggested changes as proposals. | AI output is stored as a proposal with operations, rationale, source references, validation status, and review state. | Must |
| FR-8 | Apply accepted proposal operations. | Accepted operations update canonical model files through the same validation path as human UI edits. | Must |
| FR-9 | Export readable artifacts. | The app can export at least a diagram image and a human-readable model summary. | Should |
| FR-10 | Provide a CLI validation path. | A command can validate a model package without opening the UI. | Should |

## Non-Functional Requirements

| ID | Requirement | Acceptance Criteria | Priority |
| --- | --- | --- | --- |
| NFR-1 | Linux-first. | The primary development and packaging path supports common Linux desktop environments. | Must |
| NFR-2 | Local-first. | Core modeling workflows work without a hosted service. | Must |
| NFR-3 | Git-friendly. | Model files are deterministic and reviewable in pull requests. | Must |
| NFR-4 | Provider-neutral AI integration. | AI proposal mechanics do not depend on a single model provider or agent runtime. | Must |
| NFR-5 | Privacy-conscious defaults. | Source material is not sent to external AI providers without explicit project/provider configuration. | Must |
| NFR-6 | Extensible import/export. | XMI, PlantUML, Mermaid, Structurizr DSL, SVG, and PDF can be added as import/export targets without becoming the internal source of truth. | Should |

## Security And Privacy Requirements

| ID | Requirement | Acceptance Criteria | Priority |
| --- | --- | --- | --- |
| SEC-1 | AI access is explicit. | The project records which provider is used before source material is sent to AI. | Must |
| SEC-2 | Proposals preserve provenance. | AI-suggested model elements link back to approved source references. | Must |
| SEC-3 | Secrets are excluded by default. | Common secret patterns are blocked from prompts, exports, and support bundles unless explicitly overridden. | Must |
| SEC-4 | Canonical changes are reviewable. | AI proposals cannot directly mutate canonical model files without review/application. | Must |
| SEC-5 | Audit-relevant actions are visible. | Accepted proposals, validation state, and generated commits can be traced. | Should |
