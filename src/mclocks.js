'use strict';

const Week = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
const Clock = window.mclocks.getClock();

const AppStyle = document.getElementById('mclocks').style;
AppStyle.color = Clock.fontColor;
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
    html = html + buildDateTimeHTML(clock.name, clock.timezone);
  });

  return html;
}

function buildDateTimeHTML(name, timezone) {
  let year, month, date, hour, minute, second, msecond, day;
  [year, month, date, hour, minute, second, msecond, day] = window.mclocks.Moment(timezone);
  return "<li id='" + name.toLowerCase().replace(/[^a-z\d]/g, '-') + "'>"
        + escapeHTML(name)
        + " " + d2(month + 1) + Clock.dateDelimiter + d2(date)
        + " " + Week[day]
        + " " + d2(hour) + ":" + d2(minute)
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
