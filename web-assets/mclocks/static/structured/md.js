(async function(){
	const normalizeOpenExternalLinkInNewTab = (value) => value !== "false";

	const isExternalHref = (href) => {
		try {
			const resolved = new URL(href, window.location.href);
			if (resolved.protocol === "http:" || resolved.protocol === "https:") {
				return resolved.origin !== window.location.origin;
			}
			return true;
		} catch (_) {
			return false;
		}
	};

	const applyContentLinkTargets = (openExternalLinkInNewTab) => {
		document.querySelectorAll("#content a[href]").forEach((anchor) => {
			const href = (anchor.getAttribute("href") || "").trim();
			if (!href || href.startsWith("#") || !openExternalLinkInNewTab || !isExternalHref(href)) {
				anchor.removeAttribute("target");
				anchor.removeAttribute("rel");
				return;
			}
			anchor.setAttribute("target", "_blank");
			anchor.setAttribute("rel", "noopener noreferrer");
		});
	};

	const openExternalLinkInNewTab = normalizeOpenExternalLinkInNewTab(document.body?.dataset?.openExternalLinkInNewTab);
	applyContentLinkTargets(openExternalLinkInNewTab);

	if (typeof window.mclocksSetupResizer === "function") {
		window.mclocksSetupResizer("mclocks-md-toc-width", "toc-resizer", 200, 400, "--toc-width");
	}

	const renderMermaidBlocks = async () => {
		const codeBlocks = Array.from(document.querySelectorAll("pre code.language-mermaid"));
		if (!codeBlocks.length || !window.mermaid || typeof window.mermaid.render !== "function") {
			return;
		}
		if (typeof window.mermaid.initialize === "function") {
			window.mermaid.initialize({ startOnLoad: false, securityLevel: "strict" });
		}
		for (let index = 0; index < codeBlocks.length; index += 1) {
			const code = codeBlocks[index];
			const pre = code.parentElement;
			if (!pre) {
				continue;
			}
			const source = code.textContent || "";
			const container = document.createElement("div");
			container.className = "mermaid-diagram";
			try {
				const id = `mclocks-mermaid-${index}`;
				const rendered = await window.mermaid.render(id, source);
				container.innerHTML = rendered.svg;
				pre.replaceWith(container);
			} catch (_) {
				// Keep the original code block if rendering fails.
			}
		}
	};

	await renderMermaidBlocks();

	if (window.hljs && typeof window.hljs.highlightElement === "function") {
		document.querySelectorAll("pre code").forEach((code) => {
			if ((code.className || "").split(/\s+/).includes("language-mermaid")) {
				return;
			}
			window.hljs.highlightElement(code);
		});
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

	const summaryList = document.getElementById("summary-list");
	if (summaryList) {
		const pad2 = (n) => String(n).padStart(2, "0");
		const toLocalTime = (value) => {
			const n = Number(value);
			if (!Number.isFinite(n)) {
				return null;
			}
			const d = new Date(n);
			const y = d.getFullYear();
			const mo = pad2(d.getMonth() + 1);
			const da = pad2(d.getDate());
			const h = pad2(d.getHours());
			const mi = pad2(d.getMinutes());
			const s = pad2(d.getSeconds());
			return `${y}-${mo}-${da} ${h}:${mi}:${s}`;
		};
		summaryList.querySelectorAll("li").forEach((item) => {
			const label = item.querySelector(".label");
			const value = item.querySelector(".value");
			if (!label || !value) {
				return;
			}
			if (label.textContent?.trim() !== "Last Mod") {
				return;
			}
			const formatted = toLocalTime(value.textContent?.trim());
			if (formatted) {
				value.textContent = formatted;
			}
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
