const DEBUG = true;
// const DEBUG = false;

const CURRENT_LANGUAGE = van.state("en");
const UI_STRINGS = getLocalization();

function localizeUi(key, nullIfMissing = false) {
  if (!(key in UI_STRINGS)) {
     console.log("Missing UI string=" + key);
     return nullIfMissing ? null : key;
  }
  if (!(CURRENT_LANGUAGE.val in UI_STRINGS[key])) {
     console.log("Missing UI string=" + key + " for language=" + CURRENT_LANGUAGE.val);
     return nullIfMissing ? null : UI_STRINGS[key]['en']
  }
  return () => UI_STRINGS[key][CURRENT_LANGUAGE.val];
}

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