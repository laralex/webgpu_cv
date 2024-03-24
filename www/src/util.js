// utility functions
function Util () {};

Util.addClass = function(el, className) {
  var classList = className.split(' ');
  el.classList.add(classList[0]);
  if (classList.length > 1) Util.addClass(el, classList.slice(1).join(' '));
};

Util.removeClass = function(el, className) {
  var classList = className.split(' ');
  el.classList.remove(classList[0]);
  if (classList.length > 1) Util.removeClass(el, classList.slice(1).join(' '));
};

Util.toggleClass = function(el, className, bool) {
  if(bool) Util.addClass(el, className);
  else Util.removeClass(el, className);
};

Util.moveFocus = function (element) {
  if( !element ) element = document.getElementsByTagName('body')[0];
  element.focus();
  if (document.activeElement !== element) {
    element.setAttribute('tabindex','-1');
    element.focus();
  }
};

Util.computeScrollSpeed = function(event, settings){
  settings = settings || {};

  var timer, delta,
      delay = settings.delay || 50; // in "ms" (higher means lower fidelity )

  function clear() {
    delta = 0;
  }

  clear();

  return function(event){
    delta = event.deltaY;
    clearTimeout(timer);
    timer = setTimeout(clear, delay);
    return delta;
  };
};

Util.setCookie = function(key, value) {
  let updatedCookie = encodeURIComponent(key) + "=" + encodeURIComponent(value);
  document.cookie = updatedCookie + ";max-age=3600;samesite=lax";
}

Util.getCookie = function(key) {
  let matches = document.cookie.match(new RegExp(
    "(?:^|; )" + key.replace(/([\.$?*|{}\(\)\[\]\\\/\+^])/g, '\\$1') + "=([^;]*)"
  ));
  return matches ? decodeURIComponent(matches[1]) : undefined;
}

Util.deleteCookie = function(key) {
  document.cookie = encodeURIComponent(key)+"=;max-age=-1";
}

Util.monthDiff = function(d1, d2){
  var months;
  months = (d2.getFullYear() - d1.getFullYear()) * 12;
  months -= d1.getMonth();
  months += d2.getMonth();
  const monthDiff = months <= 0 ? 0 : months;
  console.log("%%%%", d1, d2);
  return monthDiff;
}

Util.yearMonthDiff = function(monthDiff) {
  const yearDiff = Math.floor(monthDiff / 12);
  return {
    monthDiff: monthDiff,
    yearDiff: yearDiff,
    monthRemainder: monthDiff - yearDiff * 12,
  }
}

Util.formatYearsMonths = function({years, months, yearsFullWord = true, monthsFullWord = true}) {
  const getLocalized = (key) => localizeString(key)().text
  let monthStr = "";
  if (months > 0) {
    if (!monthsFullWord) {
      monthStr = months + " " + getLocalized("month_short")
    } else if (months == 1) {
      monthStr = months + " " + getLocalized("month_full")
    } else if (months < 5) {
      monthStr = months + " " + getLocalized("months_full")
    } else {
      monthStr = months + " " + getLocalized("months_many_full")
    }
  }
  
  let yearStr = "";
  if (years > 0) {
    if (!yearsFullWord) {
      yearStr = years + " " + getLocalized("year_short")
    } else if (years == 1) {
      yearStr = years + " " + getLocalized("year_full")
    } else if (years < 5) {
      yearStr = years + " " + getLocalized("years_full")
    } else {
      yearStr = years + " " + getLocalized("years_many_full")
    }
  }
  return {yearStr: yearStr, monthStr: monthStr}
}

Util.formatDateDiff = function(d1, d2, {yearsFullWord, monthsFullWord} = { yearsFullWord: true, monthsFullWord: true }) {
  const diff = Util.yearMonthDiff(Util.monthDiff(d1, d2));
  return Util.formatYearsMonths({
    years: diff.yearDiff,
    months: diff.monthRemainder,
    yearsFullWord: yearsFullWord,
    monthsFullWord: monthsFullWord,
  });
}

Util.getYearsSpan = function(d1, d2) {
  const yearsSpan = []
  for (var i = d1.getFullYear(); i <= d2.getFullYear(); ++i) {
    yearsSpan.push(i.toString());
  }
  return yearsSpan;
}

Util.remToPx = function(rem) {
  const remToPx = parseFloat(getComputedStyle(document.documentElement).fontSize);
  return rem * remToPx;
}

Util.pxToRem = function(px) {
  const pxToRem = 1.0 / parseFloat(getComputedStyle(document.documentElement).fontSize);
  return px * pxToRem;
}