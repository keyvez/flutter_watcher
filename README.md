# Flutter Watcher

A Rust-based tool that watches your Flutter project for file changes and automatically triggers hot reload by sending the 'r' key to the Flutter process.

## Features

- 🔥 Automatic hot reload on Dart file changes
- 🚀 Spawns and manages Flutter run process
- 👀 Recursive file watching
- ⚡ Debounced reload to prevent excessive reloads
- 🎯 Configurable device targeting
- 🛡️ Graceful shutdown with Ctrl+C

## Prerequisites

- Rust (install from [rustup.rs](https://rustup.rs))
- Flutter SDK installed and in PATH
- A Flutter project to watch

## Installation

### From Source

```bash
git clone <repository-url>
cd flutter_watcher
cargo build --release
```

The binary will be available at `target/release/flutter_watcher`

### Install Globally

```bash
cargo install --path .
```

## Usage

### Basic Usage

Run in the current directory (must be a Flutter project):

```bash
flutter_watcher
```

### Specify Flutter Project Path

```bash
flutter_watcher --path /path/to/flutter/project
```

### Target Specific Device

```bash
flutter_watcher --device chrome
flutter_watcher -d "iPhone 14"
```

### Pass Additional Flutter Arguments

```bash
flutter_watcher -- --release
flutter_watcher -- --dart-define=API_URL=https://api.example.com
```

### Complete Example

```bash
flutter_watcher --path ./my_app --device chrome -- --dart-define=ENV=dev
```

## How It Works

1. **Spawns Flutter Process**: Starts `flutter run` with your specified arguments
2. **Watches for Changes**: Monitors all `.dart` files in the project directory
3. **Auto-Reload**: When a file changes, sends 'r' to the Flutter process stdin
4. **Debouncing**: Waits 500ms between reloads to avoid excessive triggering

## Command Line Options

```
Options:
  -p, --path <PATH>      Path to the Flutter project directory [default: .]
  -d, --device <DEVICE>  Device ID to run Flutter on (optional)
  -h, --help             Print help
  -V, --version          Print version

Additional Flutter arguments can be passed after '--'
```

## Examples

### Web Development

```bash
flutter_watcher -d chrome
```

### Mobile Development (iOS Simulator)

```bash
flutter_watcher -d "iPhone 14 Pro"
```

### Android Emulator

```bash
flutter_watcher -d emulator-5554
```

### Desktop Development

```bash
flutter_watcher -d macos
flutter_watcher -d windows
flutter_watcher -d linux
```

## Development

### Build

```bash
cargo build
```

### Run in Development

```bash
cargo run -- --path /path/to/flutter/project
```

### Run Tests

```bash
cargo test
```

## Troubleshooting

### "Flutter process has stopped"

The Flutter process may have crashed or exited. Check the Flutter output above for errors.

### "Failed to spawn flutter run process"

Ensure Flutter is installed and available in your PATH:

```bash
flutter --version
```

### No Hot Reload Happening

- Ensure you're editing `.dart` files
- Check that the Flutter process is still running
- Verify the file is within the watched directory

## License

MIT

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
