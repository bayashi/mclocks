describe('Clipboard history panel (chist) smoke', () => {
	beforeEach(async () => {
		await browser.url('/');
		await browser.waitUntil(
			async () => {
				const readyState = await browser.execute(() => document.readyState);
				return readyState === 'complete';
			},
			{ timeout: 10000, timeoutMsg: 'Page did not load' }
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

	const CHIST_LOAD_CONFIG = {
		font: 'Courier, monospace',
		size: '14px',
		color: '#fff',
		clipboard: {
			closeOnCopy: false,
			maxClipNumber: 10,
			windowWidth: 420,
			windowHeight: 480,
		},
	};

	/**
	 * Mock Tauri invoke and boot chist UI into #mclocks (same pattern as sticky tests).
	 * @param {unknown[]} listRows - payload returned by chist_list (camelCase DTO fields).
	 */
	const setupChistPanel = async (listRows) => {
		await browser.execute(
			(configPayload, rows) => {
				document.documentElement.classList.add('chist');
				window.__TAURI_INTERNALS__ = window.__TAURI_INTERNALS__ || {};
				window.__TAURI_INTERNALS__.transformCallback = function () {
					return Date.now() + Math.random();
				};
				window.__TAURI_INTERNALS__.invoke = async function (cmd, _args) {
					if (cmd === 'load_config') {
						return configPayload;
					}
					if (cmd === 'chist_list') {
						return rows;
					}
					if (cmd === 'chist_apply') {
						return;
					}
					if (cmd === 'chist_close_panel') {
						return;
					}
					return null;
				};
			},
			CHIST_LOAD_CONFIG,
			listRows
		);

		const error = await browser.executeAsync((done) => {
			(async () => {
				try {
					const mod = await import('/src/chist.js');
					const mainElement = document.querySelector('#mclocks');
					await mod.chistPanelEntry(mainElement);
					done(null);
				} catch (e) {
					done(e.message);
				}
			})();
		});
		if (error) {
			throw new Error(error);
		}
		await browser.pause(400);
	};

	it('shows panel shell and empty state when history is empty', async () => {
		await setupChistPanel([]);

		const shellOk = await browser.execute(() => !!document.querySelector('.ch-shell'));
		expect(shellOk).toBe(true);

		const emptyText = await browser.execute(() => {
			const el = document.querySelector('.ch-empty-msg');
			return el ? el.textContent : '';
		});
		expect(emptyText).toContain('No clipboard text yet');
	});

	it('renders history cards when chist_list returns entries', async () => {
		const sampleText = 'e2e chist sample line';
		const utf8ByteLen = new TextEncoder().encode(sampleText).length;
		const unicodeScalarCount = [...sampleText].length;
		const lineCount = sampleText.split(/\r?\n/).length;

		const rows = [
			{
				text: sampleText,
				utf8ByteLen,
				unicodeScalarCount,
				lineCount,
				truncatedFromClipboard: false,
			},
		];

		await setupChistPanel(rows);

		const cardInfo = await browser.execute(() => {
			const card = document.querySelector('.ch-card');
			const textEl = document.querySelector('.ch-card-text');
			const meta = document.querySelector('.ch-footer-meta');
			return {
				cardCount: document.querySelectorAll('.ch-card').length,
				text: textEl ? textEl.textContent : null,
				meta: meta ? meta.textContent : null,
			};
		});

		expect(cardInfo.cardCount).toBe(1);
		expect(cardInfo.text).toBe(sampleText);
		expect(cardInfo.meta).toContain(String(unicodeScalarCount));
		expect(cardInfo.meta).toContain(String(lineCount));
	});
});
