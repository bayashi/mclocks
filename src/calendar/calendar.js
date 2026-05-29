import { invoke } from '@tauri-apps/api/core';
import { getCurrentWindow, LogicalSize } from '@tauri-apps/api/window';
import { cdate } from 'cdate';

import { escapeHTML, isMacOS, openMessageDialog, writeClipboardText } from '../util.js';

const EN_WEEKDAY_LABELS = ['Su', 'Mo', 'Tu', 'We', 'Th', 'Fr', 'Sa'];
const MAX_WEEKDAY_LABEL_CHARS = 3;
const SUNDAY_UTC_MS = Date.UTC(2024, 0, 7);

const CLOSE_FADE_MS = 220;
const DAY_ROLLOVER_MS = 60_000;
/** Calendar root font-size = config `size` × this factor (see `scaledFontSizeFromConfig`). */
const CALENDAR_FONT_SCALE = 1.25;
/** Fixed week rows so panel height does not change when navigating months. */
const CALENDAR_WEEK_ROWS = 6;
/** Probe each center-month offset in a year when locking window size. */
const CALENDAR_SIZE_PROBE_MONTHS = 12;

let calendarPanelClosing = false;
let lockedCalendarLogicalSize = null;
let lockedCalendarSizeKey = '';

function sizeToCssPx(size) {
	if (typeof size === 'number') {
		return `${size}px`;
	}
	if (typeof size === 'string') {
		if (/^[\d.]+$/.test(size)) {
			return `${size}px`;
		}
		return size;
	}
	return '14px';
}

function scaledFontSizeFromConfig(size, scale = CALENDAR_FONT_SCALE) {
	const base = sizeToCssPx(size);
	const pxMatch = /^([\d.]+)px$/.exec(base);
	if (pxMatch) {
		return `${parseFloat(pxMatch[1]) * scale}px`;
	}
	return `calc(${base} * ${scale})`;
}

function pad2(n) {
	return n < 10 ? `0${n}` : String(n);
}

function createDateFn(locale, timezone) {
	return cdate().locale(locale).tz(timezone).cdateFn();
}

function labelCharCount(label) {
	return [...label].length;
}

function getWeekdayLabels(locale) {
	for (const style of ['short', 'narrow']) {
		let fmt;
		try {
			fmt = new Intl.DateTimeFormat(locale, { weekday: style, timeZone: 'UTC' });
		} catch {
			break;
		}
		const labels = [];
		for (let i = 0; i < 7; i += 1) {
			labels.push(fmt.format(new Date(SUNDAY_UTC_MS + i * 86_400_000)));
		}
		const maxLen = Math.max(...labels.map(labelCharCount));
		if (maxLen <= MAX_WEEKDAY_LABEL_CHARS || style === 'narrow') {
			return labels;
		}
	}
	return EN_WEEKDAY_LABELS;
}

function encodeCopyPayload(text) {
	const bytes = new TextEncoder().encode(text);
	let bin = '';
	for (let i = 0; i < bytes.length; i += 1) {
		bin += String.fromCharCode(bytes[i]);
	}
	return btoa(bin);
}

function decodeCopyPayload(b64) {
	const bin = atob(b64);
	const bytes = new Uint8Array(bin.length);
	for (let i = 0; i < bin.length; i += 1) {
		bytes[i] = bin.charCodeAt(i);
	}
	return new TextDecoder().decode(bytes);
}

function formatDayCell(cell) {
	if (cell.pad || cell.outside) {
		return '   ';
	}
	return String(cell.day).padStart(3, ' ');
}

function weekHasInMonthDay(week) {
	return week.some((cell) => !cell.pad && !cell.outside);
}

function formatMonthCalendarText(title, weekdayLabels, weeks) {
	const lines = [title];
	lines.push(weekdayLabels.join('  '));
	for (const week of weeks) {
		if (!weekHasInMonthDay(week)) {
			continue;
		}
		lines.push(week.map((cell) => formatDayCell(cell)).join(''));
	}
	return lines.join('\n');
}

