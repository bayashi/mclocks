describe('Sticky Note - Ctrl+s creates new sticky from sticky window', () => {
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
	 * including clipboard read and create_sticky IPC calls.
	 *
	 * @param {string|null} clipboardText - Text to return from clipboard read mock
	 */
	const setupStickyWindow = async (clipboardText) => {
		await browser.execute((cbText) => {
			window.__stickyCtrlSTest = {
				createStickyCalled: false,
				createStickyText: null,
				dialogCalled: false,
				dialogMessage: null,
			};

			window.__TAURI_INTERNALS__ = window.__TAURI_INTERNALS__ || {};
			window.__TAURI_INTERNALS__.metadata = {
				currentWindow: { label: 'sticky-test-ctrl-s' },
			};
			window.__TAURI_INTERNALS__.transformCallback = function() {
				return Date.now() + Math.random();
			};
			window.__TAURI_INTERNALS__.invoke = async function(cmd, args) {
				// Clipboard read
				if (cmd && typeof cmd === 'string' && (cmd.includes('clipboard') || cmd.includes('read_text'))) {
					const val = cbText;
					if (val === null || val === undefined) {
						return '';
					}
					return val;
				}
				// Create sticky
				if (cmd === 'create_sticky') {
					window.__stickyCtrlSTest.createStickyCalled = true;
					window.__stickyCtrlSTest.createStickyText = args?.text ?? null;
					return;
				}
				// Dialog (message)
				if (cmd && typeof cmd === 'string' && cmd.includes('dialog')) {
					window.__stickyCtrlSTest.dialogCalled = true;
					window.__stickyCtrlSTest.dialogMessage = args?.message ?? args?.body ?? JSON.stringify(args);
					return true;
				}
				// App IPC commands for sticky window init
				if (cmd === 'load_config') {
					return {
						forefront: false,
						font: 'Courier, monospace',
						size: '14px',
						color: '#fff',
					};
				}
				if (cmd === 'sticky_take_init_text') return 'Existing sticky text';
				if (cmd === 'load_sticky_state') return null;
				if (cmd === 'save_sticky_state') return;
				if (cmd === 'save_sticky_text') return;
				if (cmd === 'save_window_state_exclusive') return;
				if (cmd === 'delete_sticky_text') return;
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
		}, clipboardText);

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

	const getTestResult = async () => {
		return await browser.execute(() => window.__stickyCtrlSTest);
	};

	it('should call create_sticky with clipboard text when Ctrl+s is pressed in sticky window', async () => {
		await setupStickyWindow('Hello from sticky');

		console.log('Pressing Ctrl+s in sticky window...');
		await browser.keys(['Control', 's']);
		await browser.pause(1500);

		const result = await getTestResult();
		console.log('create_sticky result:', JSON.stringify(result, null, 2));

		expect(result.createStickyCalled).toBe(true);
		expect(result.createStickyText).toBe('Hello from sticky');
	});

	it('should show dialog when Ctrl+s is pressed with empty clipboard in sticky window', async () => {
		await setupStickyWindow('');

		console.log('Pressing Ctrl+s with empty clipboard in sticky window...');
		await browser.keys(['Control', 's']);
		await browser.pause(1500);

		const result = await getTestResult();
		console.log('empty clipboard result:', JSON.stringify(result, null, 2));

		expect(result.createStickyCalled).toBe(false);
		expect(result.dialogCalled).toBe(true);
	});

	it('should show dialog when Ctrl+s is pressed with oversized clipboard text in sticky window', async () => {
		const oversizedText = 'A'.repeat(128 * 1024);
		await setupStickyWindow(oversizedText);

		console.log('Pressing Ctrl+s with oversized text in sticky window...');
		await browser.keys(['Control', 's']);
		await browser.pause(1500);

		const result = await getTestResult();
		console.log('oversized text result:', JSON.stringify(result, null, 2));

		expect(result.createStickyCalled).toBe(false);
		expect(result.dialogCalled).toBe(true);
	});

	it('should handle multiline clipboard text when Ctrl+s is pressed in sticky window', async () => {
		const multilineText = 'Line 1\nLine 2\nLine 3';
		await setupStickyWindow(multilineText);

		console.log('Pressing Ctrl+s with multiline text in sticky window...');
		await browser.keys(['Control', 's']);
		await browser.pause(1500);

		const result = await getTestResult();
		console.log('multiline result:', JSON.stringify(result, null, 2));

		expect(result.createStickyCalled).toBe(true);
		expect(result.createStickyText).toBe(multilineText);
	});
});
