# mclocks

Free desktop clock for multiple time zones🕒🌍🕕

![screenshot](https://raw.githubusercontent.com/bayashi/mclocks/main/screenshot/mclocks-screenshot-0.1.6.png)

## Download

From https://github.com/bayashi/mclocks/releases

For Windows, you can get `.exe` installer `mclocks.Setup.X.X.X.exe`.

For Mac, you can get `.dmg` file `mclocks-X.X.X.dmg` (Perhaps, you need Security setting to install .dmg from github.)

For Linux, not yet.

## config.json

You can set configuration file as `config.json` to configure clocks you prefer.

![screenshot](https://raw.githubusercontent.com/bayashi/mclocks/main/screenshot/mclocks-screenshot-0.1.8-custom.png)

### Location of config.json

* Windows `C:\Users\{USER}\AppData\Roaming\mclocks`
* Mac `/Users/{USER}/Library/Application Support/mclocks`

### Example of config.json

It should be JSON file like below.

    {
      "clocks": [
        { "name": "NY", "timezone": "America/New_York" },
        { "name": "London",   "timezone": "Europe/London" }
      ],
      "formatDateTime": "MM-DD ddd HH:mm",
      "localeDateTime": "en",
      "opacity": 1,
      "fontColor": "#fff",
      "fontSize": 14,
      "bgColor": "#151",
      "onlyText": false,
      "alwaysOnTop": false
    }

Please refer to:

* <https://en.wikipedia.org/wiki/List_of_tz_database_time_zones> for the timezone
* <https://momentjs.com/docs/#/parsing/string-format/> for the date time format

## Window state file

If you want to reset your mclock where it's located in your screen, then please try to remove `window-state.json` in your App data directory. That file would be generated by mclock automatically to keep the location and the size of your mclocks.

## How to close mclock

Windows

* Right click, then select `close`
* Alt + F4

Mac

* Select `Quit` from Dock
