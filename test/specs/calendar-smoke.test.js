describe('Calendar panel smoke', () => {
	beforeEach(async () => {
		await browser.url('/');
		await browser.waitUntil(
			async () => {
				const readyState = await browser.execute(() => document.readyState);
				return readyState === 'complete';
			},
			{ timeout: 10000, timeoutMsg: 'Page did not load' },
		);
	});

	afterEach(async function () {
		if (this.currentTest && this.currentTest.state === 'failed') {
			console.log('\n=== Test failed - Browser will stay open for 30 seconds for debugging ===');
			console.log('Test:', this.currentTest.title);
			if (this.currentTest.err) {
				console.log('Error:', this.currentTest.err.message);
			}
			await browser.pause(30000);
		}
	});

	const CALENDAR_LOAD_CONFIG = {
		font: 'Courier, monospace',
		size: '14px',
		color: '#fff',
		locale: 'en',
		clocks: [{ name: 'UTC', timezone: 'UTC' }],
	};

	const setupCalendarPanel = async () => {
		await browser.execute((configPayload) => {
			document.documentElement.classList.add('calendar');
			window.__TAURI_INTERNALS__ = window.__TAURI_INTERNALS__ || {};
			window.__TAURI_INTERNALS__.transformCallback = function () {
				return Date.now() + Math.random();
			};
			window.__TAURI_INTERNALS__.invoke = async function (cmd) {
				if (cmd === 'load_config') {
					return configPayload;
				}
				if (cmd === 'calendar_close_panel') {
					return;
				}
				return null;
			};
		}, CALENDAR_LOAD_CONFIG);

		const error = await browser.executeAsync((done) => {
			(async () => {
				try {
					const mod = await import('/src/calendar/calendar.js');
					const mainElement = document.querySelector('#mclocks');
					await mod.calendarPanelEntry(mainElement);
					done(null);
				} catch (e) {
					done(e.message);
				}
			})();
		});
		if (error) {
			throw new Error(error);
		}

		await browser.waitUntil(
			async () => {
				const count = await browser.execute(
					() => document.querySelectorAll('.cal-month').length,
				);
				return count === 3;
			},
			{
				timeout: 20000,
				timeoutMsg: 'Expected three month columns',
			},
		);
	};

	it('shows shell, three months, and six week rows per month', async () => {
		await setupCalendarPanel();

		const layout = await browser.execute(() => {
			const shell = !!document.querySelector('.cal-shell');
			const months = [...document.querySelectorAll('.cal-month')];
			const weekCounts = months.map(
				(m) => m.querySelectorAll('.cal-week').length,
			);
			const weekdayCount = months[0]
				? months[0].querySelectorAll('.cal-weekday').length
				: 0;
			return { shell, monthCount: months.length, weekCounts, weekdayCount };
		});

		expect(layout.shell).toBe(true);
		expect(layout.monthCount).toBe(3);
		expect(layout.weekCounts).toEqual([6, 6, 6]);
		expect(layout.weekdayCount).toBe(7);
	});

	it('embeds multi-line copy text on each month (not YYYY-MM only)', async () => {
		await setupCalendarPanel();

		const copyLines = await browser.execute(() => {
			const el = document.querySelector('.cal-month-copy');
			if (!el?.dataset.copyB64) {
				return null;
			}
			const bin = atob(el.dataset.copyB64);
			const bytes = new Uint8Array(bin.length);
			for (let i = 0; i < bin.length; i += 1) {
				bytes[i] = bin.charCodeAt(i);
			}
			const text = new TextDecoder().decode(bytes);
			return text.split('\n');
		});

		expect(copyLines).not.toBeNull();
		expect(copyLines.length).toBeGreaterThanOrEqual(3);
		expect(copyLines[0].length).toBeGreaterThan(7);
		expect(copyLines[1].trim().length).toBeGreaterThan(0);
	});

	it('still renders three months after next/prev navigation', async () => {
		await setupCalendarPanel();

		await browser.execute(() => {
			document.querySelector('#cal-nav-next')?.click();
		});
		await browser.waitUntil(
			async () => {
				const count = await browser.execute(
					() => document.querySelectorAll('.cal-month').length,
				);
				return count === 3;
			},
			{ timeout: 10000, timeoutMsg: 'Months missing after next' },
		);

		await browser.execute(() => {
			document.querySelector('#cal-nav-prev')?.click();
		});
		await browser.waitUntil(
			async () => {
				const count = await browser.execute(
					() => document.querySelectorAll('.cal-month').length,
				);
				return count === 3;
			},
			{ timeout: 10000, timeoutMsg: 'Months missing after prev' },
		);
	});
});
