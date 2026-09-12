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

E2E tests use **WebdriverIO**. Specs and `wdio.conf.js` stay in this repository; the WDIO npm toolchain lives in a separate repo: [bayashi/mclocks-e2e](https://github.com/bayashi/mclocks-e2e).

### CI and mclocks-e2e pin

GitHub Actions (`.github/workflows/e2e.yaml`) checks out **mclocks-e2e at a pinned commit SHA**, not the floating `main` tip. That keeps E2E reproducible when mclocks-e2e moves independently.

When you intentionally adopt a newer mclocks-e2e revision, bump the `ref` under the "Checkout mclocks-e2e" step in `e2e.yaml` (and open a mclocks PR for that bump).

Local runs still use your local mclocks-e2e checkout (sibling / `./mclocks-e2e` / `MCLOCKS_E2E_ROOT`); they are not tied to the CI pin.

### Prerequisites for Running Tests

1. Clone `mclocks-e2e` next to `mclocks` (or under `./mclocks-e2e`), then install it:

   ```bash
   # sibling layout (recommended)
   cd ..
   git clone git@github.com:bayashi/mclocks-e2e.git
   cd mclocks-e2e
   pnpm install
   cd ../mclocks
   ```

   Or set `MCLOCKS_E2E_ROOT` to the e2e checkout path.

2. **Start the application** before running tests (separate terminal):

   ```bash
   # WebdriverIO hits Vite in Chrome; clipboard uses a test-only mock (vite --mode e2e)
   pnpm dev:e2e
   ```

   Alternatively, run the full desktop app (real Tauri clipboard):

   ```bash
   pnpm tauri dev
   ```

3. Verify that the application is running at `http://localhost:1420`.

### Running Tests

With the application running, execute the following command from another terminal (mclocks root):

```bash
# Run tests in normal mode
pnpm test

# Run tests in headless mode (browser not displayed)
pnpm test:headless
```

You can also run from the e2e repo (`MCLOCKS_ROOT` defaults to `../mclocks`):

```bash
cd ../mclocks-e2e
pnpm test
```

### Test File Locations

- Test files: `test/specs/`
- Test configuration: `wdio.conf.js`
- Helpers: `test/helpers/`
- WDIO runner package: [mclocks-e2e](https://github.com/bayashi/mclocks-e2e)

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
