use std::ffi::OsString;
use std::fs;
use std::io::{self, Read, Write};
use std::path::PathBuf;
use std::process::ExitCode;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use cranberry_waffles::{parser, reference};

#[derive(Parser)]
#[command(version, about)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[cfg(feature = "wasm")]
    #[command(about = "Lower a Ruby integer expression to CLIF or WebAssembly")]
    Compile {
        file: PathBuf,
        #[arg(
            short,
            long,
            help = "Write a Wasm module exporting main; otherwise print CLIF"
        )]
        output: Option<PathBuf>,
    },
    #[command(about = "Parse a Ruby file with Prism; use - to read stdin")]
    Parse {
        file: PathBuf,
        #[arg(long, help = "Print Prism's AST")]
        ast: bool,
    },
    #[command(about = "Run a Ruby file with the reference implementation")]
    Reference {
        file: PathBuf,
        #[arg(long, default_value = "ruby")]
        ruby: OsString,
        #[arg(last = true)]
        args: Vec<OsString>,
    },
}

fn run(cli: Cli) -> Result<ExitCode> {
    match cli.command {
        #[cfg(feature = "wasm")]
        Commands::Compile { file, output } => {
            let source =
                fs::read(&file).with_context(|| format!("failed to read {}", file.display()))?;
            let parsed = parser::parse(&source);
            let function = cranberry_waffles::lowering::lower(&parsed)
                .with_context(|| format!("failed to lower {}", file.display()))?;
            if let Some(output) = output {
                let wasm = cranberry_waffles::backend::emit_wasm(&function, "main")?;
                fs::write(&output, wasm)
                    .with_context(|| format!("failed to write {}", output.display()))?;
            } else {
                println!("{}", function.display());
            }
            Ok(ExitCode::SUCCESS)
        }
        Commands::Parse { file, ast } => {
            let source = if file.as_os_str() == "-" {
                let mut source = Vec::new();
                io::stdin().read_to_end(&mut source)?;
                source
            } else {
                fs::read(&file).with_context(|| format!("failed to read {}", file.display()))?
            };
            let parsed = parser::parse(&source);
            for diagnostic in parsed.diagnostics() {
                let severity = match diagnostic.severity {
                    parser::Severity::Error => "error",
                    parser::Severity::Warning => "warning",
                };
                eprintln!(
                    "{}:{}:{}: {severity}: {}",
                    file.display(),
                    diagnostic.line,
                    diagnostic.byte_column,
                    diagnostic.message
                );
            }
            if !parsed.is_valid() {
                return Ok(ExitCode::FAILURE);
            }
            if ast {
                println!("{:#?}", parsed.node());
            } else {
                println!("{}: syntax OK", file.display());
            }
            Ok(ExitCode::SUCCESS)
        }
        Commands::Reference { file, ruby, args } => {
            let output = reference::run(&ruby, &file, &args)?;
            io::stdout().write_all(&output.stdout)?;
            io::stderr().write_all(&output.stderr)?;
            Ok(ExitCode::from(
                output
                    .status
                    .code()
                    .and_then(|code| u8::try_from(code).ok())
                    .unwrap_or(1),
            ))
        }
    }
}

fn main() -> ExitCode {
    match run(Cli::parse()) {
        Ok(code) => code,
        Err(error) => {
            eprintln!("error: {error:#}");
            ExitCode::FAILURE
        }
    }
}
