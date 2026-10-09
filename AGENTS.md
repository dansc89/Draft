# Draft development conventions

- Rust; early-CAD scope, imperial precision, full-size geometry, explicit print scale.
- Strict vertical behavioral TDD; isolate geometry and interchange from the UI.
- Preserve user drawings; reject unsupported entities rather than silently dropping them.
- No installation or automatic launch on the user desktop. GUI tests use an owned isolated display.
- No edits to Glyph or OS/Omarchy configuration from this project.
- Generated fixtures only in public CI. No production-readiness claims before independent CAD/PDF, persistence and native Wayland acceptance.
