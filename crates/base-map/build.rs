// build.rs
use std::{
    env,
    fs,
    path::{Path, PathBuf},
    process::Command,
};

fn main() {
    // ---------- 0. Locate the front‑end ----------
    let fe = Path::new("frontend");
    assert!(
        fe.join("package.json").exists(),
        "Expected `frontend/package.json` – is your front‑end in `frontend/`?"
    );

    // ---------- 1. Tell Cargo when to rerun ----------
    println!("cargo:rerun-if-changed={}", fe.join("package.json").display());
    println!("cargo:rerun-if-changed={}", fe.join("vite.config.ts").display());
    println!("cargo:rerun-if-changed={}", fe.join("vite.config.js").display());
    println!("cargo:rerun-if-changed={}", fe.join("src").display());

    // ---------- 2. Install & build ----------
    run("npm", &["ci"], fe);          // or s/ci/install/ if you prefer
    run("npm", &["run", "build"], fe); // assumes "build": "vite build" in package.json

    // ---------- 3. Copy artefacts ----------
    let dist = fe.join("dist");
    let dest = out_static_dir();
    if dest.exists() {
        fs::remove_dir_all(&dest).expect("clearing old static output failed");
    }
    fs::create_dir_all(&dest).unwrap();
    copy_dir(&dist, &dest).expect("copying Vite dist/ failed");
}

// -----------------------------------------------------------------------------
// Helpers
// -----------------------------------------------------------------------------
fn run(cmd: &str, args: &[&str], cwd: &Path) {
    let ok = Command::new(cmd)
        .args(args)
        .current_dir(cwd)
        .status()
        .unwrap_or_else(|_| panic!("failed to spawn `{cmd}`; is it on PATH?"))
        .success();
    if !ok {
        panic!("command `{cmd} {}` failed", args.join(" "));
    }
}

/// `$OUT_DIR/../../../static`  → a sibling of your crate root inside `target/*/static`.
fn out_static_dir() -> PathBuf {
    let out_dir = env::var("OUT_DIR").expect("OUT_DIR env var is set by Cargo");
    PathBuf::from(out_dir)
        .parent()            // build/
        .and_then(|p| p.parent()) // <hash>/
        .and_then(|p| p.parent()) // release/
        .map(|p| p.join("static"))
        .expect("could not compute static dir")
}

/// Recursively copy a directory.
fn copy_dir(src: &Path, dst: &Path) -> std::io::Result<()> {
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let dst_path = dst.join(entry.file_name());
        if ty.is_dir() {
            fs::create_dir_all(&dst_path)?;
            copy_dir(&entry.path(), &dst_path)?;
        } else {
            fs::copy(entry.path(), dst_path)?;
        }
    }
    Ok(())
}
