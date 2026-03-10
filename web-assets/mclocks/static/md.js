(function(){
	if (typeof window.mclocksSetupResizer === "function") {
		window.mclocksSetupResizer("mclocks-md-toc-width", "toc-resizer", 200, 400, "--toc-width");
	}

	if (window.hljs && typeof window.hljs.highlightElement === "function") {
		document.querySelectorAll("pre code").forEach((code) => window.hljs.highlightElement(code));
	}

	document.querySelectorAll("pre code").forEach((code) => {
		const pre = code.parentElement;
		if (pre.nextElementSibling?.classList.contains("copy-btn")) {
			return;
		}

		const btn = document.createElement("button");
		btn.textContent = "Copy";
		btn.className = "copy-btn";
		btn.onclick = () => {
			navigator.clipboard.writeText(code.textContent || "");
			btn.textContent = "Copied!";
			setTimeout(() => {
				btn.textContent = "Copy";
			}, 2000);
		};
		pre.parentNode.insertBefore(btn, pre.nextSibling);
	});

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

	const tocList = document.getElementById("toc-list");
	if (!tocList) {
		return;
	}

	const links = tocList.querySelectorAll("a");
	const headings = document.querySelectorAll("#content h1, #content h2, #content h3, #content h4");
	const observer = new IntersectionObserver(
		(entries) => {
			entries.forEach((entry) => {
				if (entry.isIntersecting) {
					const id = entry.target.id;
					links.forEach((a) => a.classList.toggle("active", a.getAttribute("href") === `#${id}`));
				}
			});
		},
		{ rootMargin: "0px 0px -80% 0px", threshold: 0 }
	);
	headings.forEach((h) => observer.observe(h));
})();
