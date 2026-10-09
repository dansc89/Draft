# Contributing

Use small, tested changes. Follow behavioral RED → GREEN → REFACTOR. Keep geometry, document/history, interchange, and desktop interaction separable. Use generated drawings in tests; never commit private plans.

Before committing: `cargo fmt --all -- --check`, `cargo clippy --locked --all-targets -- -D warnings`, and `cargo test --locked`. Any change to DXF/PDF needs independent-reader evidence. Do not silently discard unsupported DXF content or adjust print scale to make drawings fit.
