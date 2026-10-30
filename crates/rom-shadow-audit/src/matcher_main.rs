//! Offline CPU-only inventory reporting; never changes product accounting.
pub use rom_shadow_audit::{Identity,Result};
mod matcher;
use std::path::Path;
fn main()->Result<()> {
    let args:Vec<_>=std::env::args().skip(1).collect();
    if args.len()!=6 {return Err("rom-retirement-match SOURCE_INVENTORY NATIVE_SUMMARY PREDICTOR INVERSES CLAIM_CATALOG FRESH_OUT".into());}
    matcher::run(Path::new(&args[0]),Path::new(&args[1]),Path::new(&args[2]),
        Path::new(&args[3]),Path::new(&args[4]),Path::new(&args[5]))
}
