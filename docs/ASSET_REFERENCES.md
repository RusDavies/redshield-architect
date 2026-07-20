# Asset References

Render profiles may refer to user-provided images for custom element renderers. Asset references are view/theme metadata: they control appearance, but they do not change the semantic model element.

## Storage

Custom render assets live under the model package at `assets/render/`. Render profile asset URIs are package-relative paths under that directory, for example `assets/render/class-duck.png`.

The first supported asset kinds are:

- `image/png`
- `image/jpeg`
- `image/svg+xml`

Remote URLs are not loaded directly by render profiles. Importers may fetch or copy a remote asset, but the render profile should reference the resulting package asset path and record the original source in provenance.

## Status

Each asset has an explicit status:

- `referenced`: the profile records a planned or known asset path, but the binary is not bundled yet
- `available`: the asset is present in the package and must include `contentSha256`
- `missing`: the asset was expected but cannot be resolved
- `blocked`: policy, licensing, validation, or security checks prevent use

Renderers should use the rule fallback when an image asset is `referenced`, `missing`, or `blocked`, unless the UI is deliberately showing an edit/diagnostic state.

## Integrity

Available assets record `contentSha256` as `sha256:<hex>`. The hash is over the exact stored asset bytes. Importers should also record `byteSize` and pixel `dimensions` when known.

The workbench should warn before replacing an existing asset ID with different bytes. A deliberate replacement should update the hash, dimensions, byte size, and provenance notes.

## Provenance

Every asset records:

- `sourceType`: `user_provided`, `generated`, `library`, `imported`, or `unknown`
- `source`: a human-readable origin such as an upload filename, generator prompt reference, library package, or import source
- `license`: the declared license or usage basis

`createdBy`, `createdAt`, and `notes` are optional provenance fields. They should be used when the source is generated, imported from another tool, or subject to review.

## Trust Boundary

SVG assets are treated as images, not executable UI. A renderer should sanitize or isolate SVG content before display/export and must not execute scripts, foreign objects, event handlers, or remote references from user assets.

Render profiles can reference `html.custom` renderers, but user-authored HTML is a separate trust boundary from image assets and should require a stricter policy before being enabled.
