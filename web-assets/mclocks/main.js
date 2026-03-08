(function(){
	window.mclocksSetupResizer = (storageKey, resizerId, min, max, varName) => {
		const raw = localStorage.getItem(storageKey);
		if (raw !== null) {
			const n = Number(raw);
			if (Number.isFinite(n)) {
				const clamped = Math.max(min, Math.min(max, n));
				document.documentElement.style.setProperty(varName, `${clamped}px`);
			}
		}

		let dragging = false;
		const resizer = document.getElementById(resizerId);
		if (!resizer) {
			return;
		}

		const applyWidth = (x) => {
			const clamped = Math.max(min, Math.min(max, x));
			document.documentElement.style.setProperty(varName, `${clamped}px`);
			localStorage.setItem(storageKey, String(clamped));
		};

		resizer.addEventListener("mousedown", (e) => {
			e.preventDefault();
			dragging = true;
			resizer.classList.add("is-dragging");
		});

		window.addEventListener("mousemove", (e) => {
			if (!dragging) {
				return;
			}
			applyWidth(e.clientX);
		});

		window.addEventListener("mouseup", () => {
			if (!dragging) {
				return;
			}
			dragging = false;
			resizer.classList.remove("is-dragging");
		});
	};
})();
