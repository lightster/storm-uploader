# Storm Uploader

A macOS menubar app that automatically uploads Heroes of the Storm `.StormReplay` files.

Built with [Tauri](https://v2.tauri.app/), [SvelteKit](https://svelte.dev/), and Rust.

## Downloads

- [macOS (Apple Silicon)](https://github.com/lightster/storm-uploader/releases/latest/download/Storm-Uploader_darwin-aarch64.dmg)
- [macOS (Intel)](https://github.com/lightster/storm-uploader/releases/latest/download/Storm-Uploader_darwin-x64.dmg)
- [Windows (installer)](https://github.com/lightster/storm-uploader/releases/latest/download/Storm-Uploader_windows-x64-setup.exe)

## Development

### Prerequisites

- [Node.js](https://nodejs.org/)
- [Rust](https://www.rust-lang.org/tools/install)
- Tauri CLI: `cargo install tauri-cli --version "^2"`

### Setup

```bash
npm install
npm run tauri dev
```

### Build

```bash
npm run tauri build
```

## License

MIT
