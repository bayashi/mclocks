import { LogicalPosition } from '@tauri-apps/api/window';

// CSS app-region / startDragging break when a panel body shows a scrollbar.
export function bindHeaderWindowDrag(headerEl, currentWindow) {
	if (!headerEl || !currentWindow) {
		return;
	}
	headerEl.addEventListener('mousedown', async (event) => {
		if (event.button !== 0) {
			return;
		}
		if (event.target instanceof Element && event.target.closest('button')) {
			return;
		}
		event.preventDefault();
		const startX = event.screenX;
		const startY = event.screenY;
		let originX;
		let originY;
		try {
			const [pos, scaleFactor] = await Promise.all([
				currentWindow.outerPosition(),
				currentWindow.scaleFactor(),
			]);
			const factor = scaleFactor || 1;
			originX = pos.x / factor;
			originY = pos.y / factor;
		} catch {
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
				void currentWindow.setPosition(
					new LogicalPosition(originX + (lastX - startX), originY + (lastY - startY)),
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
	});
}