function formatMonthTitle(locale, timezone, year, month) {
	const probe = new Date(Date.UTC(year, month - 1, 15, 12, 0, 0));
	try {
		return new Intl.DateTimeFormat(locale, {
			year: 'numeric',
			month: 'long',
			timeZone: timezone,
		}).format(probe);
	} catch {
		return `${year}-${pad2(month)}`;
	}
}

function monthAnchor(fn, year, month) {
	return fn(`${year}-${pad2(month)}-01`);
}

function daysInMonth(fn, year, month) {
	return Number(monthAnchor(fn, year, month).add(1, 'M').add(-1, 'd').format('D'));
}

function wallParts(fn, inst) {
	const d = inst ?? fn();
	return {
		year: Number(d.format('YYYY')),
		month: Number(d.format('M')),
		day: Number(d.format('D')),
	};
}

function isSameYmd(a, b) {
	return a.year === b.year && a.month === b.month && a.day === b.day;
}

/** @typedef {{ timezone: string, today: { year: number, month: number, day: number } }} ClockTodayMarker */

/**
 * @param {string} locale
 * @param {Array<{ timezone?: string }>|undefined} clocks
 * @returns {ClockTodayMarker[]}
 */
function buildClockTodayMarkers(locale, clocks) {
	const markers = [];
	for (const clock of clocks ?? []) {
		const timezone = clock?.timezone?.trim();
		if (!timezone) {
			continue;
		}
		const fn = createDateFn(locale, timezone);
		markers.push({ timezone, today: wallParts(fn) });
	}
	if (markers.length === 0) {
		const fn = createDateFn(locale, 'UTC');
		markers.push({ timezone: 'UTC', today: wallParts(fn) });
	}
	return markers;
}

/**
 * @param {{ year: number, month: number, day: number }} ymd
 * @param {ClockTodayMarker[]} clockTodays
 * @returns {string[]}
 */
function timezoneLabelsForYmd(ymd, clockTodays) {
	const labels = [];
	const seen = new Set();
	for (const marker of clockTodays) {
		if (!isSameYmd(ymd, marker.today) || seen.has(marker.timezone)) {
			continue;
		}
		seen.add(marker.timezone);
		labels.push(marker.timezone);
	}
	return labels;
}

function clockTodaySignature(clockTodays) {
	return clockTodays
		.map(
			(m) =>
				`${m.timezone}:${m.today.year}-${m.today.month}-${m.today.day}`,
		)
		.join('|');
}

function buildMonthWeeks(fn, year, month, clockTodays) {
	const first = monthAnchor(fn, year, month);
	const startDow = Number(first.format('d'));
	const inMonthDays = daysInMonth(fn, year, month);
	const prev = first.add(-1, 'M');
	const prevYear = Number(prev.format('YYYY'));
	const prevMonth = Number(prev.format('M'));
	const prevMonthDays = daysInMonth(fn, prevYear, prevMonth);

	const cells = [];

	for (let i = startDow - 1; i >= 0; i -= 1) {
		const day = prevMonthDays - i;
		cells.push({
			day,
			outside: true,
			ymd: { year: prevYear, month: prevMonth, day },
		});
	}

	for (let day = 1; day <= inMonthDays; day += 1) {
		cells.push({
			day,
			outside: false,
			ymd: { year, month, day },
		});
	}

	let nextDay = 1;
	const next = first.add(1, 'M');
	const nextYear = Number(next.format('YYYY'));
	const nextMonth = Number(next.format('M'));
	while (cells.length % 7 !== 0) {
		cells.push({
			day: nextDay,
			outside: true,
			ymd: { year: nextYear, month: nextMonth, day: nextDay },
		});
		nextDay += 1;
	}

	const weeks = [];
	for (let i = 0; i < cells.length; i += 7) {
		weeks.push(cells.slice(i, i + 7));
	}
	return weeks.map((week) =>
		week.map((cell) => {
			const tzLabels = timezoneLabelsForYmd(cell.ymd, clockTodays);
			return {
				...cell,
				clockTzLabels: tzLabels,
				isClockToday: tzLabels.length > 0,
			};
		}),
	);
}

function padWeeksTo(weeks, targetCount) {
	const padded = weeks.map((week) => week.slice());
	while (padded.length < targetCount) {
		padded.push(
			Array.from({ length: 7 }, () => ({
				day: '',
				outside: true,
				pad: true,
				isClockToday: false,
				clockTzLabels: [],
			})),
		);
	}
	return padded;
}

