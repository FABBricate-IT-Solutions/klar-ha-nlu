//! Writes `web/src/generated/contract.ts` from `klar_nlu::web_contract`.
//!
//! Run: `cargo run --bin export_web_contract`
//! CI: `scripts/check_generated.sh` fails when the committed file differs.

use klar_nlu::web_contract;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    if let Err(err) = run() {
        let _ = writeln!(io::stderr(), "export_web_contract: {err}");
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}

fn run() -> io::Result<()> {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let out = manifest.join("web/src/generated/contract.ts");
    if let Some(parent) = out.parent() {
        fs::create_dir_all(parent)?;
    }
    let body = web_contract::export_typescript();
    fs::write(&out, body)?;
    println!("wrote {}", out.display());
    Ok(())
}
