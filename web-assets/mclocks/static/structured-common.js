(function(){
	if (typeof window.mclocksSetupResizer === "function") {
		window.mclocksSetupResizer("mclocks-json-sidebar-width", "resizer", 200, 400, "--sidebar-width");
	}

	const pathCopyBtn = document.getElementById("path-copy-btn");
	const pathLabel = document.getElementById("main-header-path");
	if (pathCopyBtn && pathLabel) {
		pathCopyBtn.addEventListener("click", () => {
			navigator.clipboard.writeText(pathLabel.textContent || "");
			pathCopyBtn.textContent = "Copied!";
			pathCopyBtn.blur();
			setTimeout(() => {
				pathCopyBtn.textContent = "Copy";
				pathCopyBtn.blur();
			}, 2000);
		});
	}

	const cssEscape = (value) => {
		if (window.CSS && typeof window.CSS.escape === "function") {
			return window.CSS.escape(value);
		}
		return String(value).replaceAll("\\", "\\\\").replaceAll("\"", "\\\"");
	};

	const outlineItems = document.querySelectorAll("#outline-list li[data-path]");
	const outlineList = document.getElementById("outline-list");
	let activeOutlineItem = null;
	let activeJsonNodes = [];
	let activeJsonEntryKeys = [];

	const clearHover = () => {
		if (activeOutlineItem) {
			activeOutlineItem.classList.remove("is-hovered");
			activeOutlineItem = null;
		}
		activeJsonNodes.forEach((node) => node.classList.remove("is-hovered"));
		activeJsonNodes = [];
		activeJsonEntryKeys.forEach((node) => node.classList.remove("is-hovered"));
		activeJsonEntryKeys = [];
	};

	outlineItems.forEach((item) => {
		item.addEventListener("mouseenter", () => {
			clearHover();
			const path = item.getAttribute("data-path");
			if (!path) {
				return;
			}
			const nodes = document.querySelectorAll(`#json-view [data-path="${cssEscape(path)}"]`);
			const entryKeys = document.querySelectorAll(`#json-view [data-key-path="${cssEscape(path)}"]`);
			if (nodes.length === 0 && entryKeys.length === 0) {
				return;
			}
			activeOutlineItem = item;
			activeOutlineItem.classList.add("is-hovered");
			activeJsonNodes = Array.from(nodes);
			activeJsonNodes.forEach((node) => node.classList.add("is-hovered"));
			activeJsonEntryKeys = Array.from(entryKeys);
			activeJsonEntryKeys.forEach((node) => node.classList.add("is-hovered"));
		});
		item.addEventListener("mouseleave", clearHover);
	});

	if (outlineList) {
		outlineList.addEventListener("mouseleave", clearHover);
	}
	document.addEventListener("mouseleave", clearHover);
	window.addEventListener("blur", clearHover);
	document.addEventListener("visibilitychange", () => {
		if (document.hidden) {
			clearHover();
		}
	});
})();
