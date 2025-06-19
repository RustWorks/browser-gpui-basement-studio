# Browser App

## Prerequisites

- Rust (latest stable)
- Cargo WEF CLI tool

## Setup

1. **Install Cargo WEF:**

   ```bash
   cargo install cargo-wef
   ```

2. **Initialize WEF (one-time setup):**
   ```bash
   cargo wef init
   ```

## Running the App

```bash
cargo wef run
```

## Project Structure

```
browser-app/
├── Cargo.toml          # Dependencies and project config
├── src/
│   └── main.rs         # Main application code
├── assets/             # SVG icons and UI assets
│   ├── back.svg
│   ├── forward.svg
│   ├── refresh.svg
│   ├── plus.svg
│   └── ...
└── README.md           # This file
```

## Dependencies

- `gpui` - Native UI framework
- `gpui-component` - UI components (with webview feature)
- `gpui-webview` - WebView integration
- `wef` - CEF-based web rendering
- `serde` - Serialization
- `futures-util` - Async utilities
- `flume` - Channel communication

## Troubleshooting

- If the app closes immediately, make sure you've run `cargo wef init`
- Ensure the `webview` feature is enabled for `gpui-component`
- Check that all SVG assets are present in the `assets/` directory

## Development

This is a standalone project (not part of a workspace) to avoid dependency resolution issues that can occur with workspace setups.

###

Command-option-escape to force quit
