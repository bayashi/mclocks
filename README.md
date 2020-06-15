# mclocks

Multiple timezone clocks🕒🌍🕕

![screenshot](https://raw.githubusercontent.com/bayashi/mclocks/master/screenshot/mclocks-screenshot-0.1.5.png)

## Download

https://github.com/bayashi/mclocks/releases

For Windows, from above URL, you can get installer `mclocks.Setup.X.X.X.exe`.

For Mac, not yet.

For Linux, not yet.

## config.json

You can set configuration file as `config.json` to configure clocks you prefer.

### Location of config.json

* Windows `C:\Users\user\AppData\Roaming\mclocks`

### Example of config.json

It should be JSON file like below.

    {
      "clocks": [
        { "name": "Tokyo", "timezone": "Asia/Tokyo" },
        { "name": "UTC",   "timezone": "UTC" },
        { "name": "PA",   "timezone": "America/Los_Angeles" }
      ],
      "formatDateTime": "MM-DD ddd HH:mm",
      "localeDateTime": "en",
      "opacity": 1,
      "fontColor": "#fff",
      "fontSize": 14,
      "bgColor": "#151",
      "alwaysOnTop": false,
    }

## Window state file

If you want to reset your mclock where it's located in your screen, then please try to remove `window-state.json` in your App data directory. That file would be generated by mclock automatically to keep the location and the size of your mclocks.

## How to close mclock

Windows

* Right click, then select `close`
* Alt + F4
