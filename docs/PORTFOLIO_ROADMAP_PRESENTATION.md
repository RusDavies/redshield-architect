# Portfolio Roadmap Presentation

This document defines the first named lifecycle-roadmap presentation/layout contract for a future package version. It is a design target, not implemented prototype schema yet. The current `render-lifecycle-roadmap` command keeps deriving timeline buckets, swimlanes, target-state callouts, and milestone links from portfolio facts and renderer defaults.

The contract exists for the point where users need a saved roadmap presentation that is stable enough to review, share, import, export, and reproduce. Until then, renderer defaults are preferable to saved knobs. Saved knobs are how a simple roadmap becomes a cockpit made of checkboxes, and nobody deserves that before breakfast.

## Storage Shape

When implemented, roadmap presentation presets should live in a dedicated package file:

```text
redshield/
  views/
    roadmap-presentations.json
```

The file should use the normal package version envelope:

```json
{
  "schemaVersion": "0.1.0",
  "presentations": []
}
```

Do not store roadmap presentation presets in `views/diagrams.json`. Diagram views identify the objects included in a view and any generic canvas layout. Roadmap presentations describe how lifecycle-roadmap renderers group, label, and annotate those objects.

## Presentation Object

A roadmap presentation should contain:

- `id`: stable package-local ID such as `roadmap-presentation.default-lifecycle`.
- `title`: human-readable name.
- `description`: optional short purpose.
- `appliesToViewKinds`: initially only `lifecycle_roadmap`.
- `timeline`: timeline bucket and date-label rules.
- `swimlanes`: lane grouping rules.
- `targetStates`: target-state annotation rules.
- `milestones`: milestone visibility and link rules.
- `styling`: bounded visual hints that do not replace render profiles.
- `provenance`: optional source references and creator/review notes.

The first implementation should support one active presentation per rendered lifecycle-roadmap view. Multiple presentation overlays can wait until the product has actual evidence that architects need them, not merely because "array of presets" sounds wonderfully enterprise.

## Timeline Rules

The initial `timeline` object should support:

- `bucketSource`: one of `targetDate`, `retirementDate`, `endOfSupportDate`, `currentFrom`, or `auto`.
- `bucketGranularity`: `month`, `quarter`, `half_year`, or `year`.
- `rangeStart` and `rangeEnd`: optional `YYYY-MM-DD` bounds.
- `includeUndatedBucket`: boolean.
- `dateLabelFormat`: `date`, `month`, `quarter`, or `year`.

`auto` should preserve the current renderer behavior: pick the most useful available lifecycle date in a deterministic order and derive visible buckets from included objects.

## Swimlane Rules

The initial `swimlanes` object should support:

- `groupBy`: one of `portfolioKind`, `lifecycleState`, `criticality`, `owner`, `capability`, `technology`, or `none`.
- `order`: optional ordered lane keys.
- `includeEmptyLanes`: boolean.
- `fallbackLaneTitle`: label for objects that do not match a configured lane.

Renderer implementations must keep lane membership explainable from canonical portfolio fields. A swimlane rule should not become a hidden secondary classifier.

## Target-State Rules

The initial `targetStates` object should support:

- `showCallouts`: boolean.
- `showTargetDates`: boolean.
- `showNoChangeTargets`: boolean.
- `states`: optional lifecycle states to annotate.

Target-state display is presentation metadata only. The actual target state and dates remain in `model/portfolio.json`.

## Milestone Rules

The initial `milestones` object should support:

- `showMilestoneNodes`: boolean.
- `showMilestoneLinks`: boolean.
- `linkStyle`: `solid`, `dashed`, or `dotted`.
- `includeUnreferencedMilestones`: boolean.

Milestone links still come from `lifecycle.milestoneRefs`. Presentation metadata may hide or style those links, but it must not invent milestone relationships.

## Styling Hints

The initial `styling` object may include:

- `density`: `compact`, `comfortable`, or `detailed`.
- `colorBy`: one of `lifecycleState`, `criticality`, `standardState`, `portfolioKind`, or `none`.
- `showLegend`: boolean.
- `showTimelineScale`: boolean.

Deep shape, icon, asset, and renderer selection belongs in render profiles. Roadmap presentation should control roadmap-specific grouping and annotations, not the whole drawing system.

## Proposal Operations

Roadmap presentations should be created and changed through typed proposal operations:

- `create_roadmap_presentation`
- `update_roadmap_presentation`
- `remove_roadmap_presentation`
- `assign_roadmap_presentation`

`assign_roadmap_presentation` should attach a presentation ID to a lifecycle-roadmap diagram view or renderer request without mutating portfolio objects.

The validator should reject unknown fields, unsupported enum values, duplicate IDs, empty titles, invalid dates, no-op updates, assignments to non-roadmap view kinds, and references to missing presentation or diagram IDs.

## Boundaries

Roadmap presentations should not:

- edit portfolio lifecycle facts
- infer or create missing roadmap dates
- create milestone objects or milestone references
- replace generic diagram layout metadata
- become a general report designer
- store AI prompts, provider output, or private source text

Sharing means the presentation preset is committed as package metadata and can be reviewed, diffed, imported, or exported by RedShield-aware tooling.
