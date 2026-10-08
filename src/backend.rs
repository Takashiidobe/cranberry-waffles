use anyhow::Result;
use cranelift_codegen::ir::Function;

pub fn emit_wasm(function: &Function, export: &str) -> Result<Vec<u8>> {
    clif2wasm::translate_module(&[clif2wasm::NamedClifFunc::new(export, function)], export)
}
