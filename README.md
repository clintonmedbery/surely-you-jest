# Surely You Jest

[![Release](https://img.shields.io/github/v/release/clintonmedbery/surely-you-jest?include_prereleases)](https://github.com/clintonmedbery/surely-you-jest/releases)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A terminal-based UI tool to help you navigate, run, and debug Jest tests efficiently.

![Screenshot of Surely You Jest](https://via.placeholder.com/800x500.png?text=Surely+You+Jest+Screenshot)

## Features

- Browse and navigate all Jest tests in a project
- View individual tests within a file before running them
- Run specific tests instead of entire test files
- Interactive terminal with ANSI color support
- Intuitive keyboard navigation
- Loading indicators for test execution
- Scroll through test output with ease
- Copy test commands to clipboard for external debugging

## Installation

### Download Binary (Recommended)

Download the appropriate binary for your system from the [GitHub Releases page](https://github.com/clintonmedbery/surely-you-jest/releases).

1. Download the binary for your platform
2. Make it executable (Linux/macOS): `chmod +x syj-*`
3. Move it to a directory in your PATH (e.g., `/usr/local/bin` on macOS/Linux or create a directory and add to PATH on Windows)

### From Source

```bash
# Clone the repository
git clone https://github.com/clintonmedbery/surely-you-jest.git
cd surely-you-jest

# Build the binary
cargo build --release

# The binary will be in target/release/syj
```

## Usage

Navigate to your JavaScript/TypeScript project directory and run:

```bash
syj <path-to-tests-directory>
```

For example:

```bash
# Run from the project root to analyze all tests
syj .

# Run for a specific directory of tests
syj src/components/__tests__
```

### Key Bindings

#### Main Test List
- **↑/↓**: Navigate between test files
- **→**: View tests within the selected file
- **Ctrl+→**: View the raw file contents
- **Enter**: Run all tests in the file
- **PgUp/PgDn**: Page up/down through the list
- **q**: Quit

#### Test Results View
- **↑/↓**: Navigate between individual tests
- **→/Enter**: Run the selected test
- **←**: Go back to previous view
- **q**: Quit

#### Test Running View
- **↑/↓**: Scroll through test output
- **PgUp/PgDn**: Scroll faster
- **Home/End**: Jump to top/bottom of output
- **→**: View individual test results (when available)
- **Enter**: Copy command to clipboard / View test results
- **←**: Go back to previous view
- **q**: Quit

## Development

This project includes a live-reload script that will automatically rebuild and restart the application when you make changes to the source code.

### Prerequisites

Install cargo-watch:
```bash
cargo install cargo-watch
```

### Live Reloading

```bash
# Start the development server with live reloading
./dev.sh /path/to/your/project
```

This will watch for changes in your source files and automatically rebuild and restart the application.

## Releases

Binary releases are available on the [GitHub Releases page](https://github.com/clintonmedbery/surely-you-jest/releases). You can download the appropriate version for your operating system and architecture.

For information on the release process, see [RELEASING.md](./RELEASING.md).

## Requirements

- Node.js and Jest must be installed in your project
- Requires a terminal with color support

## License

Copyright (c) Clinton Medbery <clintonmedbery@users.noreply.github.com>

This project is licensed under the MIT license ([LICENSE](./LICENSE) or <http://opensource.org/licenses/MIT>)