# Architecture and verification

## Boundaries

- `src/lib.rs`: imperial parser, bounded integer tick coordinates, validated lines, document geometry and reversible line history. No screen coordinates or PDF rendering in this model.
- `src/editor.rs`: explicit cursor/grid snapping, orthogonal constraints and guarded document replacement.
- `src/persistence.rs`: exact decimal encoding of ticks into inch-unit LINE DXF and strict decoding of Draft's emitted subset; new-file-only writes.
- `src/pdf.rs`: vector PDF generation with explicit 1:48 physical scale, landscape Letter page and printable bounds rejection.
- `src/ui.rs`: native egui/eframe desktop controls and canvas. Rendering converts geometry into screen coordinates; mouse input snaps explicitly, while numeric coordinates bypass pixel conversion.
- `src/main.rs`: native app startup or an explicit headless generated-drawing command.

## First-increment evidence

Local Rust tests exercise architectural parsing/rejection, geometry invariants, undo/redo, strict DXF round trips, tick resolution, printable bounds, symlink/no-clobber behavior, headless demo generation, unsaved Open/save failures and virtual-egui exact/canvas line entry. Formatting and strict Clippy are additional gates, not replacements for behavioral tests.

The parent independently ran the actual compiled `draft --demo` command. `ezdxf` audited the output and checked four full-size 12-foot by 8-foot rectangle edges in inch units. `pypdf` parsed the actual PDF, checked four stroked vector edges and landscape Letter geometry, and measured 216 by 144 PDF points at 1:48. Poppler read and rendered the PDF; visual inspection found a closed rectangle with no clipping or corruption. A repeated demo invocation was rejected and existing output hashes stayed identical.

`scripts/verify-demo.py` is a verifier for this generated rectangle fixture, **not a general CAD/PDF preservation validator**. Its own regression test rejects unpainted paths. Broader checks for clipping, invisibility and arbitrary content are future work.

## Not accepted yet

No normal-desktop app was automatically launched or installed. Native Wayland/Omarchy usability, crash recovery, transactional overwriting, general DXF round trips, dimensions/layers, and matched interaction-performance measurements are separate future acceptance gates. Current GUI undo/redo routing during focused text editing and dirty-close viewport-event coverage also need refinement before production use.
