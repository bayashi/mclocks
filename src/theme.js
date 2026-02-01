export function applyThemeCssVars(config) {
  if (!config) {
    return;
  }

  const root = document.documentElement;
  if (!root) {
    return;
  }

  if (config.font) {
    root.style.setProperty('--mclocks-font-family', config.font);
  }

  if (config.color) {
    root.style.setProperty('--mclocks-color', config.color);
  }

  const size = config.size;
  if (size !== undefined && size !== null) {
    const isNumericSize = typeof size === "number" || /^[\d.]+$/.test(size);
    const sizeUnit = isNumericSize ? "px" : "";
    root.style.setProperty('--mclocks-font-size', `${size}${sizeUnit}`);
  }
}

