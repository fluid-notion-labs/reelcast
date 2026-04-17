//! Build script — runs UI build steps when needed.
//! Vanilla: noop (files are embedded via include_str! at compile time).
//! Svelte:  runs `npm run build` in ui/svelte/ to produce ui/svelte/dist/.

fn main() {
    println!("cargo:rerun-if-changed=ui/vanilla/index.html");
    println!("cargo:rerun-if-changed=ui/vanilla/setup.html");
    println!("cargo:rerun-if-changed=ui/svelte/src");
    println!("cargo:rerun-if-changed=ui/svelte/package.json");
    println!("cargo:rerun-if-changed=ui/svelte/svelte.config.js");

    #[cfg(feature = "svelte")]
    build_svelte();
}

#[cfg(feature = "svelte")]
fn build_svelte() {
    use std::process::{Command, Stdio};

    let ui_dir = std::path::Path::new("ui/svelte");

    println!("cargo:warning=🔨 Building Svelte UI in {}", ui_dir.display());

    // Install deps if node_modules missing
    if !ui_dir.join("node_modules").exists() {
        println!("cargo:warning=📦 node_modules not found — running npm install...");
        let output = Command::new("npm")
            .args(["install"])
            .current_dir(ui_dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .expect("failed to run npm install — is node/npm installed?");

        for line in String::from_utf8_lossy(&output.stdout).lines() {
            println!("cargo:warning=[npm install] {}", line);
        }
        for line in String::from_utf8_lossy(&output.stderr).lines() {
            println!("cargo:warning=[npm install] {}", line);
        }
        if !output.status.success() {
            panic!("npm install failed (exit {})", output.status);
        }
        println!("cargo:warning=✅ npm install done");
    }

    // Clear stale dist/ so rust-embed doesn't embed old files
    let dist = ui_dir.join("dist");
    if dist.exists() {
        println!("cargo:warning=🧹 Clearing stale dist/");
        std::fs::remove_dir_all(&dist).expect("failed to remove dist/");
    }

    println!("cargo:warning=⚙️  Running npm run build...");
    let output = Command::new("npm")
        .args(["run", "build"])
        .current_dir(ui_dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("failed to run npm run build");

    for line in String::from_utf8_lossy(&output.stdout).lines() {
        println!("cargo:warning=[npm build] {}", line);
    }
    for line in String::from_utf8_lossy(&output.stderr).lines() {
        println!("cargo:warning=[npm build] {}", line);
    }

    if !output.status.success() {
        panic!("npm run build failed (exit {})\nCheck the output above.", output.status);
    }

    // Verify dist/ was actually produced
    if !dist.exists() {
        panic!("npm run build succeeded but dist/ was not created — check svelte.config.js");
    }

    let file_count = std::fs::read_dir(&dist)
        .map(|d| d.count())
        .unwrap_or(0);
    println!("cargo:warning=✅ Svelte build complete — {} items in dist/", file_count);
}
