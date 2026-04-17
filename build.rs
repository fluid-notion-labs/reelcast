//! Build script — runs UI build steps when needed.
//! Vanilla: noop (files are embedded via include_str! at compile time).
//! Svelte:  runs `npm run build` in ui/svelte/ to produce ui/svelte/dist/.

fn main() {
    #[cfg(feature = "svelte")]
    build_svelte();

    // Tell cargo to re-run this script if any UI source changes
    println!("cargo:rerun-if-changed=ui/vanilla/index.html");
    println!("cargo:rerun-if-changed=ui/vanilla/setup.html");
    println!("cargo:rerun-if-changed=ui/svelte/src");
    println!("cargo:rerun-if-changed=ui/svelte/package.json");
    println!("cargo:rerun-if-changed=ui/svelte/svelte.config.js");
}

#[cfg(feature = "svelte")]
fn build_svelte() {
    use std::process::Command;

    let ui_dir = std::path::Path::new("ui/svelte");

    // Install deps if node_modules missing
    if !ui_dir.join("node_modules").exists() {
        let status = Command::new("npm")
            .args(["install"])
            .current_dir(ui_dir)
            .status()
            .expect("failed to run npm install — is node/npm installed?");
        assert!(status.success(), "npm install failed");
    }

    let status = Command::new("npm")
        .args(["run", "build"])
        .current_dir(ui_dir)
        .status()
        .expect("failed to run npm run build");
    assert!(status.success(), "npm run build failed in ui/svelte/");
}
