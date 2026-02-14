# mclocks-datetime-util

An MCP (Model Context Protocol) server that gives AI assistants the ability to answer time/date questions across multiple timezones. It converts between datetime formats and epoch timestamps, calculates days between dates or days until a target date, supports flexible locale settings for weekday names in multiple languages, and integrates with the [mclocks](https://github.com/bayashi/mclocks) desktop clock app configuration.

## Features

7 tools for datetime operations:

| Tool | Description |
|------|-------------|
| **`current-time`** | Get the current time in your configured timezones |
| **`convert-time`** | Convert a datetime string or epoch timestamp to multiple timezones |
| **`next-weekday`** | Find the date of the next occurrence of a given weekday |
| **`date-to-weekday`** | Get the day of the week for a given date |
| **`days-until`** | Count the number of days from today until a specified date |
| **`days-between`** | Count the number of days between two dates |
| **`date-offset`** | Calculate the date N days before or after a given date |

### Highlights

- **Zero configuration** — works out of the box with sensible timezone defaults
- **mclocks integration** — automatically reads your mclocks `config.json` so the timezones you've set up are reflected in AI responses
- **Epoch conversion** — supports seconds, milliseconds, microseconds, and nanoseconds
- **Flexible input** — accepts ISO 8601, common date formats (e.g. `March 15, 2026`), and BigQuery-style datetime (`2024-01-01 12:00:00 UTC`)
- **Locale support** — weekday names in multiple languages via the `locale` setting

## Prerequisites

- [Node.js](https://nodejs.org/) v18 or later

## Setup

Add the following to your MCP configuration file and restart the application. The MCP server will be automatically downloaded and started.

- **Cursor**: `.cursor/mcp.json` in your project root, or global `~/.cursor/mcp.json`
- **Claude Desktop** (`claude_desktop_config.json`): [Windows] `%APPDATA%\Claude\`, [macOS] `~/Library/Application Support/Claude/`, [Linux] `~/.config/Claude/`

```json
{
  "mcpServers": {
    "mclocks-datetime-util": {
      "command": "npx",
      "args": ["-y", "mclocks-datetime-util"]
    }
  }
}
```

## Configuration

### Automatic mclocks config detection

If you use the [mclocks](https://github.com/bayashi/mclocks) desktop app, the MCP server automatically reads your `config.json` and uses:

- **`clocks`** — Timezones from your clocks become default conversion targets
- **`convtz`** — Default source timezone for datetime strings without timezone info
- **`usetz`** — Controls strict timezone conversion mode
- **`locale`** — Language for weekday names (e.g. `ja`, `pt`, `de`)

Config file locations:

- Windows: `C:\Users\{USER}\AppData\Roaming\com.bayashi.mclocks\config.json`
- macOS: `/Users/{USER}/Library/Application Support/com.bayashi.mclocks/config.json`

### Without mclocks

The server works perfectly fine without the mclocks app. When no config is found, it uses these default timezones:

`UTC`, `America/New_York`, `America/Los_Angeles`, `Europe/London`, `Europe/Berlin`, `Asia/Tokyo`, `Asia/Shanghai`, `Asia/Kolkata`, `Australia/Sydney`

### Environment variables

Override config settings (or configure without `config.json`) using environment variables. They take priority over `config.json` values.

| Variable | Description | Default |
|---|---|---|
| `MCLOCKS_CONFIG_PATH` | Path to `config.json` (usually auto-detected) | auto-detect |
| `MCLOCKS_LOCALE` | Locale for weekday names (e.g. `ja`, `pt`, `de`) | `en` |
| `MCLOCKS_CONVTZ` | Default source timezone (e.g. `Asia/Tokyo`) | *(none)* |
| `MCLOCKS_USETZ` | Set to `true` for strict timezone conversion | `false` |

Example with environment variables:

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

You can also specify a custom config path via the `--config` CLI flag:

```json
{
  "mcpServers": {
    "mclocks-datetime-util": {
      "command": "npx",
      "args": ["-y", "mclocks-datetime-util", "--config", "/path/to/config.json"]
    }
  }
}
```

## Example usage

Once configured, you can ask your AI assistant:

- "What time is it now?"
- "What time is it in Jakarta?"
- "Convert 1705312200 epoch to datetime"
- "Convert 2024-01-15T10:30:00Z to Asia/Tokyo"
- "Next Friday is what date?"
- "What day of the week is 2026-12-25?"
- "How many days until Christmas?"
- "How many days between 2026-01-01 and 2026-12-31?"
- "What date is 90 days from 2026-04-01?"

## License

[The Artistic License 2.0](https://github.com/bayashi/mclocks/blob/main/LICENSE)

## Author

Dai Okabayashi: https://github.com/bayashi
