# Portfolio Subtype Profiles

## Decision

Do not add first-class product/application/service subtype profiles to the
prototype schema yet.

Keep the current bounded portfolio kinds:

- `portfolio_application`
- `portfolio_service`
- `technology_component`
- `technology_standard`
- supporting governance, ownership, lifecycle, risk, and capability kinds

Use existing package fields for current nuance:

- `tags` for lightweight user-visible classification
- `lifecycle` and `standardState` for architectural posture
- `sourceRefs` and `externalReferences` for imported/source-system identity
- `package:` and `source:` qualified refs for cross-package or source-system
  identity

Add a bounded subtype/profile contract only after import/export work proves
which distinctions must survive round trips.

## Rationale

The current package model already distinguishes the major architectural object
roles RedShield needs for the prototype. Adding a `product` kind or an open
`subtype` string now would make the model look richer while weakening
interoperability, validation, and UI clarity.

The hard questions are not whether someone might say "application", "product",
"platform", "SaaS", "service", or "component". They will. Humans are wonderfully
talented at naming the same thing seven ways before lunch.

The hard questions are:

- which distinctions affect validation or proposal operations?
- which distinctions affect import/export mappings?
- which distinctions affect generated views?
- which distinctions are just local vocabulary?
- which distinctions must survive Git review and round-trip through another
  tool?

Those questions belong in the import/export matrix and adapter design, not in a
premature schema axis.

## Current Modeling Guidance

Represent a product as `portfolio_application` when it is an estate object the
architecture owns, operates, buys, delivers, or supports.

Represent a service as `portfolio_service` when the object describes a provided
or consumed architectural service boundary, regardless of whether it is backed
by an internal application, external SaaS product, platform capability, or
technology component.

Represent concrete runtime, framework, library, protocol, platform, database,
or infrastructure choices as `technology_component`.

Represent governance stance around a technology as `technology_standard`.

Use `tags` for non-normative labels such as:

- `commercial-product`
- `internal-application`
- `platform-product`
- `external-saas`
- `shared-service`
- `api-service`
- `data-service`
- `legacy`

Tags are useful for filtering and human readability. They must not become hidden
validation semantics.

## Future Profile Contract

If import/export evidence proves subtype profiles are needed, add a bounded
profile contract rather than a free-form subtype string.

A future profile should be package metadata such as:

```json
{
  "schemaVersion": "0.1.0",
  "profiles": [
    {
      "id": "profile.application.external-saas",
      "appliesToKinds": ["portfolio_application"],
      "title": "External SaaS application",
      "description": "Application estate object provided by an external SaaS supplier.",
      "mappingHints": [
        {
          "target": "archimate",
          "version": "4",
          "concept": "application_component",
          "confidence": "approximate"
        }
      ]
    }
  ]
}
```

A profile may provide:

- stable profile ID
- human-readable title and description
- bounded `appliesToKinds`
- optional import/export mapping hints
- optional validation hints that are explicit and testable
- optional presentation hints only when a view/export adapter needs them

A profile must not:

- change the canonical `kind` of an object
- silently relax required validation
- introduce tool-specific private semantics into core objects
- become the only place source provenance is recorded
- replace `portfolio_service`, `technology_component`, or
  `technology_standard`

## Triggers For Adding Profiles

Add profile metadata only when at least one of these is true:

- an import adapter needs to preserve a source-system type that cannot be safely
  represented by `kind`, refs, lifecycle, tags, and external references
- an export adapter needs deterministic mapping hints to avoid lossy or unstable
  output
- a generated view needs stable behavior that should survive package round trips
- validation needs an explicit, testable distinction inside one portfolio kind
- repeated real packages show the same subtype distinction affecting review
  decisions

Do not add profiles merely because another tool has a taxonomy or because a UI
filter would look tidy with another dropdown. Dropdowns are cheap. Semantics are
expensive.

## Import/Export Implications

The import/export matrix should record, for each target format:

- which target concepts map cleanly to RedShield portfolio kinds
- which target concepts need tags only
- which target concepts need profile IDs
- which target concepts are lossy or unsupported
- whether the mapping is reversible
- whether a profile would alter validation, rendering, or only export metadata

Until that matrix exists, subtype/profile fields remain deferred.

## Workbench Implications

The workbench should not expose subtype profile editing in the first MVP UI.

Near-term UI can:

- filter portfolio summaries by `kind`, lifecycle, criticality, standard state,
  owner, capability, and tags
- show tags in summaries and inspector context
- preserve imported source identity through source/external refs

Future UI may show profile badges or mapping hints after profile metadata exists.
It should still keep canonical kind visible so users do not confuse a local
profile with a different object type.

