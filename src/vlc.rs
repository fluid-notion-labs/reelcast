use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};

use crate::db::MediaFile;

/// Generate an XSPF playlist that VLC understands.
/// `base_url` is e.g. "http://192.168.1.10:3000"
pub fn xspf(media: &[MediaFile], base_url: &str) -> String {
    let tracks: String = media
        .iter()
        .map(|m| track_xml(m, base_url))
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<playlist xmlns="http://xspf.org/ns/0/" version="1">
  <trackList>
{tracks}
  </trackList>
</playlist>"#
    )
}

fn track_xml(m: &MediaFile, base_url: &str) -> String {
    let url = format!("{}/file/{}", base_url, m.id);
    let title = escape_xml(&m.title);
    let duration_ms = m.duration_secs.map(|d| d * 1000).unwrap_or(0);

    format!(
        r#"    <track>
      <location>{url}</location>
      <title>{title}</title>
      <duration>{duration_ms}</duration>
    </track>"#
    )
}

/// Generate a simple M3U playlist (single item)
pub fn m3u_single(m: &MediaFile, base_url: &str) -> String {
    let url = format!("{}/file/{}", base_url, m.id);
    format!("#EXTM3U\n#EXTINF:{},{}\n{}\n",
        m.duration_secs.unwrap_or(-1),
        m.title,
        url,
    )
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

/// Encode a file path for use in a URL
#[allow(dead_code)]
pub fn encode_path(path: &str) -> String {
    utf8_percent_encode(path, NON_ALPHANUMERIC).to_string()
}
