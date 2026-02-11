describe('Sticky Note - Forefront button behavior', () => {
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
	 * Set up Tauri mocks and initialize sticky window UI.
	 * Mocks __TAURI_INTERNALS__ to simulate a sticky window environment,
	 * then dynamically imports and calls stickyEntry().
	 *
	 * @param {boolean} configForefront - Main clock forefront config value
	 * @param {boolean|null} persistedForefront - Persisted per-sticky forefront (null = no persisted state)
	 */
	const setupStickyWindow = async (configForefront, persistedForefront = null) => {
		await browser.execute((cfgForefront, persForefront) => {
			window.__stickyForefrontTest = {
				setAlwaysOnTopCalls: [],
				saveStateCalls: [],
			};

			window.__TAURI_INTERNALS__ = window.__TAURI_INTERNALS__ || {};
			window.__TAURI_INTERNALS__.metadata = {
				currentWindow: { label: 'sticky-test-forefront' },
			};
			window.__TAURI_INTERNALS__.transformCallback = function() {
				return Date.now() + Math.random();
			};
			window.__TAURI_INTERNALS__.invoke = async function(cmd, args) {
				// App IPC commands
				if (cmd === 'load_config') {
					return {
						forefront: cfgForefront,
						font: 'Courier, monospace',
						size: '14px',
						color: '#fff',
					};
				}
				if (cmd === 'sticky_take_init_content') return { text: 'Test text', contentType: null };
				if (cmd === 'load_sticky_state') {
					if (persForefront !== null && persForefront !== undefined) {
						return {
							isOpen: false,
							openWidth: null,
							openHeight: null,
							forefront: persForefront,
						};
					}
					return null;
				}
				if (cmd === 'save_sticky_state') {
					window.__stickyForefrontTest.saveStateCalls.push(JSON.parse(JSON.stringify(args)));
					return;
				}
				if (cmd === 'save_sticky_text') return;
				if (cmd === 'save_window_state_exclusive') return;
				if (cmd === 'delete_sticky_text') return;
				// Tauri window plugin commands
				if (cmd === 'plugin:window|set_always_on_top') {
					window.__stickyForefrontTest.setAlwaysOnTopCalls.push(JSON.parse(JSON.stringify(args)));
					return;
				}
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
		}, configForefront, persistedForefront);

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

	const getButtonState = async () => {
		return await browser.execute(() => {
			const btn = document.getElementById('sticky-forefront');
			return {
				text: btn?.textContent ?? null,
				title: btn?.title ?? null,
			};
		});
	};

	const getTestResults = async () => {
		return await browser.execute(() => window.__stickyForefrontTest);
	};

	// --- Initial state tests ---

	it('should show ⊤ with "Keep forefront" when config forefront is false', async () => {
		await setupStickyWindow(false);

		const state = await getButtonState();
		expect(state.text).toBe('⊤');
		expect(state.title).toBe('Keep forefront');
	});

	it('should show ⊥ with "Behind others" when config forefront is true', async () => {
		await setupStickyWindow(true);

		const state = await getButtonState();
		expect(state.text).toBe('⊥');
		expect(state.title).toBe('Behind others');
	});

	it('should call setAlwaysOnTop(false) on init when config forefront is false', async () => {
		await setupStickyWindow(false);

		const results = await getTestResults();
		const calls = results.setAlwaysOnTopCalls;
		expect(calls.length).toBeGreaterThanOrEqual(1);
		expect(calls[0].value).toBe(false);
	});

	it('should call setAlwaysOnTop(true) on init when config forefront is true', async () => {
		await setupStickyWindow(true);

		const results = await getTestResults();
		const calls = results.setAlwaysOnTopCalls;
		expect(calls.length).toBeGreaterThanOrEqual(1);
		expect(calls[0].value).toBe(true);
	});

	// --- Persisted state override tests ---

	it('should use persisted forefront=true over config forefront=false', async () => {
		await setupStickyWindow(false, true);

		const state = await getButtonState();
		expect(state.text).toBe('⊥');
		expect(state.title).toBe('Behind others');

		const results = await getTestResults();
		const calls = results.setAlwaysOnTopCalls;
		expect(calls[0].value).toBe(true);
	});

	it('should use persisted forefront=false over config forefront=true', async () => {
		await setupStickyWindow(true, false);

		const state = await getButtonState();
		expect(state.text).toBe('⊤');
		expect(state.title).toBe('Keep forefront');

		const results = await getTestResults();
		const calls = results.setAlwaysOnTopCalls;
		expect(calls[0].value).toBe(false);
	});

	// --- Click toggle tests ---

	it('should toggle to forefront on click when initially non-forefront', async () => {
		await setupStickyWindow(false);

		const btn = await browser.$('#sticky-forefront');
		await btn.click();
		await browser.pause(100);

		const state = await getButtonState();
		expect(state.text).toBe('⊥');
		expect(state.title).toBe('Behind others');

		const results = await getTestResults();
		const calls = results.setAlwaysOnTopCalls;
		const lastCall = calls[calls.length - 1];
		expect(lastCall.value).toBe(true);
	});

	it('should toggle to non-forefront on click when initially forefront', async () => {
		await setupStickyWindow(true);

		const btn = await browser.$('#sticky-forefront');
		await btn.click();
		await browser.pause(100);

		const state = await getButtonState();
		expect(state.text).toBe('⊤');
		expect(state.title).toBe('Keep forefront');

		const results = await getTestResults();
		const calls = results.setAlwaysOnTopCalls;
		const lastCall = calls[calls.length - 1];
		expect(lastCall.value).toBe(false);
	});

	it('should toggle back to original state on double click', async () => {
		await setupStickyWindow(false);

		const btn = await browser.$('#sticky-forefront');

		// First click: enable forefront
		await btn.click();
		await browser.pause(100);
		let state = await getButtonState();
		expect(state.text).toBe('⊥');

		// Second click: disable forefront
		await btn.click();
		await browser.pause(100);
		state = await getButtonState();
		expect(state.text).toBe('⊤');
		expect(state.title).toBe('Keep forefront');

		const results = await getTestResults();
		const calls = results.setAlwaysOnTopCalls;
		const lastCall = calls[calls.length - 1];
		expect(lastCall.value).toBe(false);
	});

	// --- Persistence tests ---

	it('should persist forefront=true via save_sticky_state after enabling', async () => {
		await setupStickyWindow(false);

		const btn = await browser.$('#sticky-forefront');
		await btn.click();
		// Wait for debounced save (500ms + buffer)
		await browser.pause(1000);

		const results = await getTestResults();
		const saves = results.saveStateCalls;
		expect(saves.length).toBeGreaterThan(0);
		const lastSave = saves[saves.length - 1];
		expect(lastSave.forefront).toBe(true);
	});

	it('should persist forefront=false via save_sticky_state after disabling', async () => {
		await setupStickyWindow(true);

		const btn = await browser.$('#sticky-forefront');
		await btn.click();
		// Wait for debounced save (500ms + buffer)
		await browser.pause(1000);

		const results = await getTestResults();
		const saves = results.saveStateCalls;
		expect(saves.length).toBeGreaterThan(0);
		const lastSave = saves[saves.length - 1];
		expect(lastSave.forefront).toBe(false);
	});
});
