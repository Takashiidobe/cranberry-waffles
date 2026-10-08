use std::ops::Range;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub severity: Severity,
    pub message: String,
    pub span: Range<usize>,
    pub line: usize,
    pub byte_column: usize,
}

pub struct ParsedSource<'a> {
    result: ruby_prism::ParseResult<'a>,
    diagnostics: Vec<Diagnostic>,
}

impl ParsedSource<'_> {
    pub fn node(&self) -> ruby_prism::Node<'_> {
        self.result.node()
    }

    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    pub fn is_valid(&self) -> bool {
        !self
            .diagnostics
            .iter()
            .any(|d| d.severity == Severity::Error)
    }
}

pub fn parse(source: &[u8]) -> ParsedSource<'_> {
    let result = ruby_prism::parse(source);
    let mut diagnostics = Vec::new();
    for (severity, items) in [
        (Severity::Error, result.errors()),
        (Severity::Warning, result.warnings()),
    ] {
        for item in items {
            let location = item.location();
            let start = location.start_offset();
            let prefix = &source[..start];
            let line = prefix.iter().filter(|&&byte| byte == b'\n').count() + 1;
            let line_start = prefix
                .iter()
                .rposition(|&byte| byte == b'\n')
                .map_or(0, |i| i + 1);
            diagnostics.push(Diagnostic {
                severity,
                message: item.message().to_owned(),
                span: start..location.end_offset(),
                line,
                byte_column: start - line_start + 1,
            });
        }
    }
    ParsedSource {
        result,
        diagnostics,
    }
}