function monthHtml(fn, year, month, locale, timezone, weeks, weekdayLabels) {
	const title = formatMonthTitle(locale, timezone, year, month);
	const weekdayRow = weekdayLabels
		.map((label) => `<div class="cal-weekday">${label}</div>`)
		.join('');
	const weekRows = weeks
		.map((week) => {
			const days = week
				.map((cell) => {
					if (cell.pad) {
						return '<div class="cal-day cal-day-pad" aria-hidden="true"></div>';
					}
					const cls = [
						'cal-day',
						cell.outside ? 'cal-day-outside' : '',
						cell.isClockToday ? 'cal-day-today' : '',
					]
						.filter(Boolean)
						.join(' ');
					const tzTip =
						cell.clockTzLabels.length > 0
							? ` title="${escapeHTML(cell.clockTzLabels.join(', '))}"`
							: '';
					return `<div class="${cls}"${tzTip}>${cell.day}</div>`;
				})
				.join('');
			return `<div class="cal-week">${days}</div>`;
		})
		.join('');
	const copyText = formatMonthCalendarText(title, weekdayLabels, weeks);
	const copyB64 = encodeCopyPayload(copyText);
	return `
<section class="cal-month cal-month-copy" lang="${locale}" data-copy-b64="${copyB64}" role="button" tabindex="0" aria-label="Copy calendar text">
	<div class="cal-month-title">${title}</div>
	<div class="cal-grid">
		<div class="cal-weekdays">${weekdayRow}</div>
		<div class="cal-weeks">${weekRows}</div>
	</div>
</section>`;
}

function renderCalendarBody(
	monthsHost,
	fn,
	locale,
	timezone,
	clockTodays,
	centerMonthOffset,
) {
	const center =
		centerMonthOffset === 0 ? fn() : fn().add(centerMonthOffset, 'M');
	const prev = center.add(-1, 'M');
	const next = center.add(1, 'M');
	const triple = [
		wallParts(fn, prev),
		wallParts(fn, center),
		wallParts(fn, next),
	];
	const weekdayLabels = getWeekdayLabels(locale);
	const monthWeeks = triple.map(({ year, month }) => ({
		year,
		month,
		weeks: buildMonthWeeks(fn, year, month, clockTodays),
	}));
	const maxWeeks = CALENDAR_WEEK_ROWS;
	monthsHost.innerHTML = monthWeeks
		.map(({ year, month, weeks }) =>
			monthHtml(
				fn,
				year,
				month,
				locale,
				timezone,
				padWeeksTo(weeks, maxWeeks),
				weekdayLabels,
			),
		)
		.join('');
}

async function copyMonthElement(monthEl) {
	const b64 = monthEl.dataset.copyB64;
	if (!b64) {
		return;
	}
	let text;
	try {
		text = decodeCopyPayload(b64);
	} catch {
		return;
	}
	if (!text) {
		return;
	}
	try {
		await writeClipboardText(text);
		monthEl.classList.remove('is-copy-flash');
		void monthEl.offsetWidth;
		monthEl.classList.add('is-copy-flash');
		window.setTimeout(() => {
			monthEl.classList.remove('is-copy-flash');
		}, 220);
	} catch (error) {
		await openMessageDialog(
			`Failed to copy: ${error}`,
			'mclocks Error',
			'error',
		);
	}
}

function calendarLayoutSizeKey(locale, cfg) {
	const size = cfg?.size ?? 14;
	const font = cfg?.font ?? '';
	return `${locale}|${size}|${font}`;
}

async function waitForCalendarLayout() {
	await new Promise((resolve) => {
		requestAnimationFrame(() => requestAnimationFrame(resolve));
	});
}

