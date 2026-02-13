#!/usr/bin/env node

import { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { StdioServerTransport } from "@modelcontextprotocol/sdk/server/stdio.js";
import { z } from "zod";
import { cdate } from "cdate";
import { readFileSync } from "fs";
import { join } from "path";

const APP_IDENTIFIER = "com.bayashi.mclocks";
const CONFIG_FILE = "config.json";

const FALLBACK_TIMEZONES = [
  "UTC",
  "America/New_York",
  "America/Los_Angeles",
  "Europe/London",
  "Europe/Berlin",
  "Asia/Tokyo",
  "Asia/Shanghai",
  "Asia/Kolkata",
  "Australia/Sydney",
];

const EPOCH_UNITS = {
  seconds: 1000,
  milliseconds: 1,
  microseconds: 1 / 1000,
  nanoseconds: 1 / 1000 / 1000,
};

const WEEKDAY_MAP = {
  sunday: 0, sun: 0, su: 0,
  monday: 1, mon: 1, mo: 1,
  tuesday: 2, tue: 2, tu: 2,
  wednesday: 3, wed: 3, we: 3,
  thursday: 4, thu: 4, th: 4,
  friday: 5, fri: 5, fr: 5,
  saturday: 6, sat: 6, sa: 6,
};

// Return the OS-specific config directory (same as Rust `directories::BaseDirs::config_dir`)
function configDir() {
  if (process.platform === "win32") {
    return process.env.APPDATA;
  } else if (process.platform === "darwin") {
    return `${process.env.HOME}/Library/Application Support`;
  }
  return null;
}

// Auto-detect the mclocks config.json path
function autoDetectConfigPath() {
  const dir = configDir();
  if (!dir) {
    return null;
  }
  return join(dir, APP_IDENTIFIER, CONFIG_FILE);
}

// Resolve config path from --config arg, MCLOCKS_CONFIG_PATH env, or auto-detect
function resolveConfigPath() {
  const argIdx = process.argv.indexOf("--config");
  if (argIdx !== -1 && process.argv[argIdx + 1]) {
    return process.argv[argIdx + 1];
  }
  return process.env.MCLOCKS_CONFIG_PATH || autoDetectConfigPath();
}

// Load mclocks config.json
// Priority: --config arg > MCLOCKS_CONFIG_PATH env > auto-detect > null
function loadMclocksConfig() {
  const configPath = resolveConfigPath();
  if (!configPath) {
    return null;
  }
  try {
    const raw = readFileSync(configPath, "utf-8");
    return JSON.parse(raw);
  } catch {
    return null;
  }
}

// Extract unique timezones from clocks config
function extractTimezones(config) {
  if (!config?.clocks || !Array.isArray(config.clocks)) {
    return null;
  }
  const tzSet = new Set();
  for (const clock of config.clocks) {
    if (clock.timezone?.length > 0) {
      tzSet.add(clock.timezone);
    }
  }
  return tzSet.size > 0 ? Array.from(tzSet) : null;
}

const mclocksConfig = loadMclocksConfig();
const configTimezones = extractTimezones(mclocksConfig);
const defaultTimezones = configTimezones || FALLBACK_TIMEZONES;
// Per-field override: env > config.json > fallback
const configUseTZ = process.env.MCLOCKS_USETZ === "true" || (mclocksConfig?.usetz ?? false);
const configConvTZ = process.env.MCLOCKS_CONVTZ || mclocksConfig?.convtz || "";
const configLocale = process.env.MCLOCKS_LOCALE || mclocksConfig?.locale || "en";

// Parse weekday name to day-of-week number (0=Sunday, 6=Saturday)
function parseWeekday(input) {
  const key = input.trim().toLowerCase();
  return key in WEEKDAY_MAP ? WEEKDAY_MAP[key] : null;
}

// Normalize datetime strings for common formats that may fail to parse
function normalizeDT(src) {
  // BQ datetime format: "2024-01-01 12:00:00 UTC" -> "2024-01-01 12:00:00Z"
  const m = src.match(/^(\d\d\d\d-\d\d-\d\d \d\d:\d\d:\d\d(?:\.\d+)?) UTC$/);
  if (m) {
    return m[1] + "Z";
  }
  return src;
}

// Convert a datetime value to a specific timezone
function convertToTimezone(cdt, src, tz, usetz) {
  try {
    let result;
    if (usetz) {
      result = cdt(src).tz(tz).text();
    } else {
      const offset = cdt().tz(tz).utcOffset();
      result = cdt(src).utcOffset(offset).text();
    }
    return { timezone: tz, result };
  } catch (error) {
    return { timezone: tz, error: String(error) };
  }
}

const server = new McpServer({
  name: "mclocks-datetime-util",
  version: "0.1.0",
});

server.tool(
  "convert-time",
  "Convert a datetime string or epoch timestamp to multiple timezones. " +
  "Accepts ISO 8601 datetime, common date formats, or epoch numbers.",
  {
    source: z.string().describe(
      "The source value to convert. Can be a datetime string (e.g. '2024-01-15T10:30:00Z', '2024-01-15 10:30:00 UTC') or an epoch number (e.g. '1705312200')."
    ),
    timezones: z.array(z.string()).optional().describe(
      "Target timezones to convert to (e.g. ['Asia/Tokyo', 'America/New_York']). If omitted, uses timezones from mclocks config or built-in defaults."
    ),
    source_timezone: z.string().optional().describe(
      "Timezone of the source datetime for interpretation (e.g. 'Asia/Tokyo'). Only used when source is a datetime string without timezone info. If omitted, uses convtz from mclocks config if available."
    ),
    epoch_unit: z.enum(["seconds", "milliseconds", "microseconds", "nanoseconds"]).optional().describe(
      "Unit of the epoch timestamp. Defaults to 'seconds'. Only used when source is a numeric value."
    ),
  },
  async ({ source, timezones, source_timezone, epoch_unit }) => {
    const targetTimezones = timezones?.length > 0 ? timezones : defaultTimezones;
    const convtz = source_timezone || configConvTZ;
    const usetz = configUseTZ;
    const src = source.trim();
    const isNumeric = /^-?[0-9]+(\.[0-9]+)?$/.test(src);

    let parsedSrc;
    let inputDescription;

    if (isNumeric) {
      const unit = epoch_unit || "seconds";
      const multiplier = EPOCH_UNITS[unit];
      parsedSrc = Number(src) * multiplier;
      inputDescription = `${src} (epoch ${unit})`;
    } else {
      parsedSrc = normalizeDT(src);
      inputDescription = src;

      // Validate the datetime string
      try {
        const d = new Date(parsedSrc);
        if (isNaN(d.getTime())) {
          return {
            content: [{ type: "text", text: `Error: Could not parse "${src}" as a valid datetime.` }],
            isError: true,
          };
        }
      } catch (error) {
        return {
          content: [{ type: "text", text: `Error: Could not parse "${src}" as a valid datetime. ${error}` }],
          isError: true,
        };
      }
    }

    let cdt;
    if (!isNumeric && convtz) {
      cdt = cdate().tz(convtz).cdateFn();
    } else {
      cdt = cdate().cdateFn();
    }

    const results = [];
    for (const tz of targetTimezones) {
      results.push(convertToTimezone(cdt, parsedSrc, tz, usetz));
    }

    const lines = [`Input: ${inputDescription}`, ""];
    for (const r of results) {
      if (r.error) {
        lines.push(`  ${r.timezone}: ERROR - ${r.error}`);
      } else {
        lines.push(`  ${r.timezone}: ${r.result}`);
      }
    }

    // Add epoch values when input is a datetime string
    if (!isNumeric) {
      const epochMs = cdt(parsedSrc).t;
      lines.push("");
      lines.push(`Epoch (seconds):      ${epochMs / 1000}`);
      lines.push(`Epoch (milliseconds): ${epochMs}`);
    }

    return {
      content: [{ type: "text", text: lines.join("\n") }],
    };
  }
);

server.tool(
  "current-time",
  "Get the current time in specified timezones.",
  {
    timezones: z.array(z.string()).optional().describe(
      "Timezones to show current time in (e.g. ['Asia/Tokyo', 'UTC']). If omitted, uses timezones from mclocks config or built-in defaults."
    ),
  },
  async ({ timezones }) => {
    const targetTimezones = timezones?.length > 0 ? timezones : defaultTimezones;
    const now = new Date();
    const cdt = cdate().cdateFn();

    const lines = [];
    for (const tz of targetTimezones) {
      try {
        const offset = cdt().tz(tz).utcOffset();
        const result = cdt(now).utcOffset(offset).text();
        lines.push(`  ${tz}: ${result}`);
      } catch (error) {
        lines.push(`  ${tz}: ERROR - ${error}`);
      }
    }

    const epochSec = Math.floor(now.getTime() / 1000);
    lines.push("");
    lines.push(`Epoch (seconds):      ${epochSec}`);
    lines.push(`Epoch (milliseconds): ${now.getTime()}`);

    return {
      content: [{ type: "text", text: lines.join("\n") }],
    };
  }
);

server.tool(
  "next-weekday",
  "Find the date of the next occurrence of a given weekday from today.",
  {
    weekday: z.string().describe(
      "Day of the week (e.g. 'Monday', 'friday', 'mon', 'thu'). Case-insensitive. English only."
    ),
    timezone: z.string().optional().describe(
      "Timezone to determine 'today' (e.g. 'Asia/Tokyo'). If omitted, uses convtz from mclocks config if available, otherwise UTC."
    ),
  },
  async ({ weekday, timezone }) => {
    const targetDow = parseWeekday(weekday);
    if (targetDow === null) {
      return {
        content: [{ type: "text", text: `Error: Could not parse "${weekday}" as a weekday name. Use English names like "Monday", "Tue", "friday".` }],
        isError: true,
      };
    }

    const tz = timezone || configConvTZ || "UTC";
    const cdt = cdate().locale(configLocale).cdateFn();
    const offset = cdt().tz(tz).utcOffset();
    const now = cdt().utcOffset(offset);
    const todayDow = Number(now.format("d"));

    let daysUntil = targetDow - todayDow;
    if (daysUntil <= 0) daysUntil += 7;

    const nextDate = now.add(daysUntil, "day");
    const dateStr = nextDate.format("YYYY-MM-DD");
    const dayName = nextDate.format("dddd");

    const lines = [
      `Next ${dayName}: ${dateStr}`,
      `(${daysUntil} day${daysUntil !== 1 ? "s" : ""} from today, ${now.format("YYYY-MM-DD")})`,
      `Timezone: ${tz}`,
    ];

    return {
      content: [{ type: "text", text: lines.join("\n") }],
    };
  }
);

server.tool(
  "date-to-weekday",
  "Get the day of the week for a given date.",
  {
    date: z.string().describe(
      "Date to check (e.g. '2026-02-20', '2026/3/15', 'March 15, 2026')."
    ),
  },
  async ({ date }) => {
    const src = date.trim();
    let d;
    try {
      d = new Date(src);
      if (isNaN(d.getTime())) {
        return {
          content: [{ type: "text", text: `Error: Could not parse "${src}" as a valid date.` }],
          isError: true,
        };
      }
    } catch (error) {
      return {
        content: [{ type: "text", text: `Error: Could not parse "${src}" as a valid date. ${error}` }],
        isError: true,
      };
    }

    // Use UTC-based date string to avoid timezone issues with date-only strings
    const y = d.getUTCFullYear();
    const m = String(d.getUTCMonth() + 1).padStart(2, "0");
    const dd = String(d.getUTCDate()).padStart(2, "0");
    const utcDateStr = `${y}-${m}-${dd}T00:00:00Z`;
    const dayName = cdate(utcDateStr).locale(configLocale).format("dddd");

    return {
      content: [{ type: "text", text: `${y}-${m}-${dd}: ${dayName}` }],
    };
  }
);

server.tool(
  "days-until",
  "Count the number of days from today until a specified date. " +
  "If year is omitted, uses the current year (or next year if the date has already passed). " +
  "If only day is specified, uses the current month (or next month if the day has already passed). " +
  "If month is omitted, defaults to January. If day is omitted, defaults to the 1st.",
  {
    year: z.number().int().optional().describe(
      "Target year (e.g. 2026). If omitted, uses current year or next year if the date has already passed this year."
    ),
    month: z.number().int().min(1).max(12).optional().describe(
      "Target month (1-12). Defaults to 1 (January) if omitted."
    ),
    day: z.number().int().min(1).max(31).optional().describe(
      "Target day of month (1-31). Defaults to 1 if omitted."
    ),
    timezone: z.string().optional().describe(
      "Timezone to determine 'today' (e.g. 'Asia/Tokyo'). If omitted, uses convtz from mclocks config if available, otherwise UTC."
    ),
  },
  async ({ year, month, day, timezone }) => {
    const tz = timezone || configConvTZ || "UTC";
    const cdt = cdate().locale(configLocale).cdateFn();
    const offset = cdt().tz(tz).utcOffset();
    const now = cdt().utcOffset(offset);
    const [nowY, nowM, nowD] = ["YYYY", "M", "D"].map((f) => Number(now.format(f)));

    let [y, m, d] = [year ?? nowY, month, day || 1];

    if (year != null) {
      m = m || 1;
    } else if (month == null && day != null) {
      // Only day specified: use current month, advance to next month if passed
      m = nowM;
      if (d < nowD) {
        m++;
        if (m > 12) { m = 1; y++; }
      }
    } else {
      // Month specified (or both omitted): default month to 1, advance to next year if passed
      m = m || 1;
      if (m < nowM || (m === nowM && d < nowD)) { y++; }
    }

    // Validate the target date
    const dateStr = `${y}-${String(m).padStart(2, "0")}-${String(d).padStart(2, "0")}`;
    const dateObj = new Date(`${dateStr}T00:00:00Z`);
    if (isNaN(dateObj.getTime()) || dateObj.getUTCDate() !== d) {
      return {
        content: [{ type: "text", text: `Error: Invalid date: ${dateStr}` }],
        isError: true,
      };
    }

    // Calculate days difference using UTC dates to avoid DST issues
    const todayStr = now.format("YYYY-MM-DD");
    const diffDays = Math.round((dateObj.getTime() - new Date(`${todayStr}T00:00:00Z`).getTime()) / 86400000);
    const dayName = cdate(`${dateStr}T00:00:00Z`).locale(configLocale).format("dddd");

    let label;
    if (diffDays > 0) {
      label = `${diffDays} day${diffDays !== 1 ? "s" : ""} from today`;
    } else if (diffDays === 0) {
      label = "Today";
    } else {
      label = `${-diffDays} day${diffDays !== -1 ? "s" : ""} ago`;
    }

    return {
      content: [{ type: "text", text: `${dateStr} (${dayName}): ${label}\nToday: ${todayStr}\nTimezone: ${tz}` }],
    };
  }
);

server.tool(
  "days-between",
  "Count the number of days between two dates. The start date is not included in the count (e.g. Jan 1 to Jan 3 = 2 days).",
  {
    from: z.string().describe(
      "Start date (e.g. '2026-01-01', '2026/3/15', 'March 15, 2026')."
    ),
    to: z.string().describe(
      "End date (e.g. '2026-12-31', '2026/6/1', 'June 1, 2026')."
    ),
  },
  async ({ from, to }) => {
    const fromDate = new Date(from.trim());
    const toDate = new Date(to.trim());

    if (isNaN(fromDate.getTime())) {
      return {
        content: [{ type: "text", text: `Error: Could not parse "${from}" as a valid date.` }],
        isError: true,
      };
    }
    if (isNaN(toDate.getTime())) {
      return {
        content: [{ type: "text", text: `Error: Could not parse "${to}" as a valid date.` }],
        isError: true,
      };
    }

    const diffDays = Math.round((toDate.getTime() - fromDate.getTime()) / 86400000);
    const abs = Math.abs(diffDays);
    const fromStr = `${fromDate.getUTCFullYear()}-${String(fromDate.getUTCMonth() + 1).padStart(2, "0")}-${String(fromDate.getUTCDate()).padStart(2, "0")}`;
    const toStr = `${toDate.getUTCFullYear()}-${String(toDate.getUTCMonth() + 1).padStart(2, "0")}-${String(toDate.getUTCDate()).padStart(2, "0")}`;
    const fromDay = cdate(`${fromStr}T00:00:00Z`).locale(configLocale).format("dddd");
    const toDay = cdate(`${toStr}T00:00:00Z`).locale(configLocale).format("dddd");

    return {
      content: [{ type: "text", text: `${fromStr} (${fromDay}) → ${toStr} (${toDay}): ${abs} day${abs !== 1 ? "s" : ""}` }],
    };
  }
);

async function main() {
  const transport = new StdioServerTransport();
  await server.connect(transport);
}

main().catch((error) => {
  console.error("Fatal error:", error);
  process.exit(1);
});
