export async function setupMainAppWithTauriMocks() {
	const error = await browser.executeAsync((done) => {
		(async () => {
			try {
				window.__webDndTest = {
					invokeCalls: [],
					listenCalls: [],
					registerCalls: [],
					lastOpenedUrl: null,
				};
				window.__tauriTestCallbacks = {};

				let callbackSeq = 1;
				let listenSeq = 1;

				window.__TAURI_INTERNALS__ = window.__TAURI_INTERNALS__ || {};
				window.__TAURI_INTERNALS__.metadata = {
					currentWindow: { label: 'main' },
				};
				window.__TAURI_INTERNALS__.transformCallback = function(cb) {
					const id = callbackSeq++;
					window.__tauriTestCallbacks[id] = cb;
					return id;
				};
				window.__TAURI_INTERNALS__.invoke = async function(cmd, args) {
					window.__webDndTest.invokeCalls.push({ cmd, args: args || null });

					if (cmd === 'load_config') {
						return {
							clocks: [{ name: 'UTC', timezone: 'UTC' }],
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
							size: 14,
							margin: '10px',
							locale: 'en',
						};
					}

					if (cmd === 'restore_stickies') return;
					if (cmd === 'save_window_state_exclusive') return;
					if (cmd === 'plugin:window|set_always_on_top') return;
					if (cmd === 'plugin:window|start_dragging') return;
					if (cmd === 'plugin:event|unlisten') return;

					if (cmd === 'plugin:event|listen') {
						window.__webDndTest.listenCalls.push(args || null);
						return listenSeq++;
					}

					if (cmd === 'register_temp_web_root') {
						window.__webDndTest.registerCalls.push(args || null);
						const openedUrl = 'http://127.0.0.1:3030/tmpdir-testhash/?mode=source';
						window.__webDndTest.lastOpenedUrl = openedUrl;
						return openedUrl;
					}

					return null;
				};

				await import('/src/app.js');
				window.dispatchEvent(new Event('DOMContentLoaded'));
				await new Promise((resolve) => setTimeout(resolve, 100));
				done(null);
			} catch (e) {
				done(`setup failed: ${e?.message || String(e)}`);
			}
		})();
	});
	if (error) {
		throw new Error(error);
	}
}

export async function emitDragDropPayload(payload) {
	return await browser.execute(async (dropPayload) => {
		const listeners = window.__webDndTest?.listenCalls || [];
		const callbacks = window.__tauriTestCallbacks || {};
		let fired = 0;

		const dragDropEntries = listeners.filter((entry) => {
			const serialized = JSON.stringify(entry || {}).toLowerCase();
			return serialized.includes('drag-drop');
		});

		for (const entry of dragDropEntries) {
			const handlerId = entry?.handler
				?? entry?.payload?.handler
				?? entry?.options?.handler
				?? entry?.event?.handler
				?? null;
			const cb = callbacks[handlerId];
			if (typeof cb === 'function') {
				await cb({ payload: dropPayload });
				fired++;
			}
		}

		return {
			fired,
			listeners,
			registerCalls: window.__webDndTest?.registerCalls || [],
		};
	}, payload);
}

export async function getDndTestState() {
	return await browser.execute(() => {
		return {
			registerCalls: window.__webDndTest?.registerCalls || [],
			lastOpenedUrl: window.__webDndTest?.lastOpenedUrl || null,
		};
	});
}
