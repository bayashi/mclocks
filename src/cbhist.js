import { invoke } from '@tauri-apps/api/core';
import { getCurrentWindow, LogicalSize } from '@tauri-apps/api/window';
import { isMacOS, openMessageDialog } from './util.js';

/** Same rules as sticky / clock `initClockStyles` size handling. */
function sizeToCssPx(size) {
	if (typeof size === 'number') {
		return `${size}px`;
	}
	if (typeof size === 'string') {
		if (/^[\d.]+$/.test(size)) {
			return `${size}px`;
		}
		return size;
	}
	return '14px';
}

async function setWindowSize(currentWindow, w, h) {
	await currentWindow.setSize(new LogicalSize(w, h));
}

async function getInnerSize(currentWindow) {
	try {
		const [inner, scaleFactor] = await Promise.all([
			currentWindow.innerSize(),
			currentWindow.scaleFactor(),
		]);
		const factor = scaleFactor || 1;
		return {
			width: Math.round(inner.width / factor),
			height: Math.round(inner.height / factor),
		};
	} catch {
		return null;
	}
}

const CBHIST_CLOSE_FADE_MS = 220;

let cbhistPanelClosing = false;

const BADGE_CLIP_TRUNC = 'Copied text truncated';

const EXPAND_A11Y_MORE = 'Show full text';
const EXPAND_A11Y_LESS = 'Show less';

function escapeHtml(s) {
	const d = document.createElement('div');
	d.textContent = s;
	return d.innerHTML;
}

function truncationBadge(show) {
	if (!show) {
		return '';
	}
	return `<span class="ch-badge" title="${escapeHtml(BADGE_CLIP_TRUNC)}">${escapeHtml(BADGE_CLIP_TRUNC)}</span>`;
}

