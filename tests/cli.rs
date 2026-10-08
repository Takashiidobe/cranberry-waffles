use std::process::Command;

#[test]
#[cfg(feature = "wasm")]
fn compile_command_lowers_first_program() {
    let output = Command::new(env!("CARGO_BIN_EXE_cranberry-waffles"))
        .args(["compile", "fixtures/return_42.rb"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .unwrap();
    assert!(output.status.success(), "{:?}", output);
    let clif = String::from_utf8(output.stdout).unwrap();
    assert!(clif.contains("iconst.i64 42"));
    assert!(clif.contains("return v0"));
}

#[test]
fn parse_command_reports_syntax_errors() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_cranberry-waffles"))
        .args(["parse", "-"])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    std::io::Write::write_all(&mut child.stdin.take().unwrap(), b"def\n").unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("error:"));
    assert!(output.stdout.is_empty());
}

#[test]
fn parse_command_prints_ast() {
    let output = Command::new(env!("CARGO_BIN_EXE_cranberry-waffles"))
        .args(["parse", "fixtures/semantics.rb", "--ast"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("ProgramNode"));
}

#[test]
#[ignore = "requires Ruby on PATH; run with cargo test -- --include-ignored"]
fn reference_runs_corner_cases() {
    let output = Command::new(env!("CARGO_BIN_EXE_cranberry-waffles"))
        .args(["reference", "fixtures/semantics.rb"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .unwrap();
    assert!(output.status.success(), "{:?}", output);
    assert_eq!(output.stdout, b"semantics OK\n");
}
