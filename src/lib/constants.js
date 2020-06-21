const defaultConfig = {
  clocks: [
    { name: "NY", timezone: "America/New_York" },
    { name: "London",   timezone: "Europe/London" },
  ],
  formatDateTime: "MM-DD ddd HH:mm",
  localeDateTime: "en",
  opacity: 1.0,
  fontColor: '#fff',
  fontSize: 14,
  bgColor: '#161',
  alwaysOnTop: false,
};

  // https://github.com/sindresorhus/electron-store#schema
const clocksSchema = {
  type: "array",
  minItems: 1,
  maxItems: 10,
  items: {
    type: "object",
    minProperties: 2,
    maxProperties: 2,
    properties: {
      name: {
        type: "string",
        minLength: 1,
        maxLength: 10,
        regexp: '/^[a-z0-9\-]+$/',
      },
      // https://en.wikipedia.org/wiki/List_of_tz_database_time_zones
      timezone: {
        type: "string",
        minLength: 1,
        maxLength: 27,
        regexp: '/^[a-z0-9\/\_]+$/',
      },
    },
  },
};

const optionsSchema = {
  // https://momentjs.com/docs/#/parsing/string-format/
  formatDateTime: {
    type: "string",
    minLength: 1,
    maxLength: 50,
  },
  // https://github.com/moment/moment/tree/develop/locale
  localeDateTime: {
    type: "string",
    regexp: '/[a-z]+(-[a-z]+)?/',
    minLength: 2,
    maxLength: 8,
  },
  opacity: {
    type: "number",
    minimum: 0.1,
    maximum: 1.0,
  },
  fontColor: {
    type: "string",
    regexp: '/^#[a-fA-F0-9]+$/',
    minLength: 4,
    maxLength: 7,
  },
  fontSize: {
    type: "number",
    minimum: 8,
    maximum: 36,
  },
  bgColor: {
    type: "string",
    regexp: '/^#[a-fA-F0-9]+$/',
    minLength: 4,
    maxLength: 7,
  },
  alwaysOnTop: {
    type: "boolean",
  },
};

module.exports = {
  defaultConfig: defaultConfig,
  clocksSchema: clocksSchema,
  optionsSchema: optionsSchema,
};
