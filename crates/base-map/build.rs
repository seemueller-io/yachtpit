// build.rs
use std::{
    env,
    fs,
    path::{Path, PathBuf},
    process::Command,
};

///
///    {
///      "name": "yachtpit",
///      "private": true,
///      "workspaces": ["packages/*"],
///      "scripts": {
///        "build-and-deploy-map": "cd crates/base-map && npm run build && cd ../.. && mkdir -p crates/yachtpit/assets/ui/packages/base-map/dist && cp -r packages/base-map/dist/* crates/yachtpit/assets/ui/packages/base-map/dist/ && cp -r packages/base-map/dist/assets crates/yachtpit/assets/ui/",
///        "postinstall": "npm run build-and-deploy-map"
///      },
///      "devDependencies": {
///        "@types/bun": "latest"
///      },
///      "peerDependencies": {
///        "typescript": "^5"
///      }
///    }
///


fn main() {
    // ---------- 0. Locate the map front‑end ----------
    let map_dir = Path::new("map");
    assert!(
        map_dir.join("package.json").exists(),
        "Expected `map/package.json` – is your map front‑end in `map/`?"
    );

    // ---------- 1. Tell Cargo when to rerun ----------
    println!("cargo:rerun-if-changed={}", map_dir.join("package.json").display());
    println!("cargo:rerun-if-changed={}", map_dir.join("vite.config.ts").display());
    println!("cargo:rerun-if-changed={}", map_dir.join("vite.config.js").display());
    println!("cargo:rerun-if-changed={}", map_dir.join("src").display());

    // ---------- 2. Install & build ----------
    run("npm", &["install"], map_dir);
    run("npm", &["run", "build"], map_dir); // assumes "build": "tsc -b && vite build" in package.json

    // ---------- 3. Copy artefacts following build-and-deploy-map script ----------
    let dist = map_dir.join("dist");

    let crate_asset_output = out_static_dir();

    // Create target directory: crates/yachtpit/assets/ui/packages/base-map/dist
    let base_map_dest = Path::new("../yachtpit/assets/ui/packages/base-map/dist");
    if base_map_dest.exists() {
        fs::remove_dir_all(&base_map_dest).expect("clearing old base-map dist failed");
    }

    if crate_asset_output.exists() {
        fs::remove_dir_all(&crate_asset_output).expect("clearing old base-map dist failed");
    }

    fs::create_dir_all(&base_map_dest).unwrap();
    fs::create_dir_all(&crate_asset_output).unwrap();

    // Copy dist/* to crates/yachtpit/assets/ui/packages/base-map/dist/
    copy_dir(&dist, &base_map_dest).expect("copying base-map dist/ failed");
    copy_dir(&dist, &crate_asset_output).expect("crate asset output failed");

    // Copy dist/assets to crates/yachtpit/assets/ui/assets (if assets exist)
    let dist_assets = dist.join("assets");
    if dist_assets.exists() {
        let ui_assets_dest = Path::new("../yachtpit/assets/ui/assets");
        if ui_assets_dest.exists() {
            fs::remove_dir_all(&ui_assets_dest).expect("clearing old ui assets failed");
        }
        fs::create_dir_all(&ui_assets_dest).unwrap();
        copy_dir(&dist_assets, &ui_assets_dest).expect("copying assets to ui/ failed");
    }
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
