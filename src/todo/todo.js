import { invoke } from '@tauri-apps/api/core';
import { getCurrentWindow, LogicalSize } from '@tauri-apps/api/window';
import { ask } from '@tauri-apps/plugin-dialog';

import { escapeHTML, isMacOS, openMessageDialog } from '../util.js';

const DEFAULT_STATUSES = ['WILL', 'DOING', 'BLOCKED', 'DONE'];
const SAVE_DEBOUNCE_MS = 400;
const MIN_WIDTH = 280;
const MIN_HEIGHT = 200;
const DRAG_THRESHOLD_PX = 5;
const TRASH_ICON_SVG = `<svg class="todo-trash-icon" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" aria-hidden="true" focusable="false"><g fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round"><path d="M5 7h14"/><path d="M9 7V5.5A1.5 1.5 0 0 1 10.5 4h3A1.5 1.5 0 0 1 15 5.5V7"/><path d="M8 7l.7 12.2A1.5 1.5 0 0 0 10.2 20.5h3.6a1.5 1.5 0 0 0 1.5-1.3L16 7"/><path d="M10 11v6"/><path d="M14 11v6"/></g></svg>`;

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

function newTodoId() {
	if (typeof crypto !== 'undefined' && typeof crypto.randomUUID === 'function') {
		return crypto.randomUUID();
	}
	return `todo-${Date.now()}-${Math.random().toString(16).slice(2)}`;
}

function normalizeStatuses(raw) {
	if (!Array.isArray(raw)) {
		return [...DEFAULT_STATUSES];
	}
	const cleaned = raw.map((s) => String(s ?? '').trim()).filter((s) => s.length > 0);
	return cleaned.length > 0 ? cleaned : [...DEFAULT_STATUSES];
}

function nextStatus(current, statuses) {
	const idx = statuses.indexOf(current);
	if (idx < 0) {
		return statuses[0];
	}
	return statuses[(idx + 1) % statuses.length];
}

/** Insert position helper for vertical list reorder. */
function dragInsertBeforeElement(container, y, draggingEl) {
	const els = [...container.querySelectorAll('.todo-item')].filter((el) => el !== draggingEl);
	let closest = null;
	let closestOffset = Number.NEGATIVE_INFINITY;
	for (const child of els) {
		const box = child.getBoundingClientRect();
		const offset = y - box.top - box.height / 2;
		if (offset < 0 && offset > closestOffset) {
			closestOffset = offset;
			closest = child;
		}
	}
	return closest;
}

