#![cfg(feature = "wasm")]

use cranberry_waffles::{backend, lowering, parser};

fn execute(source: &[u8]) -> i64 {
    let parsed = parser::parse(source);
    let function = lowering::lower(&parsed).unwrap();
    cranelift_codegen::verify_function(
        &function,
        &cranelift_codegen::settings::Flags::new(cranelift_codegen::settings::builder()),
    )
    .unwrap();
    let wasm = backend::emit_wasm(&function, "main").unwrap();
    wasmparser::Validator::new().validate_all(&wasm).unwrap();
    let engine = wasmi::Engine::default();
    let module = wasmi::Module::new(&engine, &wasm).unwrap();
    let mut store = wasmi::Store::new(&engine, ());
    let linker = wasmi::Linker::<()>::new(&engine);
    let instance = linker.instantiate_and_start(&mut store, &module).unwrap();
    instance
        .get_typed_func::<(), i64>(&store, "main")
        .unwrap()
        .call(&mut store, ())
        .unwrap()
}

#[test]
fn ruby_program_returns_42_through_wasm() {
    assert_eq!(execute(include_bytes!("../fixtures/return_42.rb")), 42);
}

#[test]
fn lowers_prism_integer_values_including_radix_and_boundaries() {
    for (source, expected) in [
        ("0x2a", 42),
        ("4_2", 42),
        ("-42", -42),
        ("9223372036854775807", i64::MAX),
        ("-9223372036854775808", i64::MIN),
    ] {
        assert_eq!(execute(source.as_bytes()), expected, "{source}");
    }
}

#[test]
fn rejects_unsupported_programs_without_discarding_statements() {
    for source in [
        "",
        "puts 42",
        "42; 43",
        "return 42",
        "42 + 1",
        "9223372036854775808",
        "18446744073709551616",
        "def",
    ] {
        assert!(
            lowering::lower(&parser::parse(source.as_bytes())).is_err(),
            "{source}"
        );
    }
}

#[test]
#[ignore = "requires Ruby on PATH"]
fn result_matches_cruby() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/fixtures/return_42.rb");
    let output = std::process::Command::new("ruby")
        .args(["-e", "puts eval(File.binread(ARGV.fetch(0)))", "--", path])
        .output()
        .unwrap();
    assert!(output.status.success());
    let reference: i64 = std::str::from_utf8(&output.stdout)
        .unwrap()
        .trim()
        .parse()
        .unwrap();
    assert_eq!(
        execute(include_bytes!("../fixtures/return_42.rb")),
        reference
    );
}
