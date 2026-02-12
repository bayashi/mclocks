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
| `package.json` | Root package.json (`mcp` script for local dev) |
| `eslint.config.js` | ESLint config (includes `src-mcp/**/*.js` entry with Node.js globals) |

## Dependencies

The mclocks MCP server uses the following libraries (defined in `src-mcp/package.json`):

| Package | Purpose |
|---------|---------|
| `@modelcontextprotocol/sdk` | MCP protocol SDK (stdio transport) |
| `cdate` | Datetime and timezone conversion |
| `zod` | Input schema validation |

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

### Config integration

The server reads mclocks `config.json` automatically:

- Config path: auto-detected from OS-specific location (Windows: `%APPDATA%`, macOS: `~/Library/Application Support`)
- Override: set `MCLOCKS_CONFIG_PATH` environment variable
- Fields used: `clocks` (default timezones), `convtz` (source timezone), `usetz` (strict TZ mode)
- If no config is found, falls back to built-in timezone list

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
