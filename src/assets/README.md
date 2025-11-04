# Assets Directory

This directory contains embedded assets for the chess engine.

## Fonts

### Roboto-Regular.ttf
- **Source**: Google Roboto Font Family
- **License**: Apache License 2.0 (see LICENSE file)
- **Purpose**: Embedded font for text rendering in both native and WASM builds
- **Size**: ~504 KB
- **URL**: https://github.com/google/roboto

The font is embedded at compile time using `include_bytes!()` to ensure text rendering works in WASM builds where system fonts are not available.

## Chess Piece Images

The PNG files contain chess piece graphics used for board rendering:
- Black pieces: `b_<piece>_png_128px.png`
- White pieces: `w_<piece>_png_128px.png`
- Legal move indicator: `circle.png`

All piece images are 128x128 pixels and embedded using the `assets` module.
