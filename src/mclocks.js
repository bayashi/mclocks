'use strict';

const Week = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];

const elementClocks = document.getElementById('clocks');
const Clock = initClocks(window.mclocks.getClock());

const AppStyle = document.getElementById('mclocks').style;
AppStyle.color = Clock.fontColor;
AppStyle.fontSize = Clock.fontSize + 'px';
AppStyle.backgroundColor = Clock.bgColor;

tock();

window.mclocks.fixWidth(elementClocks.offsetWidth, elementClocks.offsetHeight);

tick();

function tick() {
  tock();
  setTimeout(tick, 1000 - Date.now() % 1000);
}

function tock() {
  Clock.clocks.map(function(clock) {
    document.getElementById(clock.id).innerHTML = buildDateTimeHTML(clock);
  });
}

function buildDateTimeHTML(clock) {
  let year, month, date, hour, minute, second, msecond, day;
  [year, month, date, hour, minute, second, msecond, day] = window.mclocks.Moment(clock.timezone);
  return " " + d2(month + 1) + Clock.dateDelimiter + d2(date)
        + " " + Week[day]
        + " " + d2(hour) + ":" + d2(minute)
        + (Clock.showSeconds ? ":" + d2(second) : '')
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

function initClocks(Clock) {
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
        + "<span id='" + clock.id + "'></span>"
        + "</li>"
        ;
}
