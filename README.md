# Draft

Accurate 2D drafting in feet and inches, with editable DXF drawings and scaled vector PDF output. Written in Rust for a native Linux desktop, with Omarchy as the primary interaction target.

**Early development, not production-ready.** Draft takes inspiration from the small, direct scope of early CAD tools; it is not an Autodesk product or a claim of AutoCAD compatibility.

## First increment

The initial build targets line drawing, architectural coordinate entry, orthogonal/grid interaction, undo/redo, supported LINE-only DXF save/reopen, and vector PDF export. See [the product contract](docs/product-contract.md) for the acceptance criteria and explicit limits. Features must be demonstrated by tests and independent file readers, not inferred from this roadmap.

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