export async function todoPanelEntry(mainElement) {
	document.documentElement.classList.add('todo');

	mainElement.innerHTML = `<div id="todo-root">
<div id="todo-header">
<div id="todo-spacer"></div>
<button id="todo-forefront" type="button" aria-label="Toggle forefront" title="Keep forefront">⊤</button>
<button id="todo-close" type="button" aria-label="Close">✖</button>
</div>
<button id="todo-add" type="button" aria-label="Add TODO">+TODO</button>
<div id="todo-body">
<div id="todo-list"></div>
</div>
<div id="todo-resize-handle" aria-hidden="true"></div>
</div>`;

	const currentWindow = getCurrentWindow();
	const listEl = document.getElementById('todo-list');
	const addButton = document.getElementById('todo-add');
	const forefrontButton = document.getElementById('todo-forefront');
	const closeButton = document.getElementById('todo-close');
	const resizeHandle = document.getElementById('todo-resize-handle');
	const todoHeader = document.getElementById('todo-header');
	const todoRoot = document.getElementById('todo-root');

	let cfg = null;
	try {
		cfg = await invoke('load_config', {});
	} catch {
		// cfg remains null
	}

	if (cfg) {
		document.documentElement.style.fontFamily = cfg.font;
		document.documentElement.style.fontSize = sizeToCssPx(cfg.size);
		document.documentElement.style.color = cfg.color;
	}

	const statuses = normalizeStatuses(cfg?.todoStatuses);
	const defaultStatus = statuses[0];

	/** @type {{ id: string, text: string, status: string, memo: string }[]} */
	let items = [];
	let forefront = cfg?.forefront ?? false;
	let saveDebouncerId = null;
	let ignoreSaveTodoWindowLocation = false;
	let todoWindowLocationLockId = null;
	/** @type {Set<string>} */
	const openMemoIds = new Set();

	try {
		const loaded = await invoke('todo_load');
		if (Array.isArray(loaded?.items)) {
			items = loaded.items.map((it) => ({
				id: String(it.id ?? newTodoId()),
				text: String(it.text ?? ''),
				status: String(it.status ?? defaultStatus),
				memo: String(it.memo ?? ''),
			}));
		}
		if (loaded?.forefront != null) {
			forefront = Boolean(loaded.forefront);
		}
	} catch {
		// keep defaults
	}

	const updateForefrontButton = () => {
		forefrontButton.textContent = forefront ? '⊥' : '⊤';
		forefrontButton.title = forefront ? 'Behind others' : 'Keep forefront';
	};
	updateForefrontButton();

	try {
		await currentWindow.setAlwaysOnTop(forefront);
	} catch {
		// ignore
	}

	const persistNow = async () => {
		try {
			await invoke('todo_save', {
				items,
				forefront,
			});
		} catch (error) {
			console.warn('[todo] Failed to save:', error);
		}
	};

	const scheduleSave = () => {
		if (saveDebouncerId != null) {
			clearTimeout(saveDebouncerId);
		}
		saveDebouncerId = setTimeout(() => {
			saveDebouncerId = null;
			persistNow();
		}, SAVE_DEBOUNCE_MS);
	};

	// Position/size via window-state (same flag pattern as sticky / main).
	const saveTodoWindowLocation = () => {
		if (ignoreSaveTodoWindowLocation) {
			return;
		}
		ignoreSaveTodoWindowLocation = true;
		todoWindowLocationLockId = setTimeout(async () => {
			todoWindowLocationLockId = null;
			if (todoRoot) {
				todoRoot.style.pointerEvents = 'none';
			}
			try {
				await invoke('save_window_state_exclusive');
			} catch {
				// ignore
			} finally {
				if (todoRoot) {
					todoRoot.style.pointerEvents = '';
				}
				ignoreSaveTodoWindowLocation = false;
			}
		}, 5000);
	};

	const readItemsFromDom = () => {
		const next = [];
		listEl.querySelectorAll('.todo-item').forEach((row) => {
			const id = row.dataset.id;
			if (!id) {
				return;
			}
			const textInput = row.querySelector('.todo-text');
			const memoInput = row.querySelector('.todo-memo');
			const statusBtn = row.querySelector('.todo-status');
			next.push({
				id,
				text: textInput?.value ?? '',
				status: statusBtn?.dataset.status ?? defaultStatus,
				memo: memoInput?.value ?? '',
			});
		});
		items = next;
	};

	const render = () => {
		if (items.length === 0) {
			listEl.innerHTML = '';
			return;
		}

		listEl.innerHTML = items
			.map((it) => {
				const memoOpen = openMemoIds.has(it.id);
				const memoClass = memoOpen ? ' is-memo-open' : '';
				const memoBtnClass = memoOpen ? ' is-on' : '';
				return `<div class="todo-item${memoClass}" data-id="${escapeHTML(it.id)}">
<div class="todo-item-row">
<button type="button" class="todo-item-btn todo-memo-toggle${memoBtnClass}" aria-label="Reorder or toggle memo" title="Drag to reorder · click for memo">☰</button>
<button type="button" class="todo-status" data-status="${escapeHTML(it.status)}" title="Cycle status">${escapeHTML(it.status)}</button>
<input class="todo-text" type="text" spellcheck="false" value="${escapeHTML(it.text)}" placeholder="TODO" />
<button type="button" class="todo-item-btn todo-delete" aria-label="Delete" title="Delete">${TRASH_ICON_SVG}</button>
</div>
<textarea class="todo-memo" spellcheck="false" rows="3" placeholder="Memo">${escapeHTML(it.memo)}</textarea>
</div>`;
			})
			.join('');
	};

	listEl.addEventListener('click', async (e) => {
		const target = e.target;
		// SVG inside the trash button is SVGElement, not HTMLElement.
		if (!(target instanceof Element)) {
			return;
		}

		const itemEl = target.closest('.todo-item');
		if (!itemEl) {
			return;
		}
		const id = itemEl.dataset.id;
		if (!id) {
			return;
		}

		// Memo toggle is handled by pointer handlers (click vs drag).
		if (target.closest('.todo-memo-toggle')) {
			return;
		}

		const statusBtn = target.closest('.todo-status');
		if (statusBtn) {
			const current = statusBtn.dataset.status ?? defaultStatus;
			const next = nextStatus(current, statuses);
			statusBtn.dataset.status = next;
			statusBtn.textContent = next;
			readItemsFromDom();
			scheduleSave();
			return;
		}

		if (target.closest('.todo-delete')) {
			const textInput = itemEl.querySelector('.todo-text');
			const memoInput = itemEl.querySelector('.todo-memo');
			const statusEl = itemEl.querySelector('.todo-status');
			const text = (textInput instanceof HTMLInputElement ? textInput.value : '').trim();
			const memo = (memoInput instanceof HTMLTextAreaElement ? memoInput.value : '').trim();
			const status = statusEl?.dataset.status ?? defaultStatus;
			const needsConfirm = text.length > 0 || memo.length > 0 || status !== defaultStatus;
			if (needsConfirm) {
				const lines = [`[${status}] ${text || '(empty)'}`];
				if (memo.length > 0) {
					lines.push(memo);
				}
				const label = lines.join('\n');
				let confirmed = false;
				try {
					confirmed = await ask(`Delete this TODO?\n\n${label}`, {
						title: 'mclocks',
						kind: 'warning',
					});
				} catch (error) {
					await openMessageDialog(`Failed to confirm: ${error}`, 'mclocks Error', 'error');
					return;
				}
				if (!confirmed) {
					return;
				}
			}
			openMemoIds.delete(id);
			readItemsFromDom();
			items = items.filter((it) => it.id !== id);
			render();
			scheduleSave();
		}
	});

	/** @type {{ itemEl: HTMLElement, id: string, startX: number, startY: number, pointerId: number, didDrag: boolean, handle: HTMLElement } | null} */
	let dragSession = null;

	const toggleMemoForId = (id) => {
		if (openMemoIds.has(id)) {
			openMemoIds.delete(id);
		} else {
			openMemoIds.add(id);
		}
		readItemsFromDom();
		render();
		const memo = listEl.querySelector(`.todo-item[data-id="${CSS.escape(id)}"] .todo-memo`);
		if (memo instanceof HTMLTextAreaElement && openMemoIds.has(id)) {
			memo.focus();
		}
	};

	const endDragSession = (e) => {
		if (!dragSession) {
			return;
		}
		const session = dragSession;
		dragSession = null;
		try {
			session.handle.releasePointerCapture(session.pointerId);
		} catch {
			// ignore
		}
		session.itemEl.classList.remove('is-dragging');
		document.documentElement.classList.remove('todo-reordering');
		if (session.didDrag) {
			readItemsFromDom();
			scheduleSave();
			return;
		}
		if (e.type === 'pointercancel') {
			return;
		}
		toggleMemoForId(session.id);
	};

	listEl.addEventListener('pointerdown', (e) => {
		if (e.button !== 0) {
			return;
		}
		const target = e.target;
		if (!(target instanceof Element)) {
			return;
		}
		const handle = target.closest('.todo-memo-toggle');
		if (!(handle instanceof HTMLElement)) {
			return;
		}
		const itemEl = handle.closest('.todo-item');
		if (!(itemEl instanceof HTMLElement)) {
			return;
		}
		const id = itemEl.dataset.id;
		if (!id) {
			return;
		}
		e.preventDefault();
		dragSession = {
			itemEl,
			id,
			startX: e.clientX,
			startY: e.clientY,
			pointerId: e.pointerId,
			didDrag: false,
			handle,
		};
		try {
			handle.setPointerCapture(e.pointerId);
		} catch {
			// ignore
		}
	});

	listEl.addEventListener('pointermove', (e) => {
		if (!dragSession || e.pointerId !== dragSession.pointerId) {
			return;
		}
		const dx = e.clientX - dragSession.startX;
		const dy = e.clientY - dragSession.startY;
		if (!dragSession.didDrag) {
			if (Math.hypot(dx, dy) < DRAG_THRESHOLD_PX) {
				return;
			}
			dragSession.didDrag = true;
			dragSession.itemEl.classList.add('is-dragging');
			document.documentElement.classList.add('todo-reordering');
		}
		e.preventDefault();
		const before = dragInsertBeforeElement(listEl, e.clientY, dragSession.itemEl);
		if (before) {
			listEl.insertBefore(dragSession.itemEl, before);
		} else {
			listEl.appendChild(dragSession.itemEl);
		}
	});

	listEl.addEventListener('pointerup', (e) => {
		if (!dragSession || e.pointerId !== dragSession.pointerId) {
			return;
		}
		endDragSession(e);
	});

	listEl.addEventListener('pointercancel', (e) => {
		if (!dragSession || e.pointerId !== dragSession.pointerId) {
			return;
		}
		endDragSession(e);
	});

	listEl.addEventListener('input', (e) => {
		const target = e.target;
		if (!(target instanceof HTMLElement)) {
			return;
		}
		if (target.classList.contains('todo-text') || target.classList.contains('todo-memo')) {
			readItemsFromDom();
			scheduleSave();
		}
	});

	addButton.addEventListener('click', () => {
		readItemsFromDom();
		const id = newTodoId();
		items.push({
			id,
			text: '',
			status: defaultStatus,
			memo: '',
		});
		render();
		scheduleSave();
		const textInput = listEl.querySelector(`.todo-item[data-id="${CSS.escape(id)}"] .todo-text`);
		if (textInput instanceof HTMLInputElement) {
			textInput.focus();
		}
	});

	forefrontButton.addEventListener('click', async () => {
		forefront = !forefront;
		updateForefrontButton();
		try {
			await currentWindow.setAlwaysOnTop(forefront);
		} catch (error) {
			await openMessageDialog(`Failed to toggle forefront: ${error}`, 'mclocks Error', 'error');
		}
		scheduleSave();
	});

	const closePanel = async () => {
		if (saveDebouncerId != null) {
			clearTimeout(saveDebouncerId);
			saveDebouncerId = null;
			readItemsFromDom();
			await persistNow();
		}
		if (todoWindowLocationLockId != null) {
			clearTimeout(todoWindowLocationLockId);
			todoWindowLocationLockId = null;
			ignoreSaveTodoWindowLocation = false;
			try {
				await invoke('save_window_state_exclusive');
			} catch {
				// ignore
			}
		}
		try {
			await invoke('todo_close_panel');
		} catch (error) {
			await openMessageDialog(`Failed to close TODO: ${error}`, 'mclocks Error', 'error');
		}
	};

	closeButton.addEventListener('click', () => {
		closePanel();
	});

	if (isMacOS() && todoHeader) {
		todoHeader.addEventListener('mousedown', async (event) => {
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

	if (resizeHandle) {
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
							Math.max(MIN_WIDTH, startSize.width + dx),
							Math.max(MIN_HEIGHT, startSize.height + dy),
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

	try {
		await currentWindow.onMoved(() => {
			saveTodoWindowLocation();
		});
	} catch {
		// ignore
	}

	try {
		await currentWindow.onResized(() => {
			saveTodoWindowLocation();
		});
	} catch {
		// ignore
	}

	window.addEventListener('keydown', (e) => {
		if (e.key === 'Escape') {
			e.preventDefault();
			closePanel();
		}
	});

	render();
}
