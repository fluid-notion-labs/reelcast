# Reelcast - Rust Media Server

A self-hosted media server written in Rust for streaming video to VLC and other players.

## Status

**Research Phase** - Evaluating protocols and VLC compatibility.

## VLC Supported Formats

### Container Formats

| Format | Extension | Notes |
|--------|----------|-------|
| MP4 | .mp4, .m4v | Most compatible for streaming |
| Matroska | .mkv, .mka | Wide codec support |
| AVI | .avi | Legacy, limited seeking |
| TS/MPEG-TS | .ts, .mts | Transport stream, good for streaming |
| MOV | .mov | QuickTime format |
| WebM | .webm | VP8/VP9/AV1 video, Vorbis/Opus audio |
| OGG | .ogv, .oga, .ogg | Theora/Vorbis |
| FLV | .flv | Flash video, legacy |

### Video Codecs

| Codec | Support | Notes |
|-------|---------|-------|
| H.264/AVC | Full | Best for streaming |
| H.265/HEVC | Full | Better compression, licensing concerns |
| VP8/VP9 | Full | royalty-free |
| AV1 | Full | Best compression, modern |
| MPEG-1/2 | Full | Universal support |
| MPEG-4 ASP (Xvid) | Full | Legacy |
| Theora | Full | royalty-free |
| VP3 (Theora predecessor) | Full | |
| WMV 1/2 | Full | Windows Media |
| VC-1 | Full | |
| DivX | Full | |
| MJPEG | Full | |

### Audio Codecs

| Codec | Support | Notes |
|-------|---------|-------|
| AAC | Full | Most common |
| MP3 | Full | Universal |
| Opus | Full | Low latency, royalty-free |
| Vorbis | Full | royalty-free |
| FLAC | Full | Lossless |
| ALAC | Full | Apple Lossless |
| AC3/E-AC3 | Full | Dolby |
| DTS | Full | |
| WMA | Full | Windows Media Audio |
| LPCM | Full | Raw PCM |

## Streaming Protocols for VLC

### HTTP (Recommended - Simplest)

Serve files directly over HTTP with range request support.

**Pros:**
- Simplest to implement
- Works through proxies/CDNs
- No special server software needed
- URL-based access

**Requirements:**
- Support HTTP Range requests for seeking
- `Accept-Ranges: bytes` header
- `Content-Length` header
- Proper `Content-Range` responses

**URL Format:**
```
http://server:8080/media/movie.mp4
```

**VLC Command:**
```
vlc http://server:8080/media/movie.mp4
```

### HLS (HTTP Live Streaming)

Segments video into .ts/.m3u8 files served over HTTP.

**Pros:**
- Adaptive bitrate streaming
- Native browser support
- Wide device compatibility
- Works through firewalls

**Cons:**
- Latency (6-30 seconds)
- Requires segmentation/transcoding
- More complex server implementation

**URL Format:**
```
http://server:8080/stream/movie.m3u8
```

**Components:**
- `.m3u8` playlist file
- `.ts` segment files

**Rust Crates:**
- `hlskit` - MP4 to HLS transcoding with ffmpeg
- Manual playlist/segment generation

### RTSP (Real-Time Streaming Protocol)

Pull-based streaming with RTP for data transport.

**Pros:**
- Low latency (1-5 seconds)
- Industry standard for IP cameras
- Good seeking support
- VLC native support

**Cons:**
- Requires dedicated ports
- NAT traversal can be tricky
- Not browser-friendly

**URL Format:**
```
rtsp://server:8554/stream
```

**Rust Crates:**
- `rtsp` - RTSP server library

### WebRTC

Peer-to-peer real-time communication.

**Pros:**
- Ultra-low latency (<0.5s)
- NAT traversal built-in
- Encrypted by default

**Cons:**
- Complex signaling required
- Not ideal for VOD
- Overkill for simple streaming

**URL Format:**
```
http://server:8080/webrtc
```

### DLNA/UPnP

Device discovery-based streaming.

**Pros:**
- Auto-discovery on network
- Smart TV compatible
- No URL needed

**Cons:**
- Complex protocol
- Limited control
- Device-dependent

**Rust Crates:**
- `vuio` - DLNA server in Rust

## Comparison Table

| Protocol | Latency | Ease | Seeking | Browser | VLC |
|----------|---------|------|---------|---------|-----|
| HTTP | Low | Easiest | Range req | Native | Yes |
| HLS | 6-30s | Medium | Yes | Native | Yes |
| RTSP | 1-5s | Medium | Yes | No | Yes |
| WebRTC | <0.5s | Hardest | Limited | Plugin | Yes |
| DLNA | Low | Hard | Yes | No | Limited |

## Recommendations

### For Simple File Streaming (Recommended Start)
- **Protocol:** HTTP with range requests
- **Container:** MP4 with moov atom at start (faststart)
- **Codec:** H.264 + AAC for maximum compatibility

### For Adaptive Bitrate
- **Protocol:** HLS
- **Segments:** 6-10 second chunks
- **Bitrates:** 1080p, 720p, 480p, 360p

### For Low Latency
- **Protocol:** RTSP
- **Container:** TS
- **Codec:** H.264

## Implementation Notes

### HTTP Range Requests (Required for Seeking)

VLC sends `Range: bytes=X-Y` headers for seeking. Server must respond with:
- `206 Partial Content`
- `Content-Range: bytes X-Y/TOTAL`
- `Content-Length: Y-X+1`

### MP4 Faststart

For web streaming, reorder moov atom to start of file:
```bash
ffmpeg -i input.mp4 -c copy -movflags +faststart output.mp4
```

### Key Rust Crates

| Crate | Purpose |
|-------|---------|
| `axum` | HTTP server |
| `tokio` | Async runtime |
| `hlskit` | HLS transcoding |
| `rtsp` | RTSP server |
| `gst-rtsp-server` | GStreamer RTSP |

## References

- [VLC Features](https://www.videolan.org/vlc/features.php)
- [VLC Supported Formats](https://wiki.videolan.org/Supported/)
- [HTTP Range Requests](https://developer.mozilla.org/en-US/docs/Web/HTTP/Range_requests)
