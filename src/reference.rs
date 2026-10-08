use std::ffi::{OsStr, OsString};
use std::path::Path;
use std::process::{Command, Output, Stdio};

use anyhow::{Context, Result};

pub fn run(ruby: &OsStr, path: &Path, args: &[OsString]) -> Result<Output> {
    Command::new(ruby)
        .arg("--")
        .arg(path)
        .args(args)
        .stdin(Stdio::null())
        .output()
        .with_context(|| format!("failed to run Ruby executable {}", ruby.to_string_lossy()))
}