function measureCalendarWindowSize(mainElement) {
	const rootStyle = getComputedStyle(mainElement);
	const padX =
		(parseFloat(rootStyle.paddingLeft) || 0) +
		(parseFloat(rootStyle.paddingRight) || 0);
	const padY =
		(parseFloat(rootStyle.paddingTop) || 0) +
		(parseFloat(rootStyle.paddingBottom) || 0);
	const shell = mainElement.querySelector('.cal-shell');
	if (!shell) {
		return {
			width: Math.ceil(mainElement.scrollWidth),
			height: Math.ceil(mainElement.scrollHeight),
		};
	}
	const shellRect = shell.getBoundingClientRect();
	return {
		width: Math.ceil(shellRect.width) + padX,
		height: Math.ceil(shellRect.height) + padY,
	};
}

async function computeMaxCalendarWindowSize(
	mainElement,
	monthsHost,
	fn,
	locale,
	timezone,
	clockTodays,
	centerMonthOffset,
) {
	let maxWidth = 0;
	let maxHeight = 0;
	for (let offset = 0; offset < CALENDAR_SIZE_PROBE_MONTHS; offset += 1) {
		renderCalendarBody(
			monthsHost,
			fn,
			locale,
			timezone,
			clockTodays,
			offset,
		);
		await waitForCalendarLayout();
		const measured = measureCalendarWindowSize(mainElement);
		maxWidth = Math.max(maxWidth, measured.width);
		maxHeight = Math.max(maxHeight, measured.height);
	}
	renderCalendarBody(
		monthsHost,
		fn,
		locale,
		timezone,
		clockTodays,
		centerMonthOffset,
	);
	await waitForCalendarLayout();
	return {
		width: maxWidth,
		height: maxHeight,
	};
}

async function applyCalendarWindowSize(mainElement, currentWindow, size) {
	if (!currentWindow || !mainElement || !size) {
		return;
	}
	const width = Math.ceil(size.width);
	const height = Math.ceil(size.height);
	if (width < 1 || height < 1) {
		return;
	}
	try {
		await currentWindow.setSize(new LogicalSize(width, height));
	} catch {
		// ignore
	}
}

async function showCalendarPanelWindow(currentWindow) {
	if (!currentWindow) {
		return;
	}
	try {
		await currentWindow.show();
	} catch {
		// ignore
	}
	try {
		await currentWindow.setFocus();
	} catch {
		// ignore
	}
}

async function closePanel() {
	if (calendarPanelClosing) {
		return;
	}
	calendarPanelClosing = true;
	document.documentElement.classList.add('calendar-is-closing');
	window.setTimeout(async () => {
		try {
			await invoke('calendar_close_panel');
		} finally {
			document.documentElement.classList.remove('calendar-is-closing');
			calendarPanelClosing = false;
		}
	}, CLOSE_FADE_MS);
}

