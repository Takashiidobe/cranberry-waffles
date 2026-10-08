use anyhow::{Context, Result, bail, ensure};
use cranelift_codegen::cursor::{Cursor, FuncCursor};
use cranelift_codegen::ir::{AbiParam, Function, InstBuilder, Signature, UserFuncName, types};
use cranelift_codegen::isa::CallConv;

use crate::parser::ParsedSource;

pub fn lower(parsed: &ParsedSource<'_>) -> Result<Function> {
    ensure!(parsed.is_valid(), "cannot lower Ruby with syntax errors");
    let program = parsed
        .node()
        .as_program_node()
        .context("expected a Ruby program")?;
    let body = program.statements().body();
    let mut statements = body.iter();
    let statement = statements
        .next()
        .context("empty Ruby programs are not supported yet")?;
    ensure!(
        statements.next().is_none(),
        "only a single integer expression is supported yet"
    );
    let integer = statement
        .as_integer_node()
        .context("only an integer literal is supported yet")?;
    let value = integer_value(integer.value())?;

    let mut signature = Signature::new(CallConv::SystemV);
    signature.returns.push(AbiParam::new(types::I64));
    let mut function = Function::with_name_signature(UserFuncName::user(0, 0), signature);
    let block = function.dfg.make_block();
    function.layout.append_block(block);
    let mut cursor = FuncCursor::new(&mut function);
    cursor.goto_bottom(block);
    let result = cursor.ins().iconst(types::I64, value);
    cursor.ins().return_(&[result]);
    Ok(function)
}

fn integer_value(integer: ruby_prism::Integer<'_>) -> Result<i64> {
    let (negative, digits) = integer.to_u32_digits();
    let mut magnitude = 0_u64;
    for &digit in digits.iter().rev() {
        magnitude = magnitude
            .checked_mul(1_u64 << 32)
            .and_then(|value| value.checked_add(u64::from(digit)))
            .context("integer literal is outside the supported i64 range")?;
    }
    if negative {
        if magnitude == 1_u64 << 63 {
            return Ok(i64::MIN);
        }
        Ok(-i64::try_from(magnitude)
            .context("integer literal is outside the supported i64 range")?)
    } else {
        match i64::try_from(magnitude) {
            Ok(value) => Ok(value),
            Err(_) => bail!("integer literal is outside the supported i64 range"),
        }
    }
}
