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

// Load mclocks config.json
// Priority: MCLOCKS_CONFIG_PATH env > auto-detect > null
function loadMclocksConfig() {
  const configPath = process.env.MCLOCKS_CONFIG_PATH || autoDetectConfigPath();
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
const configUseTZ = mclocksConfig?.usetz ?? false;
const configConvTZ = mclocksConfig?.convtz || "";

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

async function main() {
  const transport = new StdioServerTransport();
  await server.connect(transport);
}

main().catch((error) => {
  console.error("Fatal error:", error);
  process.exit(1);
});
