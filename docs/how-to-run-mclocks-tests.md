# How to Run Tests

This document explains how to run tests for the mclocks project and how to set up the development environment.

## Development Environment Setup

### Required Tools

The following tools are required:

1. **Node.js** (v18 or higher recommended)
   - Download from [Node.js official website](https://nodejs.org/)

2. **pnpm** (Package manager)
   ```bash
   npm install -g pnpm
   ```

3. **Rust** (Latest stable version)
   - Install from [Rust official website](https://www.rust-lang.org/)
   - It is recommended to use `rustup` for installation

4. **Cargo** (Rust package manager)
   - Automatically included when Rust is installed

### Installing Dependencies

Run the following commands in the project root directory:

```bash
# Install JavaScript dependencies
pnpm install

# Rust dependencies are automatically installed during build
# To install explicitly:
cd src-tauri
cargo build
cd ..
```

## JavaScript Tests

### Test Framework

This project uses **WebdriverIO** to run E2E tests.

### Prerequisites for Running Tests

**You need to start the application** before running JavaScript tests.

1. Start the application in a separate terminal:
   ```bash
   pnpm tauri dev
   ```

2. Verify that the application is running at `http://localhost:1420`.

### Running Tests

With the application running, execute the following command from another terminal:

```bash
# Run tests in normal mode
pnpm test

# Run tests in headless mode (browser not displayed)
pnpm test:headless
```

### Test File Locations

- Test files: `test/specs/mclocks.test.js`
- Test configuration: `wdio.conf.js`
- Helpers: `test/helpers/app-launcher.js`

### Test Contents

The current test suite includes the following tests:

- Application launch and initialization
- Clock display and updates
- Epoch time display toggle (Ctrl+e, Ctrl+u)
- Timer start, pause, and removal
- Format switching (Ctrl+f)
- Copying to clipboard (Ctrl+c)
- Date-time and Epoch time conversion (Ctrl+v)

## Rust Tests

### Test Locations

Rust tests are located in the following files:

- `src-tauri/src/config.rs` - Tests for config file reading and writing
- `src-tauri/src/web_server.rs` - Tests for web server functionality
- `src-tauri/src/util.rs` - Tests for utility functions

### Running Tests

From the project root directory:

```bash
# Run all tests
cd src-tauri
cargo test

# Or from the project root
cargo test --manifest-path src-tauri/Cargo.toml
```

### Running Specific Tests

```bash
cd src-tauri

# Run tests for a specific module only
cargo test config::tests
cargo test web_server::tests
cargo test util::tests

# Run a specific test function only
cargo test test_get_config_app_path

# Filter by test name
cargo test config
```

### Displaying Test Output

```bash
# Display standard output from tests (see println! output)
cargo test -- --nocapture

# Display output for a specific test
cargo test test_get_config_app_path -- --nocapture
```

### Parallel Test Execution

By default, Rust tests run in parallel. To disable parallel execution:

```bash
cargo test -- --test-threads=1
```

## TIPS

### If Rust Tests Fail

1. **Check Rust version**
   ```bash
   rustc --version
   cargo --version
   ```

2. **Update dependencies**
   ```bash
   cd src-tauri
   cargo update
   ```

3. **Try a clean build**
   ```bash
   cd src-tauri
   cargo clean
   cargo test
   ```
