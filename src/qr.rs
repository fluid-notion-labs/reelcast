//! QR code generation — outputs an SVG string for a given URL.
//! Uses the `qrcode` crate, no JS or external deps needed at runtime.

use qrcode::{QrCode, EcLevel};
use qrcode::render::svg;

pub fn svg_for_url(url: &str) -> String {
    let code = QrCode::with_error_correction_level(url, EcLevel::M)
        .unwrap_or_else(|_| QrCode::new(url).unwrap());
    code.render::<svg::Color>()
        .min_dimensions(200, 200)
        .quiet_zone(true)
        .build()
}
