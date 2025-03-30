# mclocks

The desktop clock application for multiple time zones🕒🌍🕕 

![screenshot](https://raw.githubusercontent.com/bayashi/mclocks/main/screenshot/mclocks-screenshot-0.1.8-custom.png)

## Download

From https://github.com/bayashi/mclocks/releases

### Windows

For Windows, you can get `.msi` or `.exe` installer.

### macOS

For macOS, you can get `.dmg` to install.

### Linux

For Linux, you can get `.deb`, `.rpm` or `.AppImage`.

## config.json

The `config.json` file allows you to configure the clocks according to your preferences.

The `config.json` file should be located in the following directories:

* Windows: `C:\Users\{USER}\AppData\Roaming\mclocks\`
* Mac: `/Users/{USER}/Library/Application Support/mclocks/`
* Linux: `/home/{USER}/.config/mclocks/`

There is no GUI(Graphical User Interface) for editing the `config.json` file in `mclocks`. You will need to open and edit it directly using your text editor.

### Example of config.json

The `config.json` file should be formatted as JSON, as shown below.

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

#### clocks

The `clocks` field is an array of objects, each containing `name` and `timezone` properties. Both should be String.

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

The `format` field is a date-time format string used to display the clock. To create a custom date-time format, please refer to [this formatting guide](https://momentjs.com/docs/#/parsing/string-format/).

#### locale

The `locale` field determines the language settings for displaying the date-time. You can find [a list of supported locales here](https://github.com/kawanet/cdate-locale/blob/main/locales.yml).

#### color

The `color` field defines the color of the date-time text. You can use named colors, RGB hex values, RGB values (e.g., RGB(255, 0, 0)), or any valid CSS color value.

#### font

The `font` is a font name to display date-time. It should be monospaced font. If you would set non-fixed-width font, then your mclocks may have an undesirable wobbling effect.

#### size

The `size` is a size of charactor for date-time, in pixel.

#### margin

The `margin` field determines the space between clocks

#### forefront

If the `forefront` field is set to `true`, the mclocks application will always be displayed on top of other application windows. 

## License

[The Artistic License 2.0](https://github.com/bayashi/mclocks/blob/main/LICENSE)

## Author

Dai Okabayashi: https://github.com/bayashi
