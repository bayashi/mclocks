'use strict';

const Week = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
const Clock = initClocks(window.mclocks.getClock());

const AppStyle = document.getElementById('mclocks').style;
AppStyle.color = Clock.fontColor;
AppStyle.fontSize = Clock.fontSize + 'px';
AppStyle.backgroundColor = Clock.bgColor;

adjustWindowSize();

tick();

function tick() {
  document.getElementById('clocks').innerHTML = tock();
  setTimeout(tick, 1000 - Date.now() % 1000);
}

function tock() {
  let html = '';
  Clock.clocks.map(function(clock) {
    html = html + buildDateTimeHTML(clock);
  });

  return html;
}

function buildDateTimeHTML(clock) {
  let year, month, date, hour, minute, second, msecond, day;
  [year, month, date, hour, minute, second, msecond, day] = window.mclocks.Moment(clock.timezone);
  return "<li id='" + clock.nameForAttr + "'>"
        + clock.nameForView
        + " " + d2(month + 1) + Clock.dateDelimiter + d2(date)
        + " " + Week[day]
        + " " + d2(hour) + ":" + d2(minute)
        + (Clock.showSeconds ? ":" + d2(second) : '')
        + "</li>"
        ;
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

function d2(number) {
  return ("0" + (number)).slice(-2);
}

function adjustWindowSize() {
  const vclocks = document.getElementById('vclocks');
  vclocks.innerHTML = tock();
  window.mclocks.fixWidth(vclocks.offsetWidth, vclocks.offsetHeight);
  document.getElementById('mclocks').removeChild(vclocks);
}

function initClocks(Clock) {
  Clock.clocks.map(function(clock) {
    clock.nameForAttr = clock.name.toLowerCase().replace(/[^a-z\d]/g, '-');
    clock.nameForView = escapeHTML(clock.name);
  });

  return Clock;
}
