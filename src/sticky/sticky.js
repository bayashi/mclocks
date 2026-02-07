import { invoke } from '@tauri-apps/api/core';
import { getCurrentWindow, LogicalSize } from '@tauri-apps/api/window';

import { writeClipboardText, openMessageDialog, isMacOS } from '../util.js';

const debugLog = import.meta.env.DEV
	? (...args) => console.info(...args)
	: () => {};

const MAX_OPEN_LINES = 12;

function getLineHeightPx(el) {
	const style = getComputedStyle(el);
	const lh = parseFloat(style.lineHeight);
	if (!Number.isNaN(lh) && lh > 0) {
		return lh;
	}
	const fs = parseFloat(style.fontSize);
	if (!Number.isNaN(fs) && fs > 0) {
		return fs * 1.2;
	}
	return 16;
}

function getVPaddingPx(el) {
	const style = getComputedStyle(el);
	const pt = parseFloat(style.paddingTop) || 0;
	const pb = parseFloat(style.paddingBottom) || 0;
	return pt + pb;
}

function getVBorderPx(el) {
	const style = getComputedStyle(el);
	const bt = parseFloat(style.borderTopWidth) || 0;
	const bb = parseFloat(style.borderBottomWidth) || 0;
	return bt + bb;
}

function measureSingleLineBoxHeightPx(referenceEl) {
	const style = getComputedStyle(referenceEl);
	const probe = document.createElement('span');
	probe.textContent = 'A';
	probe.style.position = 'absolute';
	probe.style.visibility = 'hidden';
	probe.style.whiteSpace = 'pre';
	probe.style.fontFamily = style.fontFamily;
	probe.style.fontSize = style.fontSize;
	probe.style.fontWeight = style.fontWeight;
	probe.style.fontStyle = style.fontStyle;
	probe.style.letterSpacing = style.letterSpacing;
	probe.style.lineHeight = style.lineHeight;
	document.body.appendChild(probe);
	const h = probe.getBoundingClientRect().height;
	probe.remove();
	return Math.ceil(h);
}

