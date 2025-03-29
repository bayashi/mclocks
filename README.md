# mclocks

The desktop clock application for multiple time zones🕒🌍🕕 

![screenshot](https://raw.githubusercontent.com/bayashi/mclocks/main/screenshot/mclocks-screenshot-0.1.8-custom.png)

## Download

From https://github.com/bayashi/mclocks/releases

### Windows OS

For Windows, you can get `.msi` or `.exe` installer.

### Mac OS

For Mac, you can get `.dmg` to install.

### Linux

For Linux, you can get `.deb`, `.rpm` or `.AppImage`.

## config.json

You can set `config.json` file to configure clocks as you prefer.

`config.json` file is locating below directory.

* Windows: `C:\Users\{USER}\AppData\Roaming\mclocks\`
* Mac: `/Users/{USER}/Library/Application Support/mclocks/`
* Linux: `/home/{USER}/.config/mclocks/`

There is no GUI to edit `config.json` in `mclocks`. You have to directly open and edit by your text editor.

### Example of config.json

`config.json` file should be JSON like below.

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

Please [refer to this list](https://en.wikipedia.org/wiki/List_of_tz_database_time_zones) to pick up time zones you need.

Please [refer to this table](https://momentjs.com/docs/#/parsing/string-format/) to build date time formatted text.

## License

[The Artistic License 2.0](https://github.com/bayashi/mclocks/blob/main/LICENSE)

## Author

Dai Okabayashi: https://github.com/bayashi
