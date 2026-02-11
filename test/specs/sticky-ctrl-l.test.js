describe('Sticky Note - Ctrl+l lock/unlock behavior', () => {
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
	 * Mocks __TAURI_INTERNALS__ to simulate a sticky window environment.
	 *
	 * @param {boolean|null} persistedLocked - Persisted locked state (null = no persisted state)
	 */
	const setupStickyWindow = async (persistedLocked = null) => {
		await browser.execute((persLocked) => {
			window.__stickyLockTest = {
				saveStateCalls: [],
			};

			window.__TAURI_INTERNALS__ = window.__TAURI_INTERNALS__ || {};
			window.__TAURI_INTERNALS__.metadata = {
				currentWindow: { label: 'sticky-test-ctrl-l' },
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
				if (cmd === 'sticky_take_init_text') return 'Test text';
				if (cmd === 'load_sticky_state') {
					if (persLocked !== null && persLocked !== undefined) {
						return {
							isOpen: false,
							openWidth: null,
							openHeight: null,
							forefront: null,
							locked: persLocked,
						};
					}
					return null;
				}
				if (cmd === 'save_sticky_state') {
					window.__stickyLockTest.saveStateCalls.push(JSON.parse(JSON.stringify(args)));
					return;
				}
				if (cmd === 'save_sticky_text') return;
				if (cmd === 'save_window_state_exclusive') return;
				if (cmd === 'delete_sticky_text') return;
				// Clipboard
				if (cmd && typeof cmd === 'string' && (cmd.includes('clipboard') || cmd.includes('read_text'))) {
					return 'clipboard text';
				}
				if (cmd === 'create_sticky') return;
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
		}, persistedLocked);

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
			const closeBtn = document.getElementById('sticky-close');
			const lockedMark = document.getElementById('sticky-locked-mark');
			const textarea = document.getElementById('sticky-text');
			return {
				closeVisible: closeBtn ? getComputedStyle(closeBtn).visibility !== 'hidden' : null,
				lockVisible: lockedMark ? getComputedStyle(lockedMark).visibility !== 'hidden' : null,
				textareaReadOnly: textarea?.readOnly ?? null,
			};
		});
	};

	const getTestResults = async () => {
		return await browser.execute(() => window.__stickyLockTest);
	};

	// --- Initial state tests ---

	it('should show close button and hide lock mark by default (no persisted state)', async () => {
		await setupStickyWindow(null);

		const state = await getUIState();
		expect(state.closeVisible).toBe(true);
		expect(state.lockVisible).toBe(false);
		expect(state.textareaReadOnly).toBe(false);
	});

	// --- Ctrl+l toggle tests ---

	it('should lock on Ctrl+l: hide close button, show lock mark, make textarea readOnly', async () => {
		await setupStickyWindow(null);

		await browser.keys(['Control', 'l']);
		await browser.pause(100);

		const state = await getUIState();
		expect(state.closeVisible).toBe(false);
		expect(state.lockVisible).toBe(true);
		expect(state.textareaReadOnly).toBe(true);
	});

	it('should unlock on second Ctrl+l: show close button, hide lock mark, editable textarea', async () => {
		await setupStickyWindow(null);

		// First Ctrl+l: lock
		await browser.keys(['Control', 'l']);
		await browser.pause(100);

		// Second Ctrl+l: unlock
		await browser.keys(['Control', 'l']);
		await browser.pause(100);

		const state = await getUIState();
		expect(state.closeVisible).toBe(true);
		expect(state.lockVisible).toBe(false);
		expect(state.textareaReadOnly).toBe(false);
	});

	// --- Restore persisted state tests ---

	it('should restore locked state from persisted data', async () => {
		await setupStickyWindow(true);

		const state = await getUIState();
		expect(state.closeVisible).toBe(false);
		expect(state.lockVisible).toBe(true);
		expect(state.textareaReadOnly).toBe(true);
	});

	it('should restore unlocked state when persisted locked is false', async () => {
		await setupStickyWindow(false);

		const state = await getUIState();
		expect(state.closeVisible).toBe(true);
		expect(state.lockVisible).toBe(false);
		expect(state.textareaReadOnly).toBe(false);
	});

	// --- Persistence tests ---

	it('should persist locked=true via save_sticky_state after Ctrl+l', async () => {
		await setupStickyWindow(null);

		await browser.keys(['Control', 'l']);
		// Wait for debounced save (500ms + buffer)
		await browser.pause(1000);

		const results = await getTestResults();
		const saves = results.saveStateCalls;
		expect(saves.length).toBeGreaterThan(0);
		const lastSave = saves[saves.length - 1];
		expect(lastSave.locked).toBe(true);
	});

	it('should persist locked=null via save_sticky_state after unlocking', async () => {
		await setupStickyWindow(null);

		// Lock
		await browser.keys(['Control', 'l']);
		await browser.pause(600);

		// Unlock
		await browser.keys(['Control', 'l']);
		await browser.pause(1000);

		const results = await getTestResults();
		const saves = results.saveStateCalls;
		expect(saves.length).toBeGreaterThan(1);
		const lastSave = saves[saves.length - 1];
		expect(lastSave.locked).toBeNull();
	});

	// --- Click lock mark to unlock ---

	it('should unlock when clicking the lock mark', async () => {
		await setupStickyWindow(true);

		const lockMark = await browser.$('#sticky-locked-mark');
		await lockMark.click();
		await browser.pause(100);

		const state = await getUIState();
		expect(state.closeVisible).toBe(true);
		expect(state.lockVisible).toBe(false);
		expect(state.textareaReadOnly).toBe(false);
	});

	it('should persist unlocked state after clicking lock mark', async () => {
		await setupStickyWindow(true);

		const lockMark = await browser.$('#sticky-locked-mark');
		await lockMark.click();
		// Wait for debounced save (500ms + buffer)
		await browser.pause(1000);

		const results = await getTestResults();
		const saves = results.saveStateCalls;
		expect(saves.length).toBeGreaterThan(0);
		const lastSave = saves[saves.length - 1];
		expect(lastSave.locked).toBeNull();
	});
});
