import http from 'node:http';
import { randomBytes } from 'node:crypto';

function escapeSingleQuotedJsString(s) {
	return s.replaceAll('\\', '\\\\').replaceAll('\'', '\\\'');
}

function readRequestBody(req) {
	return new Promise((resolve, reject) => {
		let data = '';
		req.setEncoding('utf8');
		req.on('data', (chunk) => {
			data += chunk;
		});
		req.on('end', () => resolve(data));
		req.on('error', reject);
	});
}

function createEditorHtml(token) {
	const t = escapeSingleQuotedJsString(token);
	return `<!DOCTYPE html>
<html>
<head>
<meta charset="utf-8">
<style>
body {
	font-family: monospace;
	margin: 20px;
	background-color: #f5f5f5;
}
.error {
	background-color: #fff;
	border: 1px solid #ddd;
	border-radius: 4px;
	padding: 20px;
	margin: 20px 0;
}
.error h2 {
	color: #d32f2f;
	margin-top: 0;
}
.error p {
	color: #666;
	margin: 10px 0;
}
</style>
</head>
<body>
<div id="result"></div>
<script>
window.__editorCloseCalled = false;
let data = new Object();
data.token = '${t}';
let pathname = document.location.pathname;
if (pathname.startsWith('/editor')) {
	pathname = pathname.substring(7);
	if (!pathname.startsWith('/')) {
		pathname = '/' + pathname;
	}
}
data.path = pathname;
let hash = document.location.hash;
let line = null;
if (hash && hash.startsWith('#L')) {
	let lineNum = parseInt(hash.substring(2));
	if (!isNaN(lineNum)) {
		line = lineNum;
	}
}
data.line = line;
let request = new XMLHttpRequest();
request.open('POST', '/editor');
request.setRequestHeader('Content-Type', 'application/json');
request.onload = function() {
	if (request.status === 200) {
		window.__editorCloseCalled = true;
		window.close();
	} else {
		document.getElementById('result').innerHTML = request.responseText;
	}
};
request.onerror = function() {
	document.getElementById('result').innerHTML = '<div class="error"><h2>Error</h2><p>Network error occurred.</p></div>';
};
request.send(JSON.stringify(data));
</script>
</body>
</html>`;
}