function resolveUsedLineHeightPx(lhRaw, fontSizePx) {
	const lhStr = String(lhRaw ?? '').trim();
	const fsPx = Number.isFinite(fontSizePx) && fontSizePx > 0 ? fontSizePx : 13;
	if (!lhStr || lhStr === 'normal') {
		return { lhPxUsed: fsPx * 1.45 };
	}
	if (/calc\s*\(/i.test(lhStr)) {
		return { lhPxUsed: fsPx * 1.45 };
	}
	if (/px\b/i.test(lhStr)) {
		const v = Number.parseFloat(lhStr);
		return {
			lhPxUsed: Number.isFinite(v) ? v : fsPx * 1.45,
		};
	}
	if (/rem\b/i.test(lhStr)) {
		let rootPx = Number.parseFloat(getComputedStyle(document.documentElement).fontSize);
		if (!Number.isFinite(rootPx) || rootPx <= 0) {
			rootPx = 13;
		}
		const v = Number.parseFloat(lhStr);
		return {
			lhPxUsed: Number.isFinite(v) ? v * rootPx : fsPx * 1.45,
		};
	}
	if (/em\b/i.test(lhStr)) {
		const v = Number.parseFloat(lhStr);
		return {
			lhPxUsed: Number.isFinite(v) ? v * fsPx : fsPx * 1.45,
		};
	}
	if (/^[-+]?\d*\.?\d+$/.test(lhStr)) {
		const m = Number.parseFloat(lhStr);
		if (!Number.isFinite(m) || m <= 0) {
			return { lhPxUsed: fsPx * 1.45 };
		}
		if (m <= 5) {
			return { lhPxUsed: m * fsPx };
		}
		const barePxLo = fsPx * 0.52;
		const barePxHi = fsPx * 10;
		if (m >= barePxLo && m <= barePxHi) {
			return { lhPxUsed: m };
		}
		return { lhPxUsed: m * fsPx };
	}
	const scrape = Number.parseFloat(lhStr);
	return {
		lhPxUsed: Number.isFinite(scrape) ? scrape : fsPx * 1.45,
	};
}

function previewBudgetMetrics(textEl) {
	const cs = getComputedStyle(textEl);
	const fsParsed = Number.parseFloat(cs.fontSize);
	const fsPx = Number.isFinite(fsParsed) && fsParsed > 0 ? fsParsed : 13;
	const lhRes = resolveUsedLineHeightPx(cs.lineHeight, fsPx);
	const lhSafe = lhRes.lhPxUsed;
	const pt = Number.parseFloat(cs.paddingTop) || 0;
	const pb = Number.parseFloat(cs.paddingBottom) || 0;
	const linesRaw = getComputedStyle(document.documentElement).getPropertyValue('--ch-clamp-lines').trim();
	const lineCount = Number.parseInt(linesRaw, 10);
	const clampLines = Number.isFinite(lineCount) && lineCount > 0 ? lineCount : 6;
	const layoutSlackPx = 4;
	const budget = lhSafe * clampLines + pt + pb + layoutSlackPx;
	return { budget };
}

function syncVisualClamp(card) {
	const textEl = card.querySelector('.ch-card-text');
	const toggle = card.querySelector('.ch-expand-toggle');
	if (!textEl || !toggle) {
		return;
	}
	if (card.dataset.expanded === 'true') {
		textEl.classList.remove('is-preview-clamped');
		toggle.classList.remove('ch-expand-off');
		toggle.removeAttribute('aria-hidden');
		toggle.removeAttribute('tabindex');
		return;
	}

	const gapPx = 2;
	const meta = previewBudgetMetrics(textEl);

	let overflows;
	let naturalHUnclamped = null;
	const hadClamp = textEl.classList.contains('is-preview-clamped');
	if (hadClamp) {
		void textEl.offsetHeight;
		const ceilSh = Math.ceil(textEl.scrollHeight);
		const floorCh = Math.floor(textEl.clientHeight);
		const geomOverflow = ceilSh > floorCh + gapPx;
		overflows =
			geomOverflow || card.dataset.chExpandLatch === '1';
	} else {
		textEl.classList.remove('is-preview-clamped');
		void textEl.offsetHeight;
		naturalHUnclamped = textEl.offsetHeight;
		overflows = naturalHUnclamped > meta.budget + gapPx;
		if (overflows) {
			card.dataset.chExpandLatch = '1';
		} else {
			delete card.dataset.chExpandLatch;
		}
	}

	if (overflows) {
		textEl.classList.add('is-preview-clamped');
		toggle.classList.remove('ch-expand-off');
		toggle.removeAttribute('aria-hidden');
		toggle.removeAttribute('tabindex');
	} else {
		textEl.classList.remove('is-preview-clamped');
		void textEl.offsetHeight;
		delete card.dataset.chExpandLatch;
		toggle.classList.add('ch-expand-off');
		toggle.setAttribute('aria-hidden', 'true');
		toggle.tabIndex = -1;
	}
}

function scheduleClampSync(root) {
	const run = () => {
		root.querySelectorAll('.ch-card').forEach(syncVisualClamp);
	};
	run();
	requestAnimationFrame(run);
}

function historyPayloadUnchanged(prev, data) {
	if (!Array.isArray(data) || prev.length !== data.length) {
		return false;
	}
	for (let i = 0; i < data.length; i++) {
		const a = prev[i];
		const b = data[i];
		if (!a || a.text !== b.text || Boolean(a.truncatedFromClipboard) !== Boolean(b.truncatedFromClipboard)) {
			return false;
		}
	}
	return true;
}

async function closePanel() {
	if (cbhistPanelClosing) {
		return;
	}
	cbhistPanelClosing = true;
	document.documentElement.classList.add('cbhist-is-closing');
	window.setTimeout(async () => {
		try {
			await invoke('cbhist_close_panel');
		} finally {
			document.documentElement.classList.remove('cbhist-is-closing');
			cbhistPanelClosing = false;
		}
	}, CBHIST_CLOSE_FADE_MS);
}

/** @typedef {{ text: string, utf8ByteLen: number, unicodeScalarCount: number, lineCount: number, truncatedFromClipboard: boolean }} CbhistRow */

export async function cbhistPanelEntry(mainElement) {
	/** @type {CbhistRow[]} */
	let rows = [];
	const copyFlashTimers = new Map();

	let cfg = null;
	try {
		cfg = await invoke('load_config', {});
	} catch {
		// cfg remains null
	}
	let closeAfterCopyTimer = null;
	document.documentElement.classList.remove('cbhist-is-closing');
	if (cfg) {
		document.documentElement.style.fontFamily = cfg.font;
		document.documentElement.style.fontSize = sizeToCssPx(cfg.size);
		document.documentElement.style.color = cfg.color;
	}

	mainElement.innerHTML = '';
	mainElement.classList.add('ch-cbhist-root');
	mainElement.innerHTML = `
<div class="ch-shell">
	<div class="ch-top-drag" data-tauri-drag-region></div>
	<button type="button" class="ch-close-x" id="ch-close" aria-label="Close">✖</button>
	<div class="ch-main">
		<div class="ch-list-scroll" id="ch-list-scroll"></div>
	</div>
	<div id="cbhist-resize-handle" aria-hidden="true"></div>
</div>
`;

	let currentWindow = null;
	try {
		currentWindow = getCurrentWindow();
	} catch {
		currentWindow = null;
	}
	const resizeHandle = mainElement.querySelector('#cbhist-resize-handle');

	const listScroll = mainElement.querySelector('#ch-list-scroll');
	const closeBtn = mainElement.querySelector('#ch-close');
	const topDrag = mainElement.querySelector('.ch-top-drag');

	const fmt = (n) => Number(n).toLocaleString();

	function clearCopyFlashTimers() {
		for (const id of copyFlashTimers.values()) {
			clearTimeout(id);
		}
		copyFlashTimers.clear();
	}

	function flashCopyFeedback(card, ix) {
		card.classList.remove('is-copy-flash');
		void card.offsetWidth;
		card.classList.add('is-copy-flash');
		const cur = copyFlashTimers.get(ix);
		if (cur != null) {
			clearTimeout(cur);
		}
		const tid = window.setTimeout(() => {
			copyFlashTimers.delete(ix);
			card.classList.remove('is-copy-flash');
		}, 200);
		copyFlashTimers.set(ix, tid);
	}

	function renderEmpty(messageHtml) {
		clearCopyFlashTimers();
		if (!listScroll) {
			return;
		}
		listScroll.innerHTML = `<div class="ch-empty-msg">${messageHtml}</div>`;
	}

	function cardHtml(row, index) {
		const meta = `${fmt(row.unicodeScalarCount)} chars · ${fmt(row.lineCount)} lines`;
		const textEscaped = escapeHtml(row.text ?? '');
		return `
<article class="ch-card" data-index="${index}" data-expanded="false">
	<div class="ch-card-zone">
		<div class="ch-card-text-mount">
			<div class="ch-card-text">${textEscaped}</div>
		</div>
	</div>
	<footer class="ch-entry-footer">
		<div class="ch-entry-footer-main">
			<div class="ch-entry-footer-lead">
				<span class="ch-footer-meta">${escapeHtml(meta)}</span>
				${truncationBadge(row.truncatedFromClipboard)}
			</div>
			<button type="button" class="ch-expand-toggle ch-expand-reveal ch-expand-off" data-index="${index}" aria-expanded="false" aria-hidden="true" aria-label="${escapeHtml(EXPAND_A11Y_MORE)}" title="${escapeHtml(EXPAND_A11Y_MORE)}" tabindex="-1">
				<span class="ch-expand-reveal-chevron" aria-hidden="true"></span>
				<span class="ch-expand-reveal-bar" aria-hidden="true"></span>
			</button>
		</div>
	</footer>
</article>`;
	}

	function toggleCardExpanded(card, expand) {
		const idx = Number(card.dataset.index);
		const row = rows[idx];
		if (!row) {
			return;
		}
		if (!expand) {
			delete card.dataset.chExpandLatch;
		}
		const textEl = card.querySelector('.ch-card-text');
		const toggle = card.querySelector('.ch-expand-toggle');
		if (!textEl || !toggle) {
			return;
		}
		const mount = card.querySelector('.ch-card-text-mount');
		if (expand) {
			textEl.textContent = row.text ?? '';
			card.dataset.expanded = 'true';
			toggle.setAttribute('aria-expanded', 'true');
			toggle.setAttribute('aria-label', EXPAND_A11Y_LESS);
			toggle.setAttribute('title', EXPAND_A11Y_LESS);
			if (mount) {
				mount.setAttribute('aria-expanded', 'true');
			}
		} else {
			textEl.textContent = row.text ?? '';
			card.dataset.expanded = 'false';
			toggle.setAttribute('aria-expanded', 'false');
			toggle.setAttribute('aria-label', EXPAND_A11Y_MORE);
			toggle.setAttribute('title', EXPAND_A11Y_MORE);
			if (mount) {
				mount.setAttribute('aria-expanded', 'false');
			}
		}
		syncVisualClamp(card);
	}

	function wireCards() {
		if (!listScroll) {
			return;
		}
		listScroll.querySelectorAll('.ch-expand-toggle').forEach((btn) => {
			btn.addEventListener('click', (e) => {
				e.stopPropagation();
				const card = btn.closest('.ch-card');
				if (!card) {
					return;
				}
				const wantExpand = card.dataset.expanded !== 'true';
				toggleCardExpanded(card, wantExpand);
			});
		});
		listScroll.querySelectorAll('.ch-card').forEach((card) => {
			card.addEventListener('click', async (e) => {
				if (e.target.closest('button')) {
					return;
				}
				const ix = Number(card.dataset.index);
				if (Number.isNaN(ix)) {
					return;
				}
				try {
					await invoke('cbhist_apply', { index: ix });
					flashCopyFeedback(card, ix);
					if (cfg?.clipboard?.closeOnCopy === true) {
						if (closeAfterCopyTimer != null) {
							clearTimeout(closeAfterCopyTimer);
						}
						closeAfterCopyTimer = window.setTimeout(() => {
							closeAfterCopyTimer = null;
							void closePanel();
						}, 600);
					}
				} catch (err) {
					console.warn('[cbhist] copy failed', err);
				}
			});
		});
	}

	function renderList(resetScroll = true) {
		clearCopyFlashTimers();
		if (!listScroll) {
			return;
		}
		if (resetScroll) {
			listScroll.scrollTop = 0;
		}
		const parts = rows.map((row, i) => cardHtml(row, i));
		listScroll.innerHTML = `<div class="ch-list">${parts.join('')}</div>`;
		wireCards();
		scheduleClampSync(listScroll);
	}

	closeBtn?.addEventListener('click', () => closePanel());

	if (isMacOS() && currentWindow) {
		topDrag?.addEventListener('mousedown', async (event) => {
			if (event.target.closest('button')) {
				return;
			}
			try {
				await currentWindow.startDragging();
			} catch {
				// ignore
			}
		});
	}

	if (resizeHandle && currentWindow) {
		resizeHandle.addEventListener('mousedown', async (event) => {
			event.preventDefault();
			if (isMacOS()) {
				const startX = event.screenX;
				const startY = event.screenY;
				const startSize = await getInnerSize(currentWindow);
				if (!startSize) {
					return;
				}
				let rafPending = false;
				let lastX = startX;
				let lastY = startY;
				const onMouseMove = () => {
					if (rafPending) {
						return;
					}
					rafPending = true;
					requestAnimationFrame(() => {
						rafPending = false;
						const dx = lastX - startX;
						const dy = lastY - startY;
						void setWindowSize(
							currentWindow,
							Math.max(200, startSize.width + dx),
							Math.max(200, startSize.height + dy),
						);
					});
				};
				const cleanup = () => {
					window.removeEventListener('mousemove', onMouseMoveCapture, true);
					window.removeEventListener('mouseup', cleanup, true);
				};
				const onMouseMoveCapture = (e) => {
					lastX = e.screenX;
					lastY = e.screenY;
					onMouseMove();
				};
				window.addEventListener('mousemove', onMouseMoveCapture, true);
				window.addEventListener('mouseup', cleanup, true);
			} else {
				try {
					await currentWindow.startResizeDragging('SouthEast');
				} catch (error) {
					await openMessageDialog(`Failed to start resize: ${error}`, 'mclocks Error', 'error');
				}
			}
		});
	}

	let resizeSaveTimer = null;
	if (currentWindow) {
		try {
			await currentWindow.onResized(async () => {
				const inner = await getInnerSize(currentWindow);
				if (!inner) {
					return;
				}
				if (resizeSaveTimer != null) {
					clearTimeout(resizeSaveTimer);
				}
				resizeSaveTimer = setTimeout(async () => {
					resizeSaveTimer = null;
					try {
						await invoke('save_clipboard_panel_size', {
							width: inner.width,
							height: inner.height,
						});
					} catch {
						// ignore
					}
				}, 400);
			});
		} catch {
			// ignore (API unavailable)
		}
	}

	async function reload(opts = {}) {
		const resetScroll = opts.resetScroll !== false;
		let data;
		try {
			data = await invoke('cbhist_list');
		} catch {
			if (!resetScroll) {
				return;
			}
			rows = [];
			renderEmpty('<span class="ch-empty-inner">Failed to load Clipboard History.</span>');
			return;
		}
		if (!Array.isArray(data)) {
			if (!resetScroll) {
				return;
			}
			rows = [];
			renderEmpty('<span class="ch-empty-inner">Failed to load Clipboard History.</span>');
			return;
		}
		if (data.length === 0) {
			if (!resetScroll && rows.length === 0) {
				return;
			}
			rows = [];
			if (listScroll) {
				listScroll.scrollTop = 0;
			}
			renderEmpty('<span class="ch-empty-inner">No clipboard text yet.<br>Copy something elsewhere, then reopen this panel.</span>');
			return;
		}
		if (historyPayloadUnchanged(rows, data)) {
			if (resetScroll && listScroll) {
				listScroll.scrollTop = 0;
			}
			return;
		}
		rows = data;
		renderList(resetScroll);
	}

	window.addEventListener('keydown', (e) => {
		if (e.key === 'Escape') {
			e.preventDefault();
			void closePanel();
		}
	});

	window.addEventListener('focus', () => {
		document.documentElement.classList.remove('cbhist-is-closing');
		void reload({ resetScroll: true });
	});

	const listPollMs = 200;
	window.setInterval(() => {
		void reload({ resetScroll: false });
	}, listPollMs);

	await reload();
}
