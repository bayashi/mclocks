const escapeTarget = {
  '&': '&amp;',
  "'": '&#x27;',
  '`': '&#x60;',
  '"': '&quot;',
  '<': '&lt;',
  '>': '&gt;',
};

export function escapeHTML(str) {
  return (str || '').replace(/[&'`"<>]/g, function (match) {
    return escapeTarget[match]
  });
}

export function pad(n) {
  if (n >= 0 && n < 10) {
    return "0" + n
  }

  return n
}
