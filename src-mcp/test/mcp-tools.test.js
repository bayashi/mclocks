import { Client } from "@modelcontextprotocol/sdk/client/index.js";
import { StdioClientTransport } from "@modelcontextprotocol/sdk/client/stdio.js";
import assert from "node:assert";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";

const __dirname = dirname(fileURLToPath(import.meta.url));
const serverPath = join(__dirname, "..", "server.js");

// Helper to get text from tool result
function text(result) {
  return result.content[0].text;
}

describe("mclocks MCP Tools", function () {
  let client;

  before(async function () {
    const transport = new StdioClientTransport({
      command: "node",
      args: [serverPath],
      env: {
        ...process.env,
        // Use a non-existent config path to ensure fallback defaults (locale=en, convtz="")
        MCLOCKS_CONFIG_PATH: "__nonexistent__",
      },
    });
    client = new Client({ name: "test-client", version: "1.0.0" });
    await client.connect(transport);
  });

  after(async function () {
    await client.close();
  });

  // ---- convert-time ----

  describe("convert-time", function () {
    it("should convert an ISO datetime to specified timezones", async function () {
      const result = await client.callTool({
        name: "convert-time",
        arguments: {
          source: "2024-01-15T10:30:00Z",
          timezones: ["UTC", "Asia/Tokyo"],
        },
      });
      const out = text(result);
      assert.ok(out.includes("UTC"), "should contain UTC");
      assert.ok(out.includes("Asia/Tokyo"), "should contain Asia/Tokyo");
      assert.ok(out.includes("Epoch"), "should contain epoch values");
    });

    it("should convert a BQ datetime format", async function () {
      const result = await client.callTool({
        name: "convert-time",
        arguments: {
          source: "2024-01-01 12:00:00 UTC",
          timezones: ["UTC"],
        },
      });
      assert.ok(!result.isError, "should not be an error");
      assert.ok(text(result).includes("UTC"));
    });

    it("should convert epoch seconds", async function () {
      const result = await client.callTool({
        name: "convert-time",
        arguments: {
          source: "1704067200",
          timezones: ["UTC"],
        },
      });
      const out = text(result);
      assert.ok(out.includes("epoch seconds"), "should indicate epoch seconds");
      assert.ok(out.includes("UTC"));
    });

    it("should convert epoch milliseconds", async function () {
      const result = await client.callTool({
        name: "convert-time",
        arguments: {
          source: "1704067200000",
          timezones: ["UTC"],
          epoch_unit: "milliseconds",
        },
      });
      assert.ok(!result.isError);
      assert.ok(text(result).includes("epoch milliseconds"));
    });

    it("should return error for invalid datetime", async function () {
      const result = await client.callTool({
        name: "convert-time",
        arguments: {
          source: "not-a-date",
          timezones: ["UTC"],
        },
      });
      assert.ok(result.isError, "should be an error");
      assert.ok(text(result).includes("Error"));
    });
  });

  // ---- current-time ----

  describe("current-time", function () {
    it("should return current time in specified timezones", async function () {
      const result = await client.callTool({
        name: "current-time",
        arguments: { timezones: ["UTC", "Asia/Tokyo"] },
      });
      const out = text(result);
      assert.ok(out.includes("UTC"));
      assert.ok(out.includes("Asia/Tokyo"));
      assert.ok(out.includes("Epoch"));
    });

    it("should work without arguments (use defaults)", async function () {
      const result = await client.callTool({
        name: "current-time",
        arguments: {},
      });
      assert.ok(!result.isError);
      assert.ok(text(result).includes("Epoch"));
    });
  });

  // ---- next-weekday ----

  describe("next-weekday", function () {
    it("should return a valid future date for Monday", async function () {
      const result = await client.callTool({
        name: "next-weekday",
        arguments: { weekday: "Monday", timezone: "UTC" },
      });
      const out = text(result);
      assert.match(out, /\d{4}-\d{2}-\d{2}/, "should contain a date");
      assert.ok(out.includes("Monday"), "should contain day name");
      assert.match(out, /\d+ days? from today/, "should show days until");
    });

    it("should accept abbreviated weekday names", async function () {
      const result = await client.callTool({
        name: "next-weekday",
        arguments: { weekday: "fri", timezone: "UTC" },
      });
      assert.ok(!result.isError);
      assert.match(text(result), /\d{4}-\d{2}-\d{2}/);
    });

    it("should accept two-letter abbreviations", async function () {
      const result = await client.callTool({
        name: "next-weekday",
        arguments: { weekday: "tu", timezone: "UTC" },
      });
      assert.ok(!result.isError);
      assert.match(text(result), /\d{4}-\d{2}-\d{2}/);
    });

    it("should be case-insensitive", async function () {
      const result = await client.callTool({
        name: "next-weekday",
        arguments: { weekday: "WEDNESDAY", timezone: "UTC" },
      });
      assert.ok(!result.isError);
      assert.match(text(result), /\d{4}-\d{2}-\d{2}/);
    });

    it("should return error for invalid weekday", async function () {
      const result = await client.callTool({
        name: "next-weekday",
        arguments: { weekday: "Notaday" },
      });
      assert.ok(result.isError, "should be an error");
      assert.ok(text(result).includes("Error"));
    });

    it("should always return a date 1-7 days in the future", async function () {
      const result = await client.callTool({
        name: "next-weekday",
        arguments: { weekday: "Sunday", timezone: "UTC" },
      });
      const match = text(result).match(/(\d+) days? from today/);
      assert.ok(match, "should contain days count");
      const days = parseInt(match[1], 10);
      assert.ok(days >= 1 && days <= 7, `days should be 1-7 but got ${days}`);
    });
  });

  // ---- date-to-weekday ----

  describe("date-to-weekday", function () {
    it("should return Friday for 2026-02-20", async function () {
      const result = await client.callTool({
        name: "date-to-weekday",
        arguments: { date: "2026-02-20" },
      });
      const out = text(result);
      assert.ok(out.includes("2026-02-20"), "should contain the date");
      assert.ok(out.includes("Friday"), "should be Friday");
    });

    it("should return Wednesday for 2025-01-01", async function () {
      const result = await client.callTool({
        name: "date-to-weekday",
        arguments: { date: "2025-01-01" },
      });
      assert.ok(text(result).includes("Wednesday"), "2025-01-01 should be Wednesday");
    });

    it("should handle various date formats", async function () {
      const result = await client.callTool({
        name: "date-to-weekday",
        arguments: { date: "March 15, 2026" },
      });
      assert.ok(!result.isError);
      assert.ok(text(result).includes("2026-03-15"), "should normalize to YYYY-MM-DD");
    });

    it("should return error for invalid date", async function () {
      const result = await client.callTool({
        name: "date-to-weekday",
        arguments: { date: "invalid" },
      });
      assert.ok(result.isError, "should be an error");
      assert.ok(text(result).includes("Error"));
    });
  });

  // ---- days-until ----

  describe("days-until", function () {
    it("should count days to a specific future date", async function () {
      const result = await client.callTool({
        name: "days-until",
        arguments: { year: 2030, month: 1, day: 1, timezone: "UTC" },
      });
      const out = text(result);
      assert.ok(out.includes("2030-01-01"), "should contain target date");
      assert.match(out, /from today/, "should indicate future");
    });

    it("should count days to a past date", async function () {
      const result = await client.callTool({
        name: "days-until",
        arguments: { year: 2020, month: 1, day: 1, timezone: "UTC" },
      });
      assert.match(text(result), /ago/, "should indicate past");
    });

    it("should auto-select year when omitted (month+day given)", async function () {
      const result = await client.callTool({
        name: "days-until",
        arguments: { month: 12, day: 25, timezone: "UTC" },
      });
      const out = text(result);
      assert.match(out, /12-25/, "should contain month-day");
      assert.ok(!result.isError);
    });

    it("should auto-select month when only day given", async function () {
      const result = await client.callTool({
        name: "days-until",
        arguments: { day: 28, timezone: "UTC" },
      });
      assert.ok(!result.isError);
      assert.match(text(result), /\d{4}-\d{2}-28/);
    });

    it("should default month to 1 and day to 1 for year-only", async function () {
      const result = await client.callTool({
        name: "days-until",
        arguments: { year: 2028, timezone: "UTC" },
      });
      assert.ok(text(result).includes("2028-01-01"));
    });

    it("should return error for invalid date like Feb 31", async function () {
      const result = await client.callTool({
        name: "days-until",
        arguments: { year: 2026, month: 2, day: 31, timezone: "UTC" },
      });
      assert.ok(result.isError, "Feb 31 should be invalid");
      assert.ok(text(result).includes("Error"));
    });

    it("should show Today for today's date", async function () {
      const now = new Date();
      const result = await client.callTool({
        name: "days-until",
        arguments: {
          year: now.getUTCFullYear(),
          month: now.getUTCMonth() + 1,
          day: now.getUTCDate(),
          timezone: "UTC",
        },
      });
      assert.ok(text(result).includes("Today"), "should say Today");
    });

    it("should include weekday name in output", async function () {
      const result = await client.callTool({
        name: "days-until",
        arguments: { year: 2026, month: 12, day: 25, timezone: "UTC" },
      });
      assert.ok(text(result).includes("Friday"), "2026-12-25 should be Friday");
    });
  });

  // ---- days-between ----

  describe("days-between", function () {
    it("should count 30 days from Jan 1 to Jan 31", async function () {
      const result = await client.callTool({
        name: "days-between",
        arguments: { from: "2026-01-01", to: "2026-01-31" },
      });
      assert.ok(text(result).includes("30 days"), "Jan 1 to Jan 31 should be 30 days");
    });

    it("should count 365 days for a non-leap year", async function () {
      const result = await client.callTool({
        name: "days-between",
        arguments: { from: "2026-01-01", to: "2027-01-01" },
      });
      assert.ok(text(result).includes("365 days"), "2026 is not a leap year");
    });

    it("should count 366 days for a leap year", async function () {
      const result = await client.callTool({
        name: "days-between",
        arguments: { from: "2024-01-01", to: "2025-01-01" },
      });
      assert.ok(text(result).includes("366 days"), "2024 is a leap year");
    });

    it("should handle reversed dates (to < from)", async function () {
      const result = await client.callTool({
        name: "days-between",
        arguments: { from: "2026-01-31", to: "2026-01-01" },
      });
      // Should still return positive day count (absolute value)
      assert.ok(text(result).includes("30 days"), "should return absolute days");
    });

    it("should return 0 days for same date", async function () {
      const result = await client.callTool({
        name: "days-between",
        arguments: { from: "2026-06-15", to: "2026-06-15" },
      });
      assert.ok(text(result).includes("0 day"), "same date should be 0 days");
    });

    it("should include weekday names in output", async function () {
      const result = await client.callTool({
        name: "days-between",
        arguments: { from: "2026-01-01", to: "2026-01-31" },
      });
      const out = text(result);
      assert.ok(out.includes("→"), "should contain arrow separator");
      assert.ok(out.includes("Thursday"), "2026-01-01 should be Thursday");
      assert.ok(out.includes("Saturday"), "2026-01-31 should be Saturday");
    });

    it("should return error for invalid from date", async function () {
      const result = await client.callTool({
        name: "days-between",
        arguments: { from: "bad-date", to: "2026-01-01" },
      });
      assert.ok(result.isError);
      assert.ok(text(result).includes("Error"));
    });

    it("should return error for invalid to date", async function () {
      const result = await client.callTool({
        name: "days-between",
        arguments: { from: "2026-01-01", to: "bad-date" },
      });
      assert.ok(result.isError);
      assert.ok(text(result).includes("Error"));
    });
  });
});