export async function calendarPanelEntry(mainElement) {
	let cfg = null;
	try {
		cfg = await invoke('load_config', {});
	} catch {
		// cfg remains null
	}

	document.documentElement.classList.remove('calendar-is-closing');

	const locale = cfg?.locale ?? 'en';
	const centerTz = cfg?.clocks?.[0]?.timezone?.trim() || 'UTC';

	if (cfg) {
		document.documentElement.style.fontFamily = cfg.font;
		document.documentElement.style.fontSize = scaledFontSizeFromConfig(cfg.size);
		document.documentElement.style.color = cfg.color;
	} else {
		document.documentElement.style.fontSize = scaledFontSizeFromConfig(14);
	}

	const fn = createDateFn(locale, centerTz);

	mainElement.innerHTML = '';
	mainElement.classList.add('cal-root');
	mainElement.innerHTML = `
<div class="cal-shell">
	<header class="cal-header-bar">
		<div class="cal-header-spacer"></div>
		<button type="button" class="cal-close-x" id="cal-close" aria-label="Close">✖</button>
	</header>
	<div class="cal-body">
		<div class="cal-nav-row">
			<button type="button" class="cal-nav-btn cal-nav-prev" id="cal-nav-prev" aria-label="Previous month">‹</button>
			<div class="cal-months" id="cal-months"></div>
			<button type="button" class="cal-nav-btn cal-nav-next" id="cal-nav-next" aria-label="Next month">›</button>
		</div>
	</div>
</div>
`;

	const monthsHost = mainElement.querySelector('#cal-months');
	const navPrev = mainElement.querySelector('#cal-nav-prev');
	const navNext = mainElement.querySelector('#cal-nav-next');
	const closeBtn = mainElement.querySelector('#cal-close');
	const headerBar = mainElement.querySelector('.cal-header-bar');

	let currentWindow = null;
	try {
		currentWindow = getCurrentWindow();
	} catch {
		currentWindow = null;
	}

	let centerMonthOffset = 0;
	let lastRenderSignature = '';
	let panelPreparePromise = null;
	let panelBootstrapComplete = false;

	const refreshCalendar = async (force = false) => {
		const clockTodays = buildClockTodayMarkers(locale, cfg?.clocks);
		const signature = `${clockTodaySignature(clockTodays)}|m${centerMonthOffset}`;
		if (force || signature !== lastRenderSignature || !monthsHost?.innerHTML) {
			lastRenderSignature = signature;
			if (monthsHost) {
				const sizeKey = calendarLayoutSizeKey(locale, cfg);
				if (
					!lockedCalendarLogicalSize ||
					lockedCalendarSizeKey !== sizeKey
				) {
					lockedCalendarLogicalSize =
						await computeMaxCalendarWindowSize(
							mainElement,
							monthsHost,
							fn,
							locale,
							centerTz,
							clockTodays,
							centerMonthOffset,
						);
					lockedCalendarSizeKey = sizeKey;
				} else {
					renderCalendarBody(
						monthsHost,
						fn,
						locale,
						centerTz,
						clockTodays,
						centerMonthOffset,
					);
					await waitForCalendarLayout();
				}
				await applyCalendarWindowSize(
					mainElement,
					currentWindow,
					lockedCalendarLogicalSize,
				);
			}
		}
	};

	navPrev?.addEventListener('click', (event) => {
		event.stopPropagation();
		centerMonthOffset -= 1;
		void refreshCalendar(true);
	});

	navNext?.addEventListener('click', (event) => {
		event.stopPropagation();
		centerMonthOffset += 1;
		void refreshCalendar(true);
	});

	monthsHost?.addEventListener('click', (event) => {
		const monthEl = event.target.closest('.cal-month-copy');
		if (!monthEl || !monthsHost.contains(monthEl)) {
			return;
		}
		void copyMonthElement(monthEl);
	});

	monthsHost?.addEventListener('keydown', (event) => {
		if (event.key !== 'Enter' && event.key !== ' ') {
			return;
		}
		const monthEl = event.target.closest('.cal-month-copy');
		if (!monthEl || !monthsHost.contains(monthEl)) {
			return;
		}
		event.preventDefault();
		void copyMonthElement(monthEl);
	});

	closeBtn?.addEventListener('click', () => {
		void closePanel();
	});

	if (isMacOS() && currentWindow) {
		headerBar?.addEventListener('mousedown', async (event) => {
			if (event.target.closest('button')) {
				return;
			}
			try {
				await currentWindow.startDragging();
			} catch {
				// ignore
			}
		});
	}

	window.addEventListener('keydown', (e) => {
		if (e.key === 'Escape') {
			e.preventDefault();
			void closePanel();
		}
	});

	const prepareAndShowPanel = async () => {
		if (panelPreparePromise) {
			return panelPreparePromise;
		}
		panelPreparePromise = (async () => {
			document.documentElement.classList.remove('calendar-is-closing');
			document.documentElement.classList.add('calendar-is-preparing');
			try {
				await refreshCalendar(true);
			} finally {
				document.documentElement.classList.remove('calendar-is-preparing');
			}
			await showCalendarPanelWindow(currentWindow);
			panelBootstrapComplete = true;
		})().finally(() => {
			panelPreparePromise = null;
		});
		return panelPreparePromise;
	};

	const onPanelShown = () => {
		void prepareAndShowPanel();
	};

	window.addEventListener('focus', () => {
		if (!panelBootstrapComplete) {
			return;
		}
		document.documentElement.classList.remove('calendar-is-closing');
		void refreshCalendar(true);
	});
	window.addEventListener('mclocks-calendar-show', onPanelShown);

	await prepareAndShowPanel();
	window.setInterval(() => {
		void refreshCalendar(false);
	}, DAY_ROLLOVER_MS);
}
