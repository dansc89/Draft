# Draft product contract

## Purpose
A small, accurate 2D drafting application: full-size lines entered in feet/inches, editable DXF output and properly scaled vector PDF printing. No modern AutoCAD feature-parity promise.

## First increment acceptance
- Architectural lengths and coordinates accept documented decimal/fractional syntax. Internal geometry has a documented precision and bounds; nonrepresentable values are explicitly rejected, not silently rounded.
- Zero-length and out-of-range lines do not mutate the document. Undo/redo reproduces identical geometry.
- GUI canvas dominates; keyboard/numeric entry provides precision independent of pixel placement. Orthogonal and grid interactions are explicit.
- Own supported LINE-only DXFs reopen without changed endpoints. Unknown drawing content is rejected rather than silently discarded. General-purpose DXF import is not yet promised.
- A 12-foot by 8-foot rectangle exports as a 3-inch by 2-inch vector rectangle at 1:48 on a landscape Letter sheet. PDF geometry is verified by an independent reader; dimensions are not inferred from screenshots.
- Oversize drawings fail explicitly. No automatic fit-to-page. Printing requires Actual size / 100%.
- Initial writes refuse to overwrite existing paths. Failed writes do not report success. Unsaved drawings are not silently replaced on Open. Atomic overwrite, autosave and crash recovery require later work.
- Rust tests, formatting, strict Clippy, headless exports and independent DXF/PDF checks pass. Native Wayland GUI and a broader CAD interoperability corpus are separate future gates.

## Later milestones
1. Endpoint/intersection snaps; select/move/copy/trim; dimensions and layers.
2. Configurable paper, print scale, line weights and printable bounds preview.
3. Transactional saving/recovery and versioned drawing migration.
4. Independent CAD round trips and production corpus, measured latency and native Omarchy acceptance.

Never advertise these later milestones as implemented until exercised.
