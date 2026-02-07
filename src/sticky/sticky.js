import { invoke } from '@tauri-apps/api/core';
import { getCurrentWindow, LogicalSize } from '@tauri-apps/api/window';

import { writeClipboardText, openMessageDialog, isMacOS } from '../util.js';

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

	try {
		const initText = await invoke('sticky_take_init_text', { id: label });
		textarea.value = initText ?? '';
	} catch {
		textarea.value = '';
	}

	// Load persisted open/close state and open-mode size
	let stickyState = null;
	try {
		stickyState = await invoke('load_sticky_state', { id: label });
	} catch {
		// ignore
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

	// Restore open-mode size from persisted state
	if (stickyState) {
		if (stickyState.openWidth != null && stickyState.openHeight != null) {
			savedOpenSize = { width: stickyState.openWidth, height: stickyState.openHeight };
			savedWidth = stickyState.openWidth;
			userResized = true;
		}
	}

	const setProgrammaticSize = async (width, height) => {
		lastProgrammaticSize = { width, height };
		await setWindowSize(currentWindow, width, height);
	};

	// Debounced save of open/close state and open-mode size
	const scheduleStateSave = () => {
		if (stateSaveDebouncerId != null) {
			clearTimeout(stateSaveDebouncerId);
		}
		stateSaveDebouncerId = setTimeout(async () => {
			stateSaveDebouncerId = null;
			try {
				await invoke('save_sticky_state', {
					id: label,
					isOpen: isOpen,
					openWidth: savedOpenSize?.width ?? null,
					openHeight: savedOpenSize?.height ?? null,
				});
			} catch {
				// ignore
			}
		}, 500);
	};

	// Save window state via window-state plugin (flag pattern, same as main window)
	// Only called from onMoved to avoid conflicts with programmatic resizes.
	// Flag blocks subsequent onMoved triggers until save completes (matches app.js pattern).
	// pointer-events:none blocks user interaction during save to prevent OS modal loop deadlock.
	const scheduleStickyWindowStateSave = () => {
		if (isMacOS() || ignoreStickyWindowStateSave) {
			return;
		}
		ignoreStickyWindowStateSave = true;
		stickyWindowStateSaveId = setTimeout(async () => {
			stickyWindowStateSaveId = null;
			// Block user interaction to prevent OS modal loop during save
			stickyRoot.style.pointerEvents = 'none';
			try {
				await invoke('save_window_state_exclusive');
			} catch {
				// ignore
			} finally {
				stickyRoot.style.pointerEvents = '';
				ignoreStickyWindowStateSave = false;
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
		isOpen = true;
		await ensureOpenSize();
	};

	const closeSticky = async () => {
		if (isOpen) {
			const inner = await getInnerSize(currentWindow);
			if (inner) {
				savedWidth = inner.width;
				savedOpenSize = { width: inner.width, height: inner.height };
			}
		}
		isOpen = false;
		await ensureClosedSize();
	};

	toggleButton.addEventListener('click', async () => {
		try {
			if (isOpen) {
				await closeSticky();
			} else {
				await openSticky();
			}
			scheduleStateSave();
		} catch (error) {
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
			if (!isOpen) {
				return;
			}
			const inner = await getInnerSize(currentWindow);
			if (!inner) {
				return;
			}

			if (lastProgrammaticSize) {
				const dw = Math.abs(inner.width - lastProgrammaticSize.width);
				const dh = Math.abs(inner.height - lastProgrammaticSize.height);
				if (dw <= 2 && dh <= 2) {
					return;
				}
			}

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
			} catch {
				// ignore
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