describe('/editor endpoint JS E2E', () => {
	let server;
	let baseUrl;
	let lastPost;
	let sockets;

	const tokenStore = new Map();

	before(async () => {
		sockets = new Set();
		server = http.createServer(async (req, res) => {
			try {
				const url = new URL(req.url, 'http://127.0.0.1');
				if (req.method === 'GET' && (url.pathname === '/editor' || url.pathname.startsWith('/editor/'))) {
					const expectedPath = url.pathname.replace(/^\/editor/, '') || '/';
					const token = randomBytes(16).toString('hex');
					const expiresAt = url.searchParams.get('mode') === 'expired' ? Date.now() - 1 : Date.now() + 60_000;
					tokenStore.set(token, { expectedPath, expiresAt });
					res.writeHead(200, { 'content-type': 'text/html; charset=utf-8' });
					res.end(createEditorHtml(token));
					return;
				}

				if (req.method === 'POST' && url.pathname === '/editor') {
					const body = await readRequestBody(req);
					let json;
					try {
						json = JSON.parse(body);
					} catch {
						res.writeHead(400, { 'content-type': 'text/html; charset=utf-8' });
						res.end('<div class="error"><h2>Error</h2><p>Bad Request: Invalid JSON</p></div>');
						return;
					}

					lastPost = {
						path: typeof json.path === 'string' ? json.path : null,
						line: typeof json.line === 'number' ? json.line : null,
						token: typeof json.token === 'string' ? json.token : null,
					};

					const token = lastPost.token || '';
					const entry = tokenStore.get(token);
					const now = Date.now();
					const ok = !!entry && entry.expiresAt > now && entry.expectedPath === lastPost.path;
					if (!ok) {
						res.writeHead(403, { 'content-type': 'text/html; charset=utf-8' });
						res.end('<div class="error"><h2>Error</h2><p>Forbidden: Invalid or expired token. Reload and try again.</p></div>');
						return;
					}

					tokenStore.delete(token);
					res.writeHead(200, { 'content-type': 'text/plain; charset=utf-8' });
					res.end('OK');
					return;
				}

				res.writeHead(404, { 'content-type': 'text/plain; charset=utf-8' });
				res.end('Not Found');
			} catch {
				res.writeHead(500, { 'content-type': 'text/plain; charset=utf-8' });
				res.end('Internal Server Error');
			}
		});
		server.on('connection', (socket) => {
			sockets.add(socket);
			socket.on('close', () => sockets.delete(socket));
		});

		await new Promise((resolve) => server.listen(0, '127.0.0.1', resolve));
		const addr = server.address();
		baseUrl = `http://127.0.0.1:${addr.port}`;
	});

	after(async () => {
		if (!server) {
			return;
		}
		for (const socket of sockets) {
			try {
				socket.destroy();
			} catch {
			}
		}
		await new Promise((resolve) => server.close(resolve));
	});

	beforeEach(async () => {
		lastPost = null;
		tokenStore.clear();
		await browser.url('about:blank');
	});

	it('should POST path/line and close window on 200', async () => {
		const originalHandle = await browser.getWindowHandle();
		const handlesBefore = await browser.getWindowHandles();

		await browser.newWindow(`${baseUrl}/editor/o/r/blob/main/src/lib.rs#L42`);

		await browser.waitUntil(
			async () => {
				const handles = await browser.getWindowHandles();
				return handles.length === handlesBefore.length + 1;
			},
			{
				timeout: 3000,
				timeoutMsg: 'Editor window did not open',
				interval: 100,
			},
		);

		const handlesAfterOpen = await browser.getWindowHandles();
		const editorHandle = handlesAfterOpen.find((h) => !handlesBefore.includes(h));
		expect(editorHandle).not.toBe(undefined);

		await browser.switchToWindow(editorHandle);

		await browser.waitUntil(
			async () => {
				const closeCalled = await browser.execute(() => window.__editorCloseCalled === true);
				return closeCalled === true && lastPost !== null;
			},
			{
				timeout: 10000,
				timeoutMsg: 'Editor close was not attempted',
				interval: 200,
			},
		);

		const closeCalled = await browser.execute(() => window.__editorCloseCalled === true);
		expect(closeCalled).toBe(true);

		const maybeError = await $('#result').getText();
		expect(maybeError.trim()).toBe('');

		await browser.closeWindow();
		await browser.switchToWindow(originalHandle);

		expect(lastPost).not.toBe(null);
		expect(lastPost.path).toBe('/o/r/blob/main/src/lib.rs');
		expect(lastPost.line).toBe(42);
	});

	it('should render error HTML on 403 and keep window open', async () => {
		const originalHandle = await browser.getWindowHandle();
		const handlesBefore = await browser.getWindowHandles();

		await browser.newWindow(`${baseUrl}/editor/o/r/blob/main/src/lib.rs?mode=expired#L42`);

		await browser.waitUntil(
			async () => {
				const handles = await browser.getWindowHandles();
				return handles.length === handlesBefore.length + 1;
			},
			{
				timeout: 3000,
				timeoutMsg: 'Editor window did not open',
				interval: 100,
			},
		);

		const handles = await browser.getWindowHandles();
		const editorHandle = handles.find((h) => !handlesBefore.includes(h));
		expect(editorHandle).not.toBe(undefined);
		await browser.switchToWindow(editorHandle);

		const result = await $('#result');
		await result.waitForExist({ timeout: 5000 });
		await browser.waitUntil(
			async () => {
				const text = await result.getText();
				return text.includes('Forbidden') || text.includes('Invalid or expired token');
			},
			{
				timeout: 10000,
				timeoutMsg: 'Error message was not rendered',
				interval: 200,
			},
		);

		const closeCalled = await browser.execute(() => window.__editorCloseCalled === true);
		expect(closeCalled).toBe(false);

		const handlesAfter = await browser.getWindowHandles();
		expect(handlesAfter).toContain(editorHandle);

		await browser.closeWindow();
		await browser.switchToWindow(originalHandle);
	});
});