function sizeToCssPx(size) {
	if (typeof size === "number") {
		return `${size}px`;
	}
	if (typeof size === "string") {
		if (/^[\d.]+$/.test(size)) {
			return `${size}px`;
		}
		return size;
	}
	return "14px";
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

function clamp(n, min, max) {
	return Math.max(min, Math.min(max, n));
}

export async function stickyEntry(mainElement) {
	document.documentElement.classList.add('sticky');

	mainElement.innerHTML = `<div id="sticky-root" class="sticky-closed">
<div id="sticky-header">
<button id="sticky-toggle" type="button" aria-label="Toggle open">▸</button>
<button id="sticky-copy" type="button" aria-label="Copy text">⧉</button>
<div id="sticky-spacer"></div>
<button id="sticky-close" type="button" aria-label="Close">✖</button>
</div>
<textarea id="sticky-text" spellcheck="false"></textarea>
<div id="sticky-resize-handle" aria-hidden="true"></div>
</div>`;

	const currentWindow = getCurrentWindow();
	debugLog('[sticky] stickyEntry: start');

	const stickyRoot = document.getElementById('sticky-root');
	const toggleButton = document.getElementById('sticky-toggle');
	const copyButton = document.getElementById('sticky-copy');
	const closeButton = document.getElementById('sticky-close');
	const textarea = document.getElementById('sticky-text');
	const resizeHandle = document.getElementById('sticky-resize-handle');

	let cfg = null;
	try {
		cfg = await invoke("load_config", {});
	} catch {
		// cfg remains null
	}

	if (cfg) {
		document.documentElement.style.fontFamily = cfg.font;
		document.documentElement.style.fontSize = sizeToCssPx(cfg.size);
		document.documentElement.style.color = cfg.color;
		if (cfg.forefront) {
			try {
				await currentWindow.setAlwaysOnTop(true);
			} catch {
				// ignore
			}
		}
	}

	let label = '';
	try {
		label = currentWindow.label;
	} catch {
		label = '';
	}
	debugLog('[sticky] stickyEntry: window label=%s', label);

	try {
		const initText = await invoke('sticky_take_init_text', { id: label });
		textarea.value = initText ?? '';
		debugLog('[sticky] sticky_take_init_text: ok length=%d', textarea.value?.length ?? 0);
	} catch {
		textarea.value = '';
		debugLog('[sticky] sticky_take_init_text: failed');
	}

	// Load persisted open/close state and open-mode size
	let stickyState = null;
	try {
		stickyState = await invoke('load_sticky_state', { id: label });
		debugLog('[sticky] load_sticky_state: %o', stickyState);
	} catch {
		debugLog('[sticky] load_sticky_state: failed');
	}

	// isOpen is true when the sticky is in expanded (open) state
	let isOpen = false;
	// savedOpenSize holds the last open-mode window size { width, height } to restore on re-open
	let savedOpenSize = null;
	// savedWidth holds the last known window width, preserved across open/close transitions
	let savedWidth = null;
	// userResized becomes true once the user manually resizes; disables auto-sizing on text input
	let userResized = false;
	// lastProgrammaticSize holds the size from the last programmatic resize, for tolerance check in onResized
	let lastProgrammaticSize = null;
	// copyFeedbackDelayId is a pending delay id for copy button feedback reset
	let copyFeedbackDelayId = null;
	// copyButtonDefaultText holds the original copy button label, captured on first click
	let copyButtonDefaultText = null;
	// saveDebouncerId is a pending debounce id for text save (save_sticky_text)
	let saveDebouncerId = null;
	// stateSaveDebouncerId is a pending debounce id for open/close state save (save_sticky_state)
	let stateSaveDebouncerId = null;
	// ignoreStickyWindowStateSave is a flag to block subsequent onMoved triggers until save_window_state_exclusive completes
	let ignoreStickyWindowStateSave = false;
	// stickyWindowStateSaveId is a pending delay id for window-state plugin save, cancelled on close
	let stickyWindowStateSaveId = null;
	debugLog('[sticky] isMacOS=%s', isMacOS());

	// Restore open-mode size from persisted state
	if (stickyState) {
		if (stickyState.openWidth != null && stickyState.openHeight != null) {
			savedOpenSize = { width: stickyState.openWidth, height: stickyState.openHeight };
			savedWidth = stickyState.openWidth;
			userResized = true;
		}
	}

	const setProgrammaticSize = async (width, height) => {
		debugLog('[sticky] setProgrammaticSize: %dx%d for %s', width, height, label);
		lastProgrammaticSize = { width, height };
		await setWindowSize(currentWindow, width, height);
		debugLog('[sticky] setProgrammaticSize: done for %s', label);
	};

	// Debounced save of open/close state and open-mode size
	const scheduleStateSave = () => {
		debugLog('[sticky] scheduleStateSave: called for %s isOpen=%s savedOpenSize=%o', label, isOpen, savedOpenSize);
		if (stateSaveDebouncerId != null) {
			clearTimeout(stateSaveDebouncerId);
		}
		stateSaveDebouncerId = setTimeout(async () => {
			stateSaveDebouncerId = null;
			debugLog('[sticky] scheduleStateSave: firing for %s', label);
			try {
				await invoke('save_sticky_state', {
					id: label,
					isOpen: isOpen,
					openWidth: savedOpenSize?.width ?? null,
					openHeight: savedOpenSize?.height ?? null,
				});
				debugLog('[sticky] scheduleStateSave: done for %s', label);
			} catch (error) {
				debugLog('[sticky] scheduleStateSave: failed for %s:', label, error);
			}
		}, 500);
	};

	// Save window state via window-state plugin (flag pattern, same as main window)
	// Only called from onMoved to avoid conflicts with programmatic resizes.
	// Flag blocks subsequent onMoved triggers until save completes (matches app.js pattern).
	// pointer-events:none blocks user interaction during save to prevent OS modal loop deadlock.
	const scheduleStickyWindowStateSave = () => {
		if (isMacOS() || ignoreStickyWindowStateSave) {
			debugLog('[sticky] scheduleStickyWindowStateSave: skip for %s (macOS=%s ignore=%s)', label, isMacOS(), ignoreStickyWindowStateSave);
			return;
		}
		ignoreStickyWindowStateSave = true;
		debugLog('[sticky] scheduleStickyWindowStateSave: scheduled for %s', label);
		stickyWindowStateSaveId = setTimeout(async () => {
			stickyWindowStateSaveId = null;
			// Block user interaction to prevent OS modal loop during save
			stickyRoot.style.pointerEvents = 'none';
			debugLog('[sticky] scheduleStickyWindowStateSave: firing for %s', label);
			try {
				await invoke('save_window_state_exclusive');
				debugLog('[sticky] scheduleStickyWindowStateSave: done for %s', label);
			} catch (error) {
				debugLog('[sticky] scheduleStickyWindowStateSave: error for %s: %s', label, error);
			} finally {
				stickyRoot.style.pointerEvents = '';
				ignoreStickyWindowStateSave = false;
				debugLog('[sticky] scheduleStickyWindowStateSave: unlocked for %s', label);
			}
		}, 5000);
	};

	const measureContentHeight = async () => {
		const prevMainH = mainElement.style.height;
		const prevRootH = stickyRoot.style.height;
		mainElement.style.height = 'auto';
		stickyRoot.style.height = 'auto';
		await new Promise((r) => requestAnimationFrame(r));
		const h = mainElement.offsetHeight;
		mainElement.style.height = prevMainH;
		stickyRoot.style.height = prevRootH;
		return h;
	};

	const desiredOpenTextHeight = () => {
		const lineHeight = getLineHeightPx(textarea);
		const padding = getVPaddingPx(textarea);
		const minTextHeight = Math.ceil(lineHeight + padding);
		const maxTextHeight = Math.ceil((lineHeight * MAX_OPEN_LINES) + padding);
		return clamp(textarea.scrollHeight, minTextHeight, maxTextHeight);
	};

	const ensureClosedSize = async () => {
		stickyRoot.classList.remove('sticky-open');
		stickyRoot.classList.add('sticky-closed');
		toggleButton.textContent = '▸';

		textarea.style.overflowY = 'hidden';

		const lineHeight = measureSingleLineBoxHeightPx(textarea);
		const padding = getVPaddingPx(textarea);
		const border = getVBorderPx(textarea);
		const oneLineTextHeight = Math.ceil(lineHeight + padding + border);
		textarea.style.height = `${oneLineTextHeight}px`;
		textarea.scrollTop = 0;

		await new Promise((r) => requestAnimationFrame(r));

		const inner = await getInnerSize(currentWindow);
		const width = savedWidth ?? inner?.width ?? 360;
		savedWidth = width;

		const needHeight = await measureContentHeight();
		await setProgrammaticSize(width, needHeight);
	};

	const ensureOpenSize = async () => {
		stickyRoot.classList.remove('sticky-closed');
		stickyRoot.classList.add('sticky-open');
		toggleButton.textContent = '▾';
		textarea.style.overflowY = 'auto';
		textarea.style.height = '';

		await new Promise((r) => requestAnimationFrame(r));

		if (savedOpenSize) {
			const width = savedOpenSize.width;
			savedWidth = width;
			await setProgrammaticSize(width, savedOpenSize.height);
			return;
		}

		const textHeight = desiredOpenTextHeight();
		textarea.style.height = `${textHeight}px`;
		await new Promise((r) => requestAnimationFrame(r));

		const inner = await getInnerSize(currentWindow);
		const width = savedWidth ?? inner?.width ?? 360;
		savedWidth = width;

		const needHeight = await measureContentHeight();
		textarea.style.height = '';
		savedOpenSize = { width, height: needHeight };
		await setProgrammaticSize(width, needHeight);
	};

	const openSticky = async () => {
		debugLog('[sticky] openSticky: start for %s', label);
		isOpen = true;
		await ensureOpenSize();
		debugLog('[sticky] openSticky: done for %s', label);
	};

	const closeSticky = async () => {
		debugLog('[sticky] closeSticky: start for %s isOpen=%s', label, isOpen);
		if (isOpen) {
			const inner = await getInnerSize(currentWindow);
			if (inner) {
				savedWidth = inner.width;
				savedOpenSize = { width: inner.width, height: inner.height };
				debugLog('[sticky] closeSticky: captured open size %o for %s', savedOpenSize, label);
			}
		}
		isOpen = false;
		await ensureClosedSize();
		debugLog('[sticky] closeSticky: done for %s', label);
	};

	toggleButton.addEventListener('click', async () => {
		debugLog('[sticky] toggle: click isOpen=%s for %s', isOpen, label);
		try {
			if (isOpen) {
				await closeSticky();
			} else {
				await openSticky();
			}
			scheduleStateSave();
		} catch (error) {
			debugLog('[sticky] toggle: error for %s: %s', label, error);
			await openMessageDialog(`Failed to toggle sticky: ${error}`, "mclocks Error", "error");
		}
	});

	copyButton.addEventListener('click', async () => {
		try {
			if (copyButtonDefaultText == null) {
				copyButtonDefaultText = copyButton.textContent;
			}
			await writeClipboardText(textarea.value ?? '');
			copyButton.classList.add('is-copied');
			copyButton.textContent = '✓';
			if (copyFeedbackDelayId != null) {
				clearTimeout(copyFeedbackDelayId);
			}
			copyFeedbackDelayId = setTimeout(() => {
				copyButton.classList.remove('is-copied');
				copyButton.textContent = copyButtonDefaultText;
				copyFeedbackDelayId = null;
			}, 500);
		} catch (error) {
			await openMessageDialog(`Failed to copy: ${error}`, "mclocks Error", "error");
		}
	});

	closeButton.addEventListener('click', async () => {
		try {
			if (saveDebouncerId != null) {
				clearTimeout(saveDebouncerId);
				saveDebouncerId = null;
			}
			if (stateSaveDebouncerId != null) {
				clearTimeout(stateSaveDebouncerId);
				stateSaveDebouncerId = null;
			}
			if (stickyWindowStateSaveId != null) {
				clearTimeout(stickyWindowStateSaveId);
				stickyWindowStateSaveId = null;
				ignoreStickyWindowStateSave = false;
			}
			await invoke('delete_sticky_text', { id: label });
			debugLog('[sticky] deleted persistent data for %s', label);
			await currentWindow.close();
		} catch (error) {
			await openMessageDialog(`Failed to close sticky: ${error}`, "mclocks Error", "error");
		}
	});

	resizeHandle.addEventListener('mousedown', async (event) => {
		event.preventDefault();
		if (!isOpen) {
			return;
		}
		try {
			await currentWindow.startResizeDragging('SouthEast');
		} catch (error) {
			await openMessageDialog(`Failed to start resize: ${error}`, "mclocks Error", "error");
		}
	});

	try {
		await currentWindow.onResized(async () => {
			debugLog('[sticky] onResized: fired for %s isOpen=%s', label, isOpen);
			if (!isOpen) {
				debugLog('[sticky] onResized: skip (closed) for %s', label);
				return;
			}
			const inner = await getInnerSize(currentWindow);
			if (!inner) {
				debugLog('[sticky] onResized: skip (no inner) for %s', label);
				return;
			}

			if (lastProgrammaticSize) {
				const dw = Math.abs(inner.width - lastProgrammaticSize.width);
				const dh = Math.abs(inner.height - lastProgrammaticSize.height);
				if (dw <= 2 && dh <= 2) {
					debugLog('[sticky] onResized: skip (within tolerance) for %s dw=%d dh=%d', label, dw, dh);
					return;
				}
			}

			debugLog('[sticky] onResized: user resize detected %dx%d for %s', inner.width, inner.height, label);
			userResized = true;
			savedOpenSize = { width: inner.width, height: inner.height };
			savedWidth = inner.width;
			scheduleStateSave();
		});
	} catch {
		// ignore
	}

	// Save window position on move (non-macOS only, flag pattern)
	try {
		await currentWindow.onMoved(() => {
			debugLog('[sticky] onMoved: fired for %s ignore=%s macOS=%s', label, ignoreStickyWindowStateSave, isMacOS());
			scheduleStickyWindowStateSave();
		});
	} catch {
		// ignore
	}

	textarea.addEventListener('input', async () => {
		// Debounced save to persistent store
		if (saveDebouncerId != null) {
			clearTimeout(saveDebouncerId);
		}
		saveDebouncerId = setTimeout(async () => {
			saveDebouncerId = null;
			try {
				await invoke('save_sticky_text', { id: label, text: textarea.value });
				debugLog('[sticky] saved text for %s', label);
			} catch (error) {
				debugLog('[sticky] save failed:', error);
			}
		}, 500);

		if (!isOpen || userResized) {
			return;
		}
		const textHeight = desiredOpenTextHeight();
		textarea.style.height = `${textHeight}px`;
		await new Promise((r) => requestAnimationFrame(r));

		const width = savedWidth ?? 360;
		savedWidth = width;

		const needHeight = await measureContentHeight();
		textarea.style.height = '';
		savedOpenSize = { width, height: needHeight };
		await setProgrammaticSize(width, needHeight);
	});

	// Restore open/close state from persisted data
	if (stickyState?.isOpen) {
		await openSticky();
	} else {
		await closeSticky();
	}
}
