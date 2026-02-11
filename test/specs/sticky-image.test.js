// Minimal 1x1 red pixel PNG as base64
const TINY_PNG_BASE64 = 'iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8z8BQDwADhQGAWjR9awAAAABJRU5ErkJggg==';

describe('Sticky Note - Image sticky behavior', () => {
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

	afterEach(async function() {
		if (this.currentTest && this.currentTest.state === 'failed') {
			console.log('\n=== Test failed - Browser will stay open for 30 seconds for debugging ===');
			console.log('Test:', this.currentTest.title);
			if (this.currentTest.err) {
				console.log('Error:', this.currentTest.err.message);
			}
			await browser.pause(30000);
		}
	});

	/**
	 * Set up Tauri mocks and initialize an image sticky window.
	 *
	 * @param {object} opts
	 * @param {string|null} opts.imageBase64 - Base64 PNG to return from load_sticky_image (null to simulate missing file)
	 * @param {boolean|null} opts.persistedLocked - Persisted locked state
	 * @param {boolean} opts.persistedOpen - Persisted open state
	 */
	const setupImageStickyWindow = async (opts = {}) => {
		const { imageBase64 = TINY_PNG_BASE64, persistedLocked = null, persistedOpen = false } = opts;
		await browser.execute((imgB64, persLocked, persOpen, tinyPng) => {
			window.__stickyImageTest = {
				saveStateCalls: [],
				writeImageCalled: false,
				deleteStickyCalled: false,
			};

			window.__TAURI_INTERNALS__ = window.__TAURI_INTERNALS__ || {};
			window.__TAURI_INTERNALS__.metadata = {
				currentWindow: { label: 'sticky-test-image' },
			};
			window.__TAURI_INTERNALS__.transformCallback = function() {
				return Date.now() + Math.random();
			};
			window.__TAURI_INTERNALS__.invoke = async function(cmd, args) {
				// App IPC commands
				if (cmd === 'load_config') {
					return {
						forefront: false,
						font: 'Courier, monospace',
						size: '14px',
						color: '#fff',
					};
				}
				if (cmd === 'sticky_take_init_content') {
					return { text: '', contentType: 'image' };
				}
				if (cmd === 'load_sticky_state') {
					return {
						isOpen: persOpen,
						openWidth: null,
						openHeight: null,
						forefront: null,
						locked: persLocked,
						contentType: 'image',
					};
				}
				if (cmd === 'load_sticky_image') {
					if (imgB64 === null) {
						throw new Error('file not found');
					}
					return imgB64;
				}
				if (cmd === 'save_sticky_state') {
					window.__stickyImageTest.saveStateCalls.push(JSON.parse(JSON.stringify(args)));
					return;
				}
				if (cmd === 'save_sticky_text') return;
				if (cmd === 'save_window_state_exclusive') return;
				if (cmd === 'delete_sticky_text') {
					window.__stickyImageTest.deleteStickyCalled = true;
					return;
				}
				// Clipboard write image
				if (cmd && typeof cmd === 'string' && cmd.includes('write_image')) {
					window.__stickyImageTest.writeImageCalled = true;
					return;
				}
				// Clipboard read text (return empty so image fallback can trigger)
				if (cmd && typeof cmd === 'string' && (cmd.includes('clipboard') || cmd.includes('read_text'))) {
					return '';
				}
				if (cmd === 'create_sticky') return;
				if (cmd === 'create_sticky_image') return;
				// Dialog
				if (cmd && typeof cmd === 'string' && cmd.includes('dialog')) return true;
				// Tauri window plugin commands
				if (cmd === 'plugin:window|set_always_on_top') return;
				if (cmd === 'plugin:window|set_size') return;
				if (cmd === 'plugin:window|inner_size') return { width: 360, height: 100 };
				if (cmd === 'plugin:window|scale_factor') return 1;
				if (cmd === 'plugin:window|close') return;
				if (cmd === 'plugin:window|start_resize_dragging') return;
				// Tauri event plugin commands
				if (cmd === 'plugin:event|listen') return Date.now();
				if (cmd === 'plugin:event|unlisten') return;
				return null;
			};
		}, imageBase64, persistedLocked, persistedOpen, TINY_PNG_BASE64);

		// Dynamically import stickyEntry and render sticky UI
		const error = await browser.executeAsync((done) => {
			(async () => {
				try {
					const mod = await import('/src/sticky/sticky.js');
					const mainElement = document.querySelector('#mclocks');
					await mod.stickyEntry(mainElement);
					done(null);
				} catch (e) {
					done('stickyEntry failed: ' + e.message);
				}
			})();
		});
		if (error) {
			throw new Error(error);
		}
		await browser.pause(300);
	};

	const getUIState = async () => {
		return await browser.execute(() => {
			const textarea = document.getElementById('sticky-text');
			const img = document.getElementById('sticky-image');
			const copyBtn = document.getElementById('sticky-copy');
			const closeBtn = document.getElementById('sticky-close');
			const lockedMark = document.getElementById('sticky-locked-mark');
			const toggleBtn = document.getElementById('sticky-toggle');
			return {
				textareaDisplay: textarea ? getComputedStyle(textarea).display : null,
				textareaValue: textarea?.value ?? null,
				imgDisplay: img ? getComputedStyle(img).display : null,
				imgSrc: img?.src ?? null,
				copyDisplay: copyBtn ? getComputedStyle(copyBtn).display : null,
				closeVisible: closeBtn ? getComputedStyle(closeBtn).visibility !== 'hidden' : null,
				lockVisible: lockedMark ? getComputedStyle(lockedMark).visibility !== 'hidden' : null,
				toggleText: toggleBtn?.textContent ?? null,
			};
		});
	};

	const getTestResults = async () => {
		return await browser.execute(() => window.__stickyImageTest);
	};

	// --- Rendering tests ---

	it('should show <img> and hide <textarea> for image sticky', async () => {
		await setupImageStickyWindow();

		const state = await getUIState();
		expect(state.textareaDisplay).toBe('none');
		expect(state.imgDisplay).not.toBe('none');
		expect(state.imgSrc).toContain('data:image/png;base64,');
	});

	it('should show copy button for image sticky', async () => {
		await setupImageStickyWindow();

		const state = await getUIState();
		expect(state.copyDisplay).not.toBe('none');
	});

	// --- Lock tests ---

	it('should lock on Ctrl+l for image sticky', async () => {
		await setupImageStickyWindow();

		await browser.keys(['Control', 'l']);
		await browser.pause(100);

		const state = await getUIState();
		expect(state.closeVisible).toBe(false);
		expect(state.lockVisible).toBe(true);
	});

	it('should unlock on second Ctrl+l for image sticky', async () => {
		await setupImageStickyWindow();

		await browser.keys(['Control', 'l']);
		await browser.pause(100);
		await browser.keys(['Control', 'l']);
		await browser.pause(100);

		const state = await getUIState();
		expect(state.closeVisible).toBe(true);
		expect(state.lockVisible).toBe(false);
	});

	it('should restore persisted locked state for image sticky', async () => {
		await setupImageStickyWindow({ persistedLocked: true });

		const state = await getUIState();
		expect(state.closeVisible).toBe(false);
		expect(state.lockVisible).toBe(true);
	});

	it('should persist locked=true via save_sticky_state after Ctrl+l on image sticky', async () => {
		await setupImageStickyWindow();

		await browser.keys(['Control', 'l']);
		await browser.pause(1000);

		const results = await getTestResults();
		const saves = results.saveStateCalls;
		expect(saves.length).toBeGreaterThan(0);
		const lastSave = saves[saves.length - 1];
		expect(lastSave.locked).toBe(true);
	});

	// --- Close tests ---

	it('should call delete_sticky_text when close button is clicked on image sticky', async () => {
		await setupImageStickyWindow();

		const closeBtn = await $('#sticky-close');
		await closeBtn.click();
		await browser.pause(500);

		const results = await getTestResults();
		expect(results.deleteStickyCalled).toBe(true);
	});

	// --- Error handling tests ---

	it('should show error message when image file is missing', async () => {
		await setupImageStickyWindow({ imageBase64: null });

		const state = await getUIState();
		// Image hidden, textarea shown with error
		expect(state.imgDisplay).toBe('none');
		expect(state.textareaDisplay).not.toBe('none');
		expect(state.textareaValue).toBe('Error: Image file not found');
	});

	// --- Toggle tests ---

	it('should start in closed state by default', async () => {
		await setupImageStickyWindow();

		const state = await getUIState();
		expect(state.toggleText).toBe('▸');
	});

	it('should restore open state from persisted data', async () => {
		await setupImageStickyWindow({ persistedOpen: true });

		const state = await getUIState();
		expect(state.toggleText).toBe('▾');
	});
});
