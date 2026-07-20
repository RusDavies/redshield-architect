# Render Export Behavior

Render profiles choose how model elements appear in a diagram view. Exporters must preserve the semantic model IDs, connector geometry, labels, and reviewability of the view while handling renderer-specific assets predictably.

This document defines the first export contract for built-in renderers, image-backed renderers, SVG assets, and future custom HTML renderers across the browser workbench, Tauri desktop, SVG export, and PDF export paths.

## Goals

- Export diagrams without changing semantic model truth.
- Keep exported objects traceable to model element and relationship IDs.
- Preserve labels, ports, connector endpoints, selection-independent bounds, and route hints.
- Make missing, blocked, or unsupported render assets visible instead of silently dropping nodes.
- Keep unsafe renderer content from executing during export.

## Export Targets

### Browser Workbench

The browser workbench is the interactive preview surface. It should render the same resolved profile that a durable export would use, but it may show diagnostic placeholders for asset states that are not exportable yet.

Expected behavior:

- Built-in UML renderers use React/CSS for interactive display.
- `image.element` renders an image only when the asset status is `available` and the URI resolves inside the package asset root.
- `referenced`, `missing`, and `blocked` image assets show a visible placeholder with the asset status.
- `html.custom` stays disabled until a stricter trust policy exists.
- The inspector exposes the matched rule, renderer ID, asset ID/status, and model ID.

### Tauri Desktop

The Tauri shell should use the same TypeScript workbench renderer for preview, but file access and export should go through the backend/core boundary.

Expected behavior:

- Asset file reads are package-relative and mediated by the backend.
- Remote URLs are not fetched during export.
- Export requests pass the diagram ID, resolved profile ID, and target format to the backend.
- The backend validates the model package and render profile before producing files.
- Exported files should include enough metadata to identify the package, diagram, and source commit when available.

### SVG Export

SVG is the first structured diagram export target. It should remain useful for code review, browser viewing, documentation, and downstream conversion.

Expected behavior:

- Built-in UML renderers export as plain SVG shapes and text, not screenshots.
- Exported nodes include stable `id` or `data-model-id` metadata for the source model element.
- Exported connectors include stable metadata for the source relationship ID.
- Labels are emitted as text when practical, with font fallback chosen by the exporter.
- `image.element` embeds or links package-local image assets according to export mode:
  - self-contained mode embeds available PNG/JPEG/SVG assets as data URIs after validation.
  - linked mode writes package-relative links and records the dependency list.
- Referenced, missing, or blocked image assets export as deterministic placeholders.
- SVG image assets must be sanitized or isolated before embedding. Scripts, event handlers, `foreignObject`, external references, and remote loads are not allowed.
- `html.custom` does not export to SVG until a trusted renderer converts it into safe SVG primitives or a raster fallback.

### PDF Export

PDF export is a delivery format, not a canonical model format. It should be generated from the same validated intermediate export representation used for SVG where possible.

Expected behavior:

- The preferred path is model/profile -> sanitized export scene -> SVG -> PDF.
- Built-in renderers remain vector shapes and selectable text when the converter supports it.
- Image assets are embedded only after the same status, provenance, path, and sanitization checks used for SVG.
- Missing or blocked assets produce placeholders, not blank space.
- Export metadata should include diagram title, package ID, schema version, and source commit when available.
- PDF export must not execute custom HTML or SVG script content.

## Renderer Rules

### Built-In UML Renderers

Built-in renderers are the lowest-risk path. They should export as deterministic primitives:

- `uml.actor`: line/circle glyph plus label.
- `uml.use_case`: ellipse plus label.
- `uml.class`: class box with compartments.
- `uml.component`: component box with component tabs.
- `uml.activity`: rounded action/activity node.
- `uml.sequence_participant`: participant header and lifeline when sequence views are supported.

If a renderer lacks enough semantic detail for a compartment or label, it should export an empty compartment or omit the unsupported detail consistently.

### Image-Backed Elements

Image-backed renderers are allowed only for package-local assets declared in the render profile.

Status behavior:

- `available`: render and export the asset, subject to path, hash, kind, and sanitization checks.
- `referenced`: show/export a placeholder because the asset is planned but not bundled.
- `missing`: show/export a placeholder and emit a validation/export warning.
- `blocked`: show/export a placeholder and emit a blocking or high-severity warning, depending on the export policy.

Hit boxes, connector ports, labels, selection outlines, and trace overlays use the diagram/view metadata, not the visible image bounds guessed from pixels.

### SVG Assets

SVG assets are image assets, not executable UI.

Before embedding or converting an SVG asset, the exporter must reject or strip:

- script elements
- event-handler attributes
- `foreignObject`
- external image/font/style references
- remote URLs
- active animation that changes semantic layout

If sanitization cannot be proven, the asset is treated as `blocked`.

### Custom HTML

`html.custom` is a separate trust boundary from image assets. It is allowed in the schema so the model can describe future renderer intent, but early exporters must treat it as unsupported unless a project policy explicitly enables a trusted renderer.

Default behavior:

- Browser preview: disabled or diagnostic placeholder.
- Tauri preview: disabled or diagnostic placeholder.
- SVG export: placeholder with renderer ID and rule ID.
- PDF export: placeholder through the SVG/PDF path.

Future support should require sandboxing, explicit project policy, deterministic sizing, and a safe conversion path to SVG or raster output.

## Export Scene

All export targets should eventually share a small intermediate export scene:

- diagram ID and title
- profile ID and resolved rule IDs
- nodes with model IDs, bounds, renderer target, label settings, ports, and asset references
- connectors with relationship IDs, endpoints, route hints, labels, and style
- warnings and blocked-renderer diagnostics

The export scene is not canonical project state. It is a deterministic product of canonical model files, view metadata, render profile, and export options.

## Validation And Warnings

Export validation should report:

- missing assets
- blocked or unsupported renderers
- invalid package-relative paths
- asset hash mismatches for `available` assets
- unresolved rule assets
- connector endpoints outside expected ports
- labels that cannot fit within requested export constraints

Warnings may allow preview export. Blocking findings should prevent release-quality export unless the user explicitly chooses a diagnostic export mode.

## Decisions

- SVG export should prefer vector primitives for built-in renderers.
- PDF export should be downstream of the same sanitized export scene rather than a separate renderer.
- Missing or blocked assets must be visible in exported output.
- `html.custom` remains schema-described but exporter-disabled until a stronger trust model exists.
- Renderer customization remains view/profile metadata; semantic model files do not depend on export appearance.
