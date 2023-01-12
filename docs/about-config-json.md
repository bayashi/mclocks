# About config.json

This document supports __mclocks__ `v0.1.19`.

`config.json` of mclocks is a file to keep your configurations.

## Location of config.json

`config.json` file is locating below directory.

* Windows `C:\Users\{USER}\AppData\Roaming\mclocks`
* Mac `/Users/{USER}/Library/Application Support/mclocks`

There is no GUI to edit `config.json`. You have to directly open it by your text editor.

## Example of config.json

`config.json` file should be JSON like below. And these are default values.

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

### Description of each parameter

#### clocks

`clocks` should be an array of objects. The objects must have 2 key and value pairs.

```
"clocks": [
  { "name": "NY", "timezone": "America/New_York" },
  { "name": "London",   "timezone": "Europe/London" }
],
```
The object keys are `name` and `timezone`.

```
{ "name": "NY", "timezone": "America/New_York" }
```

The value of `name` is free text that you can set it as you prefer. It going to be shown on screen. The value of `timezone` is the time zone for the clock. Please [refer to this list](https://en.wikipedia.org/wiki/List_of_tz_database_time_zones) to pick time zones up.

#### formatDateTime

You can set a format of datetime string. Please [refer to this table](https://momentjs.com/docs/#/parsing/string-format/) to build formatted text.

```
"formatDateTime": "MM-DD ddd HH:mm",
```

#### localeDateTime

If you show the name of the day or month, ex. `Sunday` or `January`, you should set locale to translate it.

```
"localeDateTime": "en",
```

Please [refer to this list](https://raw.githubusercontent.com/kawanet/cdate-locale/main/locales.yml) for the locales.

#### opacity

The opacity of the window between 0.0 (fully transparent) and 1.0 (fully opaque).

```
"opacity": 1,
```

#### fontColor

The font color of the clock text. You can set this value by several ways:

* hex-color: `#fff`
* named-color: `white`
* rgb(): `rgb(255,255,255)`

```
"fontColor": "#fff",
```

#### fontSize

The font size of the clock text. The unit of value is "px".

```
"fontSize": 14,
```

#### bgColor

The background color of the clock.

```
"bgColor": "#151",
```

#### onlyText

If you want to show clock without background, then you can set `true` for `onlyText`.

```
"onlyText": false,
```

#### alwaysOnTop

Set `true` when you want the window of mclocks should always stay on top of other windows.

```
"alwaysOnTop": false
```



