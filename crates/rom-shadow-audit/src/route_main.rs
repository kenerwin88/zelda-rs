//! CPU-only ordered source/native join; no pricing feedback.
use rom_shadow_audit::{bridge::route,Result};
use std::path::Path;
fn main()->Result<()> {
    let a:Vec<_>=std::env::args().skip(1).collect();
    if a.len()!=5 && !(a.len()==6 && a[5]=="--prune-native-chunks"){return Err("rom-route-match ROM ORDERED_SOURCE NATIVE_SUMMARY PREDICTOR FRESH_OUT [--prune-native-chunks]".into());}
    route::run_chunks(Path::new(&a[0]),Path::new(&a[1]),Path::new(&a[2]),Path::new(&a[3]),Path::new(&a[4]),a.len()==6)
}
