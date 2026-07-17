# Public Threat Model

## Scope

This threat model covers the early RedShield Architect concept: a local-first requirements and architecture modeling workbench with optional AI-assisted proposal generation.

## Assets

- source notes, requirements, architecture text, and model data
- traceability links and decision evidence
- AI proposal packages and provenance
- local project files and Git history
- provider configuration and approval decisions
- exports and support bundles

## Trust Boundaries

- local user project files
- desktop application process
- optional AI provider or agent integration
- Git repository
- import/export files
- generated support bundles or examples

## Threats And Controls

| ID | Threat | Impact | Baseline Control | Status |
| --- | --- | --- | --- | --- |
| TH-1 | Sensitive source material is sent to an external AI provider without clear approval. | Privacy breach, contractual exposure, loss of trust. | Explicit provider selection and visible provenance before AI use. | Planned |
| TH-2 | AI output silently mutates canonical model files. | Unreviewed requirements, bad architecture state, poisoned traceability. | AI output is stored as reviewable typed proposals. | Planned |
| TH-3 | Secrets are included in prompts, logs, examples, exports, or support bundles. | Credential exposure. | Default secret-pattern redaction and explicit sensitive-file overrides. | Planned |
| TH-4 | Importers accept malformed or malicious model files. | Local crash, data corruption, unsafe generated output. | Schema validation, bounded parsing, and import previews. | Planned |
| TH-5 | Generated diagrams or reports misrepresent canonical model state. | Bad design decisions or false review evidence. | Views derive from model IDs and validation state. | Planned |
| TH-6 | Proposal provenance is incomplete or forged. | Weak auditability and poor trust in AI-suggested changes. | Store source references, provider metadata, validation result, and review state. | Planned |
| TH-7 | Project exports contain unrelated private files. | Accidental data disclosure. | Export allowlists and previewable bundles. | Planned |

## Privacy Defaults

The product should not assume that architecture notes, requirements, code summaries, or model data are safe to send to a third party. External AI use should be explicit, configurable, and visible in proposal metadata.

## Open Questions

- Which provider configuration format should be used for the first prototype?
- How much automatic redaction is appropriate before it becomes misleading?
- Which support-bundle and example-export formats are safe enough for early users?
- How should project-level policy interact with future team or enterprise controls?
