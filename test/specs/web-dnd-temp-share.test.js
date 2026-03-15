import {
	emitDragDropPayload,
	getDndTestState,
	setupMainAppWithTauriMocks,
} from '../helpers/web-dnd-test-helper.js';

describe('Web DnD temp-share flow', () => {
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

	it('should call register_temp_web_root when directory drop is received', async () => {
		await setupMainAppWithTauriMocks();

		const droppedPath = 'C:/tmp/mclocks-drop-dir';
		const firedResult = await emitDragDropPayload({
			type: 'drop',
			paths: [droppedPath],
			position: { Physical: { x: 100, y: 80 } },
		});
		console.log('Drag-drop callbacks fired:', JSON.stringify(firedResult, null, 2));

		await browser.waitUntil(
			async () => {
				const status = await browser.execute(() => {
					const registerCalls = window.__webDndTest?.registerCalls || [];
					return {
						count: registerCalls.length,
						first: registerCalls[0] || null,
					};
				});
				return status.count > 0;
			},
			{
				timeout: 5000,
				timeoutMsg: 'register_temp_web_root was not called after drop',
				interval: 100,
			}
		);

		const result = await getDndTestState();
		expect(result.registerCalls.length).toBeGreaterThan(0);
		expect(result.registerCalls[0].droppedPath).toBe(droppedPath);
		expect(result.lastOpenedUrl).toContain('/tmpdir-');
	});

	it('should ignore drop payload without paths', async () => {
		await setupMainAppWithTauriMocks();

		const firedResult = await emitDragDropPayload({
			type: 'drop',
			paths: [],
			position: { Physical: { x: 100, y: 80 } },
		});
		console.log('Drag-drop callbacks fired (empty paths):', JSON.stringify(firedResult, null, 2));

		await browser.pause(300);
		const result = await getDndTestState();
		expect(result.registerCalls.length).toBe(0);
	});
});
