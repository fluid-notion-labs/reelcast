//! Embedded UI assets.
//!
//! Vanilla (default): embedded at compile time from ui/vanilla/.
//! Svelte: build.rs runs `npm run build`, then ui/svelte/dist/ is embedded.
//!
//! Both served the same way — no runtime file I/O, self-contained binary.

use rust_embed::Embed;

#[cfg(not(feature = "svelte"))]
#[derive(Embed)]
#[folder = "ui/vanilla/"]
pub struct Assets;

#[cfg(feature = "svelte")]
#[derive(Embed)]
#[folder = "ui/svelte/dist/"]
pub struct Assets;

pub fn get(path: &str) -> Option<rust_embed::EmbeddedFile> {
    Assets::get(path)
}
