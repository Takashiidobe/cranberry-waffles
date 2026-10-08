use cranberry_waffles::parser::{Severity, parse};

#[test]
fn parses_semantic_corner_cases() {
    let parsed = parse(include_bytes!("../fixtures/semantics.rb"));
    assert!(parsed.is_valid(), "{:?}", parsed.diagnostics());
    assert!(parsed.node().as_program_node().is_some());
}

#[test]
fn rejects_invalid_syntax_with_source_locations() {
    let source = b"puts 1\n)\n";
    let parsed = parse(source);
    assert!(!parsed.is_valid());
    let error = parsed
        .diagnostics()
        .iter()
        .find(|d| d.severity == Severity::Error)
        .unwrap();
    assert_eq!(error.line, 2);
    assert_eq!(error.byte_column, 1);
    assert_eq!(&source[error.span.clone()], b")");
}

#[test]
fn honors_ruby_encoding_comments_without_utf8_conversion() {
    let parsed = parse(b"# encoding: ISO-8859-1\nname = \"\xe9\"\n");
    assert!(parsed.is_valid(), "{:?}", parsed.diagnostics());
}
