# Draft

Accurate 2D drafting in feet and inches, with editable DXF drawings and scaled vector PDF output. Written in Rust for a native Linux desktop, with Omarchy as the primary interaction target.

**Early development, not production-ready.** Draft takes inspiration from the small, direct scope of early CAD tools; it is not an Autodesk product or a claim of AutoCAD compatibility.

## Working first increment

- Native Rust desktop canvas: two-click lines, selectable grid snapping, orthogonal cursor mode, zoom/pan and undo/redo.
- Exact start/end coordinates in feet, inches and architectural fractions, stored as integer **1/64-inch ticks**. Unsupported precision is rejected, never silently rounded. Coordinate bounds are ±100,000 feet.
- Save and reopen Draft's own LINE-only, inch-unit DXF subset, up to **8 MiB**. Oversized saves fail before writing and leave the drawing unsaved. Independent `ezdxf` auditing verifies the emitted geometry; externally edited/general DXF imports are **not** supported yet.
- Vector PDF export: **landscape Letter, 1:48 (`1/4" = 1'-0"`)**, half-inch margins. Oversize drawings are rejected rather than silently scaled.
- New-file-only saving/export, an unsaved Open guard and explicit unsaved-close confirmation. Choose a new filename for each save; overwrite, autosave and crash recovery remain future work.

The headless demo produces a full-size **12-foot by 8-foot rectangle**. Independent readers verify its DXF endpoints and a **3-inch by 2-inch** vector rectangle in the PDF. See [the product contract](docs/product-contract.md) for limits. Native Wayland usability/performance and broader CAD interoperability remain unverified; this is not a production release.

## Development

Install stable Rust and the Linux development dependencies used in `.github/workflows/ci.yml`.

```sh
cargo test --locked
cargo fmt --all -- --check
cargo clippy --locked --all-targets -- -D warnings
cargo run --locked
```

For a headless generated drawing and export proof (choose a new output directory):

```sh
cargo run --locked -- --demo /tmp/draft-demo
```

Run `uv run --with ezdxf --with pypdf python scripts/verify-demo.py /tmp/draft-demo` to independently check DXF geometry and physical PDF scale. These tools validate generated fixtures only; do not use private customer drawings in CI.

## Direction

Draw at full size. Zoom never changes geometry. Print scale is explicit and must not silently become “fit to page.” Numeric architectural input must either be represented accurately or rejected with a clear explanation.

Future work includes endpoint/intersection snapping, dimension entities, selection/editing, layers, general DXF interoperability, configurable sheets/scales, atomic overwrite/recovery, and measured native Wayland acceptance. A save/export button alone is not a production persistence guarantee.

Development is separate from [Glyph](https://github.com/dansc89/Glyph). No Glyph files or desktop installation are changed by this repository.

## License

MIT. DXF is Autodesk's documented interchange format, not a software license; any libraries added to Draft need their own license review.
