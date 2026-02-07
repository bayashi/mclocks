describe('Sticky Note - Create from main window (Ctrl+s)', () => {
	const testConfig = {
		clocks: [
			{ name: 'UTC', timezone: 'UTC' },
			{ name: 'JST', timezone: 'Asia/Tokyo' }
		],
		epochClockName: 'Epoch',
		format: 'HH:mm:ss',
		timerIcon: '⏱',
		withoutNotification: false,
		maxTimerClockNumber: 10,
		usetz: false,
		convtz: null,
		disableHover: false,
		forefront: false,
		font: 'Courier, monospace',
		color: '#fff',
		size: '14px',
		locale: 'en',
		margin: '10px'
	};

	beforeEach(async () => {
		await browser.url('/');
		await browser.execute((config) => {
			sessionStorage.setItem('__defaultClockConfig', JSON.stringify(config));
			window.__defaultClockConfig = config;
		}, testConfig);
		await browser.refresh();
		await browser.waitUntil(
			async () => {
				const readyState = await browser.execute(() => document.readyState);
				return readyState === 'complete';
			},
			{ timeout: 10000, timeoutMsg: 'Page did not load after refresh' }
		);
		// Wait for clocks to initialize
		await browser.waitUntil(
			async () => {
				const count = await browser.execute(
					() => document.querySelectorAll('[id^="mclk-"]').length
				);
				return count >= 1;
			},
			{ timeout: 30000, interval: 1000, timeoutMsg: 'Clocks did not initialize' }
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
	 * Install invoke mock that intercepts Tauri IPC calls.
	 * Sets window.__clipboardText as the clipboard read return value.
	 * Tracks create_sticky and dialog calls on window.__stickyTest.
	 */
	const installInvokeMock = async (clipboardText) => {
		await browser.execute((text) => {
			window.__clipboardText = text;
			window.__stickyTest = {
				createStickyCalled: false,
				createStickyText: null,
				dialogCalled: false,
				dialogMessage: null,
			};

			const originalInvoke = window.__TAURI_INTERNALS__?.invoke;
			const mockInvoke = async function(cmd, args) {
				// Clipboard read
				if (cmd && typeof cmd === 'string' && (cmd.includes('clipboard') || cmd.includes('read_text'))) {
					const val = window.__clipboardText;
					if (val === null || val === undefined) {
						return '';
					}
					return val;
				}
				// Create sticky
				if (cmd === 'create_sticky') {
					window.__stickyTest.createStickyCalled = true;
					window.__stickyTest.createStickyText = args?.text ?? null;
					return;
				}
				// Dialog (message)
				if (cmd && typeof cmd === 'string' && cmd.includes('dialog')) {
					window.__stickyTest.dialogCalled = true;
					// args.message is the body of the dialog for Tauri v2 plugin
					window.__stickyTest.dialogMessage = args?.message ?? args?.body ?? JSON.stringify(args);
					return true;
				}
				// Fallback to original
				if (originalInvoke) {
					try {
						return await originalInvoke.call(this, cmd, args);
					} catch {
						return null;
					}
				}
				return null;
			};

			if (window.__TAURI_INTERNALS__) {
				window.__TAURI_INTERNALS__.invoke = mockInvoke;
			} else {
				window.__TAURI_INTERNALS__ = { invoke: mockInvoke };
			}
		}, clipboardText);
	};

	const getStickyTestResult = async () => {
		return await browser.execute(() => window.__stickyTest);
	};

	it('should call create_sticky with clipboard text when Ctrl+s is pressed', async () => {
		await installInvokeMock('Hello sticky note');

		console.log('Pressing Ctrl+s to create sticky...');
		await browser.keys(['Control', 's']);
		await browser.pause(1500);

		const result = await getStickyTestResult();
		console.log('create_sticky result:', JSON.stringify(result, null, 2));

		expect(result.createStickyCalled).toBe(true);
		expect(result.createStickyText).toBe('Hello sticky note');
	});

	it('should show dialog when Ctrl+s is pressed with empty clipboard', async () => {
		await installInvokeMock('');

		console.log('Pressing Ctrl+s with empty clipboard...');
		await browser.keys(['Control', 's']);
		await browser.pause(1500);

		const result = await getStickyTestResult();
		console.log('empty clipboard result:', JSON.stringify(result, null, 2));

		expect(result.createStickyCalled).toBe(false);
		expect(result.dialogCalled).toBe(true);
	});

	it('should show dialog when Ctrl+s is pressed with oversized clipboard text', async () => {
		// 128KB + 1 byte to exceed the limit
		const oversizedText = 'A'.repeat(128 * 1024);
		await installInvokeMock(oversizedText);

		console.log('Pressing Ctrl+s with oversized clipboard text...');
		await browser.keys(['Control', 's']);
		await browser.pause(1500);

		const result = await getStickyTestResult();
		console.log('oversized text result:', JSON.stringify(result, null, 2));

		expect(result.createStickyCalled).toBe(false);
		expect(result.dialogCalled).toBe(true);
	});

	it('should handle multiline clipboard text for sticky creation', async () => {
		const multilineText = 'Line 1\nLine 2\nLine 3';
		await installInvokeMock(multilineText);

		console.log('Pressing Ctrl+s with multiline text...');
		await browser.keys(['Control', 's']);
		await browser.pause(1500);

		const result = await getStickyTestResult();
		console.log('multiline result:', JSON.stringify(result, null, 2));

		expect(result.createStickyCalled).toBe(true);
		expect(result.createStickyText).toBe(multilineText);
	});
});
