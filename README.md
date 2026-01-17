# mclocks

The desktop clock application for multiple time zones🕒🌍🕕 

![screenshot](https://raw.githubusercontent.com/bayashi/mclocks/main/screenshot/mclocks-screenshot-0.1.8-custom.png)

In addition, it also includes features:

* Timer
* Countdown timer
* Epoch time and date-time convertor
* Sticky notes
* Web server for developers (serves static files and provides dump and slow endpoints for debugging)

`mclocks` doesn't need an internet connection — everything runs 100% locally.

## Download

From https://github.com/bayashi/mclocks/releases

### Windows

For Windows, you can get the installer `.msi` file, or `.exe` the executable file.

### macOS

For macOS, you can get `.dmg` file to install.

(The shortcut keys in this document are for Windows OS. If you are using macOS, please interpret them accordingly, replacing keys such as `Ctrl` with `Ctrl + Command` and `Alt` with `Option` where appropriate.)

### Linux

For Linux, you can get `.deb`, `.rpm` or `.AppImage` file to install.

## config.json

The `config.json` file allows you to configure the clocks according to your preferences.

The `config.json` file should be located in the following directories:

* Windows: `C:\Users\{USER}\AppData\Roaming\com.bayashi.mclocks\`
* Mac: `/Users/{USER}/Library/Application Support/com.bayashi.mclocks/`
* Linux: `/home/{USER}/.config/com.bayashi.mclocks/`

When you start `mclocks`, then press `Ctrl + o` to edit your `config.json` file.

### Backwards Compatibility Notes

The directory of the `config.json` file has been changed to `com.bayashi.mclocks` from just `mclocks` after version 0.2.9.

And after version 0.2.13, old `config.json` file is automatically migrated into new directory if the new config file doesn't exist.

### Example of config.json

The `config.json` file should be formatted as JSON, as shown like below.

    {
      "clocks": [
        { "name": "UTC", "timezone": "UTC" }
      ],
      "format": "MM-DD ddd HH:mm",
      "locale": "en",
      "color": "#fff",
      "font": "Courier, monospace",
      "size": 14,
      "margin": "1.65em",
      "forefront": false
    }

## The fields of config.json

#### clocks

The `clocks` field is an array of objects, each containing `name` and `timezone` properties. Both should be String. By default, both are `UTC`.

* `name` is a label that will be displayed for the clock.
* For selecting time zones, please refer to this [list of time zones](https://en.wikipedia.org/wiki/List_of_tz_database_time_zones).

Here is an example of a `clocks` array for three time zones.

    {
      "clocks": [
        { "name": "Tokyo", "timezone": "Asia/Tokyo" },
        { "name": "UTC", "timezone": "UTC" },
        { "name": "SF", "timezone": "America/Los_Angeles" }
      ],
      "format": "MM-DD ddd HH:mm",
      ...snip...
    }

#### format

string: `MM-DD ddd HH:mm`

The `format` field is a date-time format string used to display the clock. To create a custom date-time format, please refer to [this formatting guide](https://momentjs.com/docs/#/parsing/string-format/).

#### format2

string: `""`

The `format2` field is same as `format`. These are switched each other by `Ctrl + f` key. The `format2` is optional field.

#### locale

string: `en`

The `locale` field determines the language settings for displaying the date-time. You can find [a list of supported locales here](https://github.com/kawanet/cdate-locale/blob/main/locales.yml).

#### color

string: `#fff`

The `color` field defines the color of the date-time text. You can use named colors, RGB hex values, RGB values (e.g., `RGB(255, 0, 0)`), or any valid CSS color value.

#### font

string: `Courier, monospace`

The `font` is a font name to display date-time. It should be monospaced font. If you would set non-fixed-width font, then your mclocks may have an undesirable wobbling effect.

#### size

number | string: 14

The `size` is a size of charactor for date-time, in pixel. It can also be specified as a string that includes a unit (e.g., `"125%"`, `"1.5em"`).

#### margin

string: `1.65em`

The `margin` field determines the space between clocks

#### forefront

bool: `false`

If the `forefront` field is set to `true`, the mclocks application will always be displayed on top of other application windows. 

## Countdown clock

By setting up the config as shown below for the `clock`, it will be displayed as a countdown clock to a given `target` date-time.

	"clocks": [
		{
			"countdown": "WAC Tokyo D-%D %h:%m:%s",
			"target": "2025-09-13",
			"timezone": "Asia/Tokyo"
		}
	],

Above countdown `clock` will be displayed like below:

    WAC Tokyo D-159 12:34:56

Indicating 159 days, 12 hours, 34 minutes, and 56 seconds left until September 13, 2025.

### Countdown format verbs

The `countdown` fieled text accepts below template verbs:

* `%TG`: Target date-time string
* `%D`: Remaining day count to target date-time
* `%H`: Remaining time as hour to target date-time
* `%h`: An hour(hh) of remaining time (hh:mm:ss)
* `%M`: Remaining time as minute to target date-time
* `%m`: A minute(mm) of remaining time (hh:mm:ss)
* `%S`: Remaining time as second to target date-time
* `%s`: A second(ss) of remaining time (hh:mm:ss)

## Simple Timer

![simple timer](https://raw.githubusercontent.com/bayashi/mclocks/main/screenshot/mclocks-screenshot-timer.png)

Click `mclocks` app window, then push `Ctrl + 1` key, then start 1-minute timer. Push `Ctrl + Alt + 1` key, start 10-minute timer. Other number keys work as well. Starting timers up to 5.

`Ctrl + p` to pause / re-start the timers.

`Ctrl + 0` to delete the oldest timer. `Ctrl + Alt + 0` to delete the newest timer.

NOTE: Countdown clock and simple timer will send notification by default when the timer is complete. If you don't need notifications, set `withoutNotification: true` in `config.json`.

## Display Epoch time

![epoch-time](https://raw.githubusercontent.com/bayashi/mclocks/main/screenshot/mclocks-screenshot-epoch-time.png)

Click `mclocks` app window, then push `Ctrl + e` key, then toggle to display Epoch time.

## Convert between date-time and Epoch time

Click `mclocks` app window, then paste a date-time or Epoch time, then a dialog appears to display conversion results. And it's able to copy the results to the clipboard. If you don't want to copy, then press `[No]` to just close the dialog.

When pasting with `Ctrl + v`, the value (Epoch time) is treated as seconds. If you use `Ctrl + Alt + v`, it's treated as milliseconds, and with `Ctrl + Alt + Shift + V`, it's treated as microseconds, and with `Ctrl + Alt + Shift + N + V`, it's treated as nanoseconds and converted accordingly.

![convert-from-epoch-to-datetime](https://raw.githubusercontent.com/bayashi/mclocks/main/screenshot/mclocks-screenshot-converter-epoch.png)

![convert-from-datetime-to-epoch](https://raw.githubusercontent.com/bayashi/mclocks/main/screenshot/mclocks-screenshot-converter-date.png)

When pasted date-time values do not include timezone information, they are treated as local timezone by default. To handle them as a specific timezone, set the timezone in the convtz option.

    "convtz": "UTC"

## Convert Text Feature

Click `mclocks` app window, then use the following keyboard shortcuts to process clipboard text and open it in an editor:

* `Ctrl + i`: Quotes each line of clipboard text with double quotes and appends a comma to the end (except the last line)
* `Ctrl + Shift + i`: Quotes each line of clipboard text with single quotes and appends a comma to the end (except the last line)

Empty lines are preserved as-is in all operations.

(This Convert Text Feature has nothing to do with clocks or time, but software developers might find it handy! 😊)

## Sticky Notes

Click `mclocks` app window, then press `Ctrl + s` to create a sticky note from the clipboard text. Each sticky note opens in a separate window.

Each sticky note's text is limited to 1KB (1024 bytes). Longer text is automatically truncated.

### Closing Sticky Notes

Click the `×` button in the top-right corner of a sticky note window to close it. The note will be removed from the saved data.

## Keyboard Shortcuts

### Show Help

`F1` (Windows) or `Cmd + Shift + /` (macOS) to open help page (this README) in browser

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
| `Ctrl + 0` | Delete the oldest timer (leftmost) |
| `Ctrl + Alt + 0` | Delete the newest timer (rightmost) |

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
| `Ctrl + Shift + i` | Quote each line of clipboard text with single quotes, append comma to the end (except the last line), and open in editor |

### Sticky Notes

| Shortcut | Description |
|----------|-------------|
| `Ctrl + s` | Create a sticky note from clipboard text |

## Web Server

`mclocks` can serve static files via a built-in web server. This feature allows you to easily view your code snippets in a browser. Add a `web` field to your `config.json`:

    {
      "web": {
        "root": "/path/to/your/webroot",
        "dump": true,
        "slow": true,
        "status": true
      }
    }

* `root`: Path to the directory containing files to serve (required)
* `port`: Port number to listen on (default: 3030)
* `open_browser_at_start`: If set to `true`, automatically opens the web server URL in the default browser when `mclocks` starts (default: `false`)
* `dump`: If set to `true`, enables the `/dump` endpoint that returns request details as JSON (default: `false`)
* `slow`: If set to `true`, enables the `/slow` endpoint that delays the response (default: `false`)
* `status`: If set to `true`, enables the `/status/{code}` endpoint that returns arbitrary HTTP status codes (default: `false`)

If the `web` field is configured in your `config.json`, the web server starts automatically when `mclocks` launches. Access files at `http://127.0.0.1:3030`. The web server only listens on `127.0.0.1` (localhost), so it is only accessible from your local machine.

### Supported file types

The web server supports the following file types:

* Text: `html`, `css`, `js`, `json`, `md`, `txt`
* Images: `png`, `jpg`, `jpeg`, `gif`, `svg`, `ico`

### /dump endpoint

When `dump: true` is set in the `web` configuration, the web server provides a `/dump` endpoint that returns request details as JSON.

The endpoint responds with a JSON object containing:
* `method`: HTTP method (e.g., "GET", "POST")
* `path`: Request path after `/dump/` (e.g., "/test" for `/dump/test`)
* `query`: Query parameters as an array of key-value objects (e.g., `[{"key1": "value1"}, {"key2": "value2"}]`)
* `headers`: Request headers as an array of key-value objects (e.g., `[{"Content-Type": "application/json"}]`)
* `body`: Request body as a string (if present)
* `parsed_body`: Parsed JSON object if Content-Type indicates JSON, or error message string if parsing fails

Access the dump endpoint at `http://127.0.0.1:3030/dump` or any path under `/dump/` (e.g., `/dump/test?key=value`).

### /slow endpoint

When `slow: true` is set in the `web` configuration, the web server provides a `/slow` endpoint that delays the response before returning 200 OK.

The endpoint is accessible via any HTTP method (GET, POST, etc.) and supports the following paths:

* `/slow`: Waits 30 seconds (default) and returns 200 OK
* `/slow/120`: Waits 120 seconds (or any specified number of seconds) and returns 200 OK

This endpoint is useful for testing timeout behavior, connection handling, or simulating slow network conditions.

Examples:
* `http://127.0.0.1:3030/slow` - waits 30 seconds
* `http://127.0.0.1:3030/slow/60` - waits 60 seconds
* `http://127.0.0.1:3030/slow/120` - waits 120 seconds

If an invalid seconds parameter is provided (e.g., `/slow/abc`), the endpoint returns a 400 Bad Request error.

### /status endpoint

When `status: true` is set in the `web` configuration, the web server provides a `/status/{code}` endpoint that returns arbitrary HTTP status codes defined in RFC standards (100-599).

The endpoint returns a plain text response with the status code and its corresponding phrase, along with appropriate headers as required by the HTTP specification.

**Examples:**
* `http://127.0.0.1:3030/status/200` - returns 200 OK
* `http://127.0.0.1:3030/status/404` - returns 404 Not Found
* `http://127.0.0.1:3030/status/500` - returns 500 Internal Server Error
* `http://127.0.0.1:3030/status/418` - returns 418 I'm a teapot (with special message)
* `http://127.0.0.1:3030/status/301` - returns 301 Moved Permanently (with Location header)

**Status-specific headers:**

The endpoint automatically adds appropriate headers for specific status codes:

* **3xx Redirection** (301, 302, 303, 305, 307, 308): Adds `Location` header
* **401 Unauthorized**: Adds `WWW-Authenticate` header
* **405 Method Not Allowed**: Adds `Allow` header
* **407 Proxy Authentication Required**: Adds `Proxy-Authenticate` header
* **416 Range Not Satisfiable**: Adds `Content-Range` header
* **426 Upgrade Required**: Adds `Upgrade` header
* **429 Too Many Requests**: Adds `Retry-After` header (60 seconds)
* **503 Service Unavailable**: Adds `Retry-After` header (60 seconds)
* **511 Network Authentication Required**: Adds `WWW-Authenticate` header

**Response body handling:**

* **204 No Content** and **304 Not Modified**: Returns empty response body (as per HTTP specification)
* **418 I'm a teapot**: Returns special message "I'm a teapot" instead of standard status phrase
* **All other status codes**: Returns plain text in format `{code} {phrase}` (e.g., "404 Not Found")

This endpoint is useful for testing how your applications handle different HTTP status codes, error handling, redirects, authentication requirements, and rate limiting scenarios.

----------

## License

[The Artistic License 2.0](https://github.com/bayashi/mclocks/blob/main/LICENSE)

## Author

Dai Okabayashi: https://github.com/bayashi
