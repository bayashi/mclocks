// DST helpers for MCP tool text (Intl + Jan/Jul offset heuristic).

// Minutes east of UTC at `date` in `tz` (e.g. Tokyo +540), or null.
function offsetEastMin(tz, date) {
  try {
    const parts = new Intl.DateTimeFormat("en-US", {
      timeZone: tz,
      year: "numeric",
      month: "2-digit",
      day: "2-digit",
      hour: "2-digit",
      minute: "2-digit",
      second: "2-digit",
      hour12: false,
    }).formatToParts(date);
    const get = (t) => {
      const p = parts.find((x) => x.type === t);
      return p ? Number(p.value) : NaN;
    };
    const y = get("year");
    const mo = get("month");
    const da = get("day");
    const h = get("hour");
    const mi = get("minute");
    const se = get("second");
    if ([y, mo, da, h, mi, se].some((n) => Number.isNaN(n))) {
      return null;
    }
    const utcMs = Date.UTC(y, mo - 1, da, h, mi, se);
    return (utcMs - date.getTime()) / 60000;
  } catch {
    return null;
  }
}

// Calendar year in `tz` for wall time at `date`.
function yearInTz(tz, date) {
  try {
    return Number(new Intl.DateTimeFormat("en", { timeZone: tz, year: "numeric" }).format(date));
  } catch {
    return NaN;
  }
}

// Mid-Jan vs mid-Jul (UTC anchors) east-offset minutes, or null. Single Date pair per call.
function janJulEastMinutes(tz, year) {
  const jan = offsetEastMin(tz, new Date(Date.UTC(year, 0, 15, 12, 0, 0)));
  const jul = offsetEastMin(tz, new Date(Date.UTC(year, 6, 15, 12, 0, 0)));
  if (jan === null || jul === null) {
    return null;
  }
  return { jan, jul };
}

// { dst: boolean } only when zone uses DST and offset matches Jan/Jul extremes; else null.
// null also when historicMode (MCLOCKS_USETZ / usetz), invalid instant, or indeterminate.
function dstField(tz, instant, historicMode) {
  if (historicMode) {
    return null;
  }
  if (!(instant instanceof Date) || isNaN(instant.getTime())) {
    return null;
  }
  try {
    new Intl.DateTimeFormat("en", { timeZone: tz }).format(instant);
  } catch {
    return null;
  }
  const year = yearInTz(tz, instant);
  if (Number.isNaN(year)) {
    return null;
  }
  const anchors = janJulEastMinutes(tz, year);
  if (!anchors || anchors.jan === anchors.jul) {
    return null;
  }
  const oJan = anchors.jan;
  const oJul = anchors.jul;
  const o = offsetEastMin(tz, instant);
  if (o === null) {
    return null;
  }
  const hi = Math.max(oJan, oJul);
  const lo = Math.min(oJan, oJul);
  if (o !== hi && o !== lo) {
    return null;
  }
  return { dst: o === hi };
}

// Sparse map: only zones with a dst row. Values are { dst: boolean }. Null if empty.
// historicMode: skip all DST meta when true (strict historic offsets).
export function dstMetaMap(timeZones, instant, historicMode) {
  const meta = {};
  for (const tz of timeZones) {
    const row = dstField(tz, instant, historicMode);
    if (row) {
      meta[tz] = row;
    }
  }
  return Object.keys(meta).length > 0 ? meta : null;
}

// Append JSON footer after body when meta is non-null.
export function appendDstMeta(text, meta) {
  if (!meta) {
    return text;
  }
  return `${text}\n\n${JSON.stringify(meta)}`;
}
