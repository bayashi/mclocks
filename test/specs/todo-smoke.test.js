describe('TODO panel smoke', () => {
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

	const TODO_LOAD_CONFIG = {
		font: 'Courier, monospace',
		size: '14px',
		color: '#fff',
		forefront: false,
		todoStatuses: ['WILL', 'DOING', 'BLOCKED', 'DONE'],
	};

	/**
	 * Mock Tauri invoke / window APIs and boot todo UI into #mclocks.
	 * @param {{ items?: object[], forefront?: boolean|null }} persist - todo_load payload
	 */
	const setupTodoPanel = async (persist = {}) => {
		const persistPayload = {
			items: persist.items ?? [],
			forefront: persist.forefront ?? null,
		};

		await browser.execute(
			(configPayload, loaded) => {
				window.__todoSmokeTest = {
					saveCalls: [],
				};

				window.__TAURI_INTERNALS__ = window.__TAURI_INTERNALS__ || {};
				window.__TAURI_INTERNALS__.metadata = {
					currentWindow: { label: 'todo' },
				};
				window.__TAURI_INTERNALS__.transformCallback = function () {
					return Date.now() + Math.random();
				};
				window.__TAURI_INTERNALS__.invoke = async function (cmd, args) {
					if (cmd === 'load_config') {
						return configPayload;
					}
					if (cmd === 'todo_load') {
						return loaded;
					}
					if (cmd === 'todo_save') {
						window.__todoSmokeTest.saveCalls.push(JSON.parse(JSON.stringify(args)));
						return;
					}
					if (cmd === 'todo_close_panel') {
						return;
					}
					if (cmd === 'save_window_state_exclusive') {
						return;
					}
					if (cmd === 'plugin:window|set_always_on_top') {
						return;
					}
					if (cmd === 'plugin:window|set_size') {
						return;
					}
					if (cmd === 'plugin:window|inner_size') {
						return { width: 450, height: 320 };
					}
					if (cmd === 'plugin:window|scale_factor') {
						return 1;
					}
					if (cmd === 'plugin:window|start_resize_dragging') {
						return;
					}
					if (cmd === 'plugin:window|start_dragging') {
						return;
					}
					if (cmd === 'plugin:event|listen') {
						return Date.now();
					}
					if (cmd === 'plugin:event|unlisten') {
						return;
					}
					return null;
				};
			},
			TODO_LOAD_CONFIG,
			persistPayload,
		);

		const error = await browser.executeAsync((done) => {
			(async () => {
				try {
					const mod = await import('/src/todo/todo.js');
					const mainElement = document.querySelector('#mclocks');
					await mod.todoPanelEntry(mainElement);
					done(null);
				} catch (e) {
					done(e.message);
				}
			})();
		});
		if (error) {
			throw new Error(error);
		}
		await browser.pause(300);
	};

	it('shows shell, add button, and resize handle', async () => {
		await setupTodoPanel();

		const shell = await browser.execute(() => ({
			root: !!document.getElementById('todo-root'),
			add: !!document.getElementById('todo-add'),
			addLabel: document.getElementById('todo-add')?.textContent ?? null,
			resize: !!document.getElementById('todo-resize-handle'),
			forefront: !!document.getElementById('todo-forefront'),
			close: !!document.getElementById('todo-close'),
		}));

		expect(shell.root).toBe(true);
		expect(shell.add).toBe(true);
		expect(shell.addLabel).toBe('+TODO');
		expect(shell.resize).toBe(true);
		expect(shell.forefront).toBe(true);
		expect(shell.close).toBe(true);
	});

	it('adds an item, accepts text input, and persists via todo_save', async () => {
		await setupTodoPanel();

		await browser.execute(() => {
			document.getElementById('todo-add').click();
		});

		await browser.waitUntil(
			async () => {
				const count = await browser.execute(
					() => document.querySelectorAll('.todo-item').length,
				);
				return count === 1;
			},
			{ timeout: 5000, timeoutMsg: 'Expected one TODO row after +TODO' },
		);

		await browser.execute(() => {
			const input = document.querySelector('.todo-text');
			if (!(input instanceof HTMLInputElement)) {
				return;
			}
			input.value = 'smoke todo item';
			input.dispatchEvent(new Event('input', { bubbles: true }));
		});

		await browser.waitUntil(
			async () => {
				const calls = await browser.execute(() => window.__todoSmokeTest.saveCalls);
				if (!Array.isArray(calls) || calls.length === 0) {
					return false;
				}
				const last = calls[calls.length - 1];
				const items = last?.items;
				return Array.isArray(items) && items.some((it) => it.text === 'smoke todo item');
			},
			{ timeout: 5000, timeoutMsg: 'todo_save was not called with typed text' },
		);

		const lastSave = await browser.execute(() => {
			const calls = window.__todoSmokeTest.saveCalls;
			return calls[calls.length - 1];
		});
		expect(lastSave.items.length).toBe(1);
		expect(lastSave.items[0].status).toBe('WILL');
		expect(lastSave.items[0].text).toBe('smoke todo item');
	});

	it('cycles status on click (WILL → DOING)', async () => {
		await setupTodoPanel({
			items: [
				{
					id: 'todo-smoke-1',
					text: 'cycle me',
					status: 'WILL',
					memo: '',
				},
			],
		});

		const before = await browser.execute(() => {
			const btn = document.querySelector('.todo-status');
			return btn?.dataset.status ?? null;
		});
		expect(before).toBe('WILL');

		await browser.execute(() => {
			document.querySelector('.todo-status').click();
		});

		const after = await browser.execute(() => {
			const btn = document.querySelector('.todo-status');
			return {
				status: btn?.dataset.status ?? null,
				label: btn?.textContent ?? null,
			};
		});
		expect(after.status).toBe('DOING');
		expect(after.label).toBe('DOING');

		await browser.waitUntil(
			async () => {
				const calls = await browser.execute(() => window.__todoSmokeTest.saveCalls);
				if (!Array.isArray(calls) || calls.length === 0) {
					return false;
				}
				const last = calls[calls.length - 1];
				return last?.items?.[0]?.status === 'DOING';
			},
			{ timeout: 5000, timeoutMsg: 'todo_save was not called with DOING status' },
		);
	});
});
