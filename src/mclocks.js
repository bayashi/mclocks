'use strict';

const elementClocks = document.getElementById('clocks');
const Clock = initClocks(window.mclocks.getClock(), elementClocks);

initStyles();
initScreen(elementClocks);
tick();

function tick() {
  tock();
  setTimeout(tick, 1000 - Date.now() % 1000);
}

function tock() {
  Clock.clocks.map(function(clock) {
    document.getElementById(clock.id).innerHTML
         = escapeHTML(window.mclocks.Moment(clock.timezone, Clock.localeDateTime, Clock.formatDateTime));
  });
}

const escapeTarget = {
  '&': '&amp;',
  "'": '&#x27;',
  '`': '&#x60;',
  '"': '&quot;',
  '<': '&lt;',
  '>': '&gt;',
};

function escapeHTML (string) {
  return (string || '').replace(/[&'`"<>]/g, function(match) {
    return escapeTarget[match]
  });
}

function initClocks(Clock, elementClocks) {
  let html = '';
  Clock.clocks.map(function(clock) {
    clock.id = clock.name.toLowerCase().replace(/[^a-z\d]/g, '-');
    html = html + clockBox(clock);
  });
  elementClocks.innerHTML = html;
  Clock.clocks.map(function(clock, index) {
    if (index !== Clock.clocks.length - 1) {
      document.getElementById(clock.id).style.paddingRight = '1.65em';
    }
  });
  return Clock;
}

function clockBox(clock) {
  return "<li>"
        + escapeHTML(clock.name)
        + " <span id='" + clock.id + "'></span>"
        + "</li>"
        ;
}

function initScreen(elementClocks) {
  tock();
  window.mclocks.fixSize(elementClocks.offsetWidth, elementClocks.offsetHeight);
}

function initStyles() {
  const AppStyle = document.getElementById('mclocks').style;
  AppStyle.color = Clock.fontColor;
  AppStyle.fontSize = Clock.fontSize + 'px';
  if (Clock.onlyText) {
    AppStyle.backgroundColor = 'rgba(0, 0, 0, 0)';
    AppStyle.border = 'none';
  } else {
    AppStyle.backgroundColor = Clock.bgColor;
    AppStyle.border = '1px solid ' + Clock.bgColor;
    AppStyle.borderRadius = Math.round(Clock.fontSize / 3) + 'px';
  }
}
