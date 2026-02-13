# How to Develop mclocks MCP Server

This document explains how to develop and maintain the MCP (Model Context Protocol) server of mclocks.

## Overview

The MCP server (`mclocks-datetime-util`) provides datetime and timezone conversion tools to AI assistants like Cursor. It is published as a standalone npm package, separate from the main mclocks Tauri application.

For end-user setup instructions, see the **MCP Server** section in the main [README.md](../README.md).

## Related Files

| File | Description |
|------|-------------|
| `src-mcp/server.js` | MCP server implementation (single file) |
| `src-mcp/package.json` | npm package definition for `mclocks-datetime-util` |
| `src-mcp/test/mcp-tools.test.js` | MCP tool tests (Mocha + MCP Client) |
| `package.json` | Root package.json (`mcp`, `test:mcp` scripts for local dev) |
| `eslint.config.js` | ESLint config (includes `src-mcp/**/*.js` entry with Node.js globals) |

## Dependencies

The mclocks MCP server uses the following libraries (defined in `src-mcp/package.json`):

| Package | Purpose |
|---------|---------|
| `@modelcontextprotocol/sdk` | MCP protocol SDK (stdio transport, also used by tests as client) |
| `cdate` | Datetime and timezone conversion |
| `zod` | Input schema validation |

Dev dependencies (for testing):

| Package | Purpose |
|---------|---------|
| `mocha` | Test runner |

These are independent from the root `package.json` dependencies. The root package.json does NOT include MCP-specific dependencies.

## Local Development

### Install dependencies

```bash
cd src-mcp
npm install
```

### Run the MCP server locally

From the project root:

```bash
pnpm mcp
```

Or directly:

```bash
node src-mcp/server.js
```

The server communicates via stdio (stdin/stdout), so running it directly will wait for JSON-RPC input. This is mainly useful for verifying that the server starts without errors.

### Configure Cursor to use the local server

For development, point Cursor to the local `server.js` instead of the published npm package.

Add to `.cursor/mcp.json`:

```json
{
  "mcpServers": {
    "mclocks-datetime-util": {
      "command": "node",
      "args": ["C:\\path\\to\\mclocks\\src-mcp\\server.js"]
    }
  }
}
```

After saving, restart Cursor to pick up the changes.

### Lint

```bash
pnpm exec eslint .
```

The ESLint config (`eslint.config.js`) includes a dedicated entry for `src-mcp/**/*.js` with Node.js globals enabled.

### Test

```bash
pnpm test:mcp
```

Or directly:

```bash
cd src-mcp
npm test
```

Tests use the MCP Client SDK to launch the server as a subprocess and call tools via stdio. The test environment sets `MCLOCKS_CONFIG_PATH` to a non-existent path to ensure consistent fallback defaults regardless of the developer's machine config.

## mclocks MCP Tools

The server exposes the following tools:

### `convert-time`

Converts a datetime string or epoch timestamp to multiple timezones.

Parameters:
- `source` (required) - Datetime string or epoch number
- `timezones` (optional) - Target timezone array
- `source_timezone` (optional) - Source timezone for interpretation
- `epoch_unit` (optional) - `"seconds"` | `"milliseconds"` | `"microseconds"` | `"nanoseconds"`

### `current-time`

Returns the current time in specified timezones.

Parameters:
- `timezones` (optional) - Target timezone array

### `next-weekday`

Finds the date of the next occurrence of a given weekday from today.

Parameters:
- `weekday` (required) - Day of the week in English (e.g. `"Monday"`, `"fri"`, `"tu"`)
- `timezone` (optional) - Timezone to determine "today"

### `date-to-weekday`

Returns the day of the week for a given date.

Parameters:
- `date` (required) - Date string (e.g. `"2026-02-20"`, `"March 15, 2026"`)

### `days-until`

Counts the number of days from today until a specified date. Supports smart defaults when year/month/day are omitted.

Parameters:
- `year` (optional) - If omitted, uses current year (or next year if date has passed)
- `month` (optional) - If omitted, defaults to January (or current month when only day is specified)
- `day` (optional) - If omitted, defaults to 1
- `timezone` (optional) - Timezone to determine "today"

### `days-between`

Counts the number of days between two dates. The start date is not included in the count.

Parameters:
- `from` (required) - Start date string
- `to` (required) - End date string

### Config integration

The server reads mclocks `config.json` automatically:

- Config path resolution (highest priority first):
  1. `--config <path>` command-line argument
  2. `MCLOCKS_CONFIG_PATH` environment variable
  3. Auto-detect from OS-specific location (Windows: `%APPDATA%`, macOS: `~/Library/Application Support`)
- Fields used from config: `clocks` (default timezones), `convtz` (source timezone), `usetz` (strict TZ mode), `locale` (weekday name localization)
- If no config is found, falls back to built-in defaults

#### Environment variable overrides

The following environment variables override individual config fields (env > config.json > fallback):

| Variable | Overrides | Default |
|----------|-----------|---------|
| `MCLOCKS_LOCALE` | `locale` | `"en"` |
| `MCLOCKS_CONVTZ` | `convtz` | `""` |
| `MCLOCKS_USETZ` | `usetz` | `false` (set `"true"` to enable) |

This allows configuring the MCP server via `mcp.json` without requiring the mclocks app config:

```json
{
  "mcpServers": {
    "mclocks-datetime-util": {
      "command": "npx",
      "args": ["-y", "mclocks-datetime-util"],
      "env": {
        "MCLOCKS_LOCALE": "ja",
        "MCLOCKS_CONVTZ": "Asia/Tokyo"
      }
    }
  }
}
```

## Updating Dependencies

```bash
cd src-mcp
npm update
```

To check for outdated packages:

```bash
cd src-mcp
npm outdated
```

After updating, verify the server starts correctly and test the tools in Cursor.

## Publishing to npm

### First-time setup

```bash
npm login
```

### Publish

```bash
cd src-mcp
npm publish
```

### Version bump

Update the version in `src-mcp/package.json` before publishing:

```bash
cd src-mcp
npm version patch  # or minor, major
npm publish
```

Note: The version in `src-mcp/package.json` is independent from the mclocks app version in the root `package.json`.

### Verify the published package

```bash
npx mclocks-datetime-util
```

Or check on npm: https://www.npmjs.com/package/mclocks-datetime-util
