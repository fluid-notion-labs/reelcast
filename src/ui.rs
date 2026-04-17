//! Embedded UI assets.
//!
//! Vanilla (default): files embedded at compile time from ui/vanilla/.
//! Svelte: dist/ built by build.rs then embedded from ui/svelte/dist/.
//!
//! Both are served the same way — no runtime file I/O.

#[cfg(not(feature = "svelte"))]
mod inner {
    use rust_embed::Embed;

    #[derive(Embed)]
    #[folder = "ui/vanilla/"]
    pub struct Assets;
}

#[cfg(feature = "svelte")]
mod inner {
    use rust_embed::Embed;

    #[derive(Embed)]
    #[folder = "ui/svelte/dist/"]
    pub struct Assets;
}

pub use inner::Assets;

/// Get a file from the embedded assets.
pub fn get(path: &str) -> Option<rust_embed::EmbeddedFile> {
    Assets::get(path)
}
