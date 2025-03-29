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
