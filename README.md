# Cranberry waffles

A testing target to fuzz against ruby.

The initial scaffold parses Ruby with [ruby-prism](https://github.com/ruby/prism)
and runs files with CRuby as a reference. The first lowering path supports a
program containing one integer literal, returning its value through CLIF and
Wasm. A Ruby runtime, broader AST lowering, native execution, a differential
harness, and the browser UI are future work.

## Getting started

Install Rust, a C compiler, and libclang (Prism's vendored C parser uses bindgen).
Install Ruby to run reference cases; parsing does not require a Ruby installation.

```sh
cargo run -- parse fixtures/semantics.rb
cargo run -- parse fixtures/semantics.rb --ast
printf 'puts 1 + 2\n' | cargo run -- parse -
cargo run -- reference fixtures/semantics.rb
cargo run -- reference path/to/case.rb --ruby /path/to/ruby -- argument1 argument2
cargo test -- --include-ignored
cargo check --features wasm
cargo run --features wasm -- compile fixtures/return_42.rb
cargo run --features wasm -- compile fixtures/return_42.rb -o /tmp/return_42.wasm
cargo test --all-features -- --include-ignored
```

`parse` exits unsuccessfully on syntax errors and reports errors and warnings on
stderr with one-based line and byte-column locations. Source stays as bytes so
Prism can handle Ruby encoding comments. `--ast` prints Prism's debug AST for
valid input. The library exposes the borrowed AST while keeping its parse result
alive, plus owned diagnostics with byte spans.

`reference` executes a file with Ruby, forwards captured stdout and stderr, and
preserves its numeric exit code (signal termination becomes exit code 1). Script
arguments follow `--`. It inherits the environment and working directory and
closes stdin. This runner executes trusted local cases; it currently has no
timeout or isolation. It can run standalone regression cases; running ruby/spec
will also require its MSpec harness and supporting files.

The optional `wasm` feature enables Cranelift 0.136 and the pinned
[clif2wasm fork](https://github.com/Takashiidobe/clif2wasm).
`backend::emit_wasm` accepts an already-lowered CLIF function and returns a Wasm
module exporting it. `lowering::lower` accepts a parsed Ruby program with exactly
one integer literal in the signed 64-bit range. The `compile` command prints
CLIF by default; `-o` writes a Wasm module exporting `main: () -> i64`. This raw
integer ABI is a first experiment, and will need a Ruby value representation
when the runtime grows. Unsupported syntax and larger integers produce errors.
The default build keeps the parser and reference harness independent of backend
compilation.

`fixtures/return_42.rb` contains `42`: its final expression is the program's
result, rather than its process exit status. The lowering tests verify CLIF,
validate the Wasm module, and execute `main` with Wasmi. The Ruby-dependent test
compares that result with CRuby's `eval` of the same file.

`fixtures/semantics.rb` starts the case corpus with truthiness, signed division,
modulo, and arbitrary-precision integer checks. Ruby-dependent tests are ignored
by default; `--include-ignored` runs them when Ruby is installed.

## Intended pipeline

```
Prism -> Lower to Cranelift -> clif2wasm -> Browser execution
                    -> Run natively on rust
```

This is a research project to see where the ruby spec might be vague to
lower, given the same parser (prism), but a different backend
(cranelift). We should be able to test against different targets. The
Browser execution side is for sharing links (like compiler explorer does
for testing compilers).

Some research questions:

Q1: is `ruby/spec` concrete enough to distinguish plausibly correct but
incorrect implementations of ruby constructs?

Q2: Can metamorphic tests identify surprising behavior within CRuby that
runs counter to documented behavior?

Q3: Can testing these semantics through differential testing produce
clarifications to the written ruby spec as well as the automated test
suite (`ruby/spec`)?
