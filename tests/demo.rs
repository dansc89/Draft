use std::process::Command;
#[test]
fn demo_runs_headless_and_refuses_existing_directory() {
    let dir = std::env::temp_dir().join(format!("draft-demo-test-{}", std::process::id()));
    assert!(!dir.exists(), "test directory must be new");
    let run = || {
        Command::new(env!("CARGO_BIN_EXE_draft"))
            .env_remove("DISPLAY")
            .env_remove("WAYLAND_DISPLAY")
            .args(["--demo", dir.to_str().unwrap()])
            .output()
            .unwrap()
    };
    let output = run();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let text = std::fs::read_to_string(dir.join("rectangle.dxf")).unwrap();
    let lines = draft::persistence::decode_dxf(&text).unwrap();
    assert_eq!(lines.len(), 4);
    assert_eq!(lines[0].b.x, 12 * 12 * 64);
    assert_eq!(lines[1].b.y, 8 * 12 * 64);
    let pdf = std::fs::read(dir.join("rectangle.pdf")).unwrap();
    assert_eq!(pdf, draft::pdf::encode_pdf(&lines).unwrap());
    assert!(!run().status.success());
    assert_eq!(pdf, std::fs::read(dir.join("rectangle.pdf")).unwrap());
    std::fs::remove_dir_all(dir).unwrap();
}
