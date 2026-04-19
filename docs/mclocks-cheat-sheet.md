# mclocks cheat sheet

Quick reference for keyboard shortcuts, the full `config.json` shape (with defaults).

## Keyboard Shortcuts

🔔 NOTE: Keys are shown for Windows; on macOS read `Ctrl` as `Command` and `Alt` as `Option` where appropriate.

### Show Help

| Shortcut | Description |
|----------|-------------|
| `F1` (Windows) or `Cmd + Shift + /` (macOS) | Open this cheat sheet in the browser |

### Configuration, Display Formats

| Shortcut | Description |
|----------|-------------|
| `Ctrl + o` | Open `config.json` file in editor |
| `Ctrl + f` | Switch between `format` and `format2` (if `format2` is defined in `config.json`) |
| `Ctrl + e` or `Ctrl + u` | Toggle to display Epoch time |

### Timer

| Shortcut | Description |
|----------|-------------|
| `Ctrl + 1` to `Ctrl + 9` | Start timer (1 minute × number key) |
| `Ctrl + Alt + 1` to `Ctrl + Alt + 9` | Start timer (10 minutes × number key) |
| `Ctrl + p` | Pause / resume all timers |
| `Ctrl + 0` | Delete the oldest timer |
| `Ctrl + Alt + 0` | Delete the newest timer |

### Sticky Note

| Shortcut | Description |
|----------|-------------|
| `Ctrl + s` | Create a new sticky note from clipboard text |

### Clipboard datetime Operations

| Shortcut | Description |
|----------|-------------|
| `Ctrl + c` | Copy current mclocks text to clipboard |
| `Ctrl + v` | Convert clipboard content (Epoch time as seconds, or date-time) |
| `Ctrl + Alt + v` | Convert clipboard content (Epoch time as milliseconds) |
| `Ctrl + Alt + Shift + V` | Convert clipboard content (Epoch time as microseconds) |
| `Ctrl + Alt + Shift + N + V` | Convert clipboard content (Epoch time as nanoseconds) |

### Text Conversion

| Shortcut | Description |
|----------|-------------|
| `Ctrl + i` | Quote each line of clipboard text with double quotes, append comma to the end (except the last line), and open in editor |
| `Ctrl + Shift + i` | Append comma to the end of each line (no quotes) for INT list IN condition (except the last line), and open in editor |
| `Ctrl + t` | Convert clipboard Excel/TSV text to Markdown table and open in editor |
| `Ctrl + Shift + t` | Open a Markdown table template in editor |

## `config.json` overview

Full config keys and default values.

```jsonc
{
  // Time zone rows to show. With {} the default is one UTC row
  "clocks": [
    {
      // Row label
      "name": "UTC",
      // IANA time zone name
      // https://en.wikipedia.org/wiki/List_of_tz_database_time_zones
      "timezone": "UTC",
      // Countdown target datetime, etc.
      "countdown": null,
      // Target datetime, etc.
      "target": null,
    },
  ],

  // String passed to CSS font-family
  "font": "Courier, monospace",
  // Font size: number or string like "14px"
  "size": 14,
  // Text color as CSS color
  "color": "#fff",
  // chrono-style date/time format string
  // https://momentjs.com/docs/#/parsing/string-format/
  "format": "MM-DD ddd HH:mm",
  // Second format toggled with Ctrl+f (omit for no toggle)
  "format2": null,
  // Locale string
  // https://github.com/kawanet/cdate-locale/blob/main/locales.yml
  "locale": "en",
  // Always on top of other windows
  "forefront": false,
  // Line spacing / margin as CSS length
  "margin": "1.65em",
  // Icon prefix for timer rows
  "timerIcon": "⧖ ",
  // If true, skip OS notifications when timers finish
  "withoutNotification": false,
  // Max concurrent timer rows
  "maxTimerClockNumber": 5,
  // Label for the Epoch row when toggled
  "epochClockName": "Epoch",
  // If true, append time zone abbreviations on each clock row
  "usetz": false,
  // When non-empty, time zone used for conversion/display
  "convtz": "",
  // If true, disable hover tooltips
  "disableHover": true,

  // Optional web configuration for static hosting and related features
  "web": {
    // Absolute path of directory to serve (required when using `web` feature)
    "root": "/path/to/your/webroot",
    // Preferred main HTTP port (>= 2000, default 3030; if key omitted, pick free port downward)
    "port": 3030,
    // Open server URL in browser on launch
    "openBrowserAtStart": false,
    // Enable `/dump` request echo endpoint
    "dump": false,
    // Enable `/slow` delayed response endpoint
    "slow": false,
    // Enable `/status/{code}` arbitrary status endpoint
    "status": false,
    // Markdown and related content options
    "content": {
      "markdown": {
        // Allow raw HTML inside Markdown
        "allowRawHTML": false,
        // Open external Markdown links in a new tab
        "openExternalLinkInNewTab": true,
        // Enable POST `/preview` for CLI Markdown preview
        "enablePreviewApi": false,
      },
    },
    // Actual assets listen port is derived from the main port when `web` from config is active
    "assets": {
      "port": 0,
    },
    // `/editor`: open local files from GitHub-style URLs
    "editor": {
      // Parent directory of local clones (/editor disabled if unset)
      "reposDir": null,
      // If true, include original host as a path segment
      "includeHost": false,
      // Command to launch the editor
      "command": "code",
      // Argument list for the editor command
      "args": ["-g", "{file}:{line}"],
    },
  },
}
```
