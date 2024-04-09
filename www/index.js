
const {div, button, i, label, img, svg, path, input, details, summary, p, li, a, option, select, span, ul, h1, h2, h3} = van.tags

const ADD_PARALLAX = true;
const CANVAS_ID = "main-canvas";

const SIDEBAR_WIDTH_OVERRIDE_PX = van.state(0);
const SIDEBAR_WIDTH_FONT_PX = van.state(0);
const CURRENT_GRAPHICS_LEVEL = van.state(GraphicsLevel.Low);
const CURRENT_FPS_LIMIT = van.state(45);
const CURRENT_FONT_FAMILY = van.state("\"Share Tech\"");
const DEFAULT_MAIN_CHAPTER = "chapter_career";
const DEFAULT_SUB_CHAPTER = "career_huawei";
const CURRENT_CV_PAGE = [van.state(DEFAULT_MAIN_CHAPTER), van.state(DEFAULT_SUB_CHAPTER)];
const CV_PAGE_ORDER = {}
const IS_TUTORIAL_SHOWN = van.state(false);
const IS_INTRO_SHOWN = van.state(true);
const IS_FULLSCREEN = van.state(false);
let WASM_INSTANCE = undefined;

async function loadBuildData() {
   const response = await fetch('./build-data.json');
   const buildData = await response.json();
   Object.entries(buildData).forEach((kv) => {
      BUILD_DATA[kv[0]] = kv[1];
   });
}

export const BUILD_DATA = {
   'git-commit': "stub",
   'git-commit-date': "stub",
   'debug': false,
   'deploy-date': "stub"
};

// {
//    'git-commit': "stub",
//    'git-commit-date': "stub",
//    'debug': false,
//    'deploy-date': "stub"
// };

// import mywasm from 'my-wasm';
// import init, { wasm_loop, wasm_resize, wasm_startup, wasm_set_fps_limit, wasm_set_graphics_level } from '/wasm/index.js';
import init, { WasmInterface, DemoId, GraphicsLevel } from '/wasm/index.js';
import { UI_STRINGS, CURRENT_LANGUAGE, localizeString, localizeUi, localizeUiPostprocess } from '/modules/localization.js';
import { Util } from '/modules/util.js';
import { CvContent } from '/modules/cv.js';
import { CURRENT_DEMO_LOADING_PROGRESS, CURRENT_GRAPHICS_SWITCHING_PROGRESS } from '/modules/exports_to_wasm.js';

function dumpCvCookies() {
   Util.setCookie('mainChapter', CURRENT_CV_PAGE[0].val);
   Util.setCookie('subChapter', CURRENT_CV_PAGE[1].val);
}

function dumpUiCookies() {
   Util.setCookie('language', CURRENT_LANGUAGE.val);
   Util.setCookie('fontFamily', CURRENT_FONT_FAMILY.val);
   Util.setCookie('showIntro', IS_INTRO_SHOWN.val);
   Util.setCookie('sidebarWidth', SIDEBAR_WIDTH_OVERRIDE_PX.val);
}

function dumpDemoCookies() {
   Util.setCookie('graphicsLevel', CURRENT_GRAPHICS_LEVEL.val);
   Util.setCookie('fpsLimit', CURRENT_FPS_LIMIT.val);
}

function dumpCookies() {
   dumpCvCookies();
   dumpUiCookies()
   dumpDemoCookies();
}

function loadCookies() {
   const setState = function(key, state){
      const c = Util.getCookie(key);
      if (c !== undefined) {
         state.val = c;
      }
   }
   setState('language', CURRENT_LANGUAGE);
   setState('fontFamily', CURRENT_FONT_FAMILY);
   setState('fpsLimit', CURRENT_FPS_LIMIT);
   setState('graphicsLevel', CURRENT_GRAPHICS_LEVEL);
   setState('mainChapter', CURRENT_CV_PAGE[0]);
   setState('subChapter', CURRENT_CV_PAGE[1]);
   setState('showIntro', IS_INTRO_SHOWN);
   setState('sidebarWidth', SIDEBAR_WIDTH_OVERRIDE_PX);
}

function clearCookies() {
   Util.deleteCookie('language');
   Util.deleteCookie('fontFamily');
   Util.deleteCookie('fpsLimit');
   Util.deleteCookie('graphicsLevel');
   Util.deleteCookie('mainChapter');
   Util.deleteCookie('subChapter');
   Util.deleteCookie('showIntro');
   Util.deleteCookie('sidebarWidth');
}
   
   // van.derive(() => console.log('full', IS_FULLSCREEN.val));
function setFullScreen(elem, enableFullscreen) {
   // ## The below if statement seems to work better ## if ((document.fullScreenElement && document.fullScreenElement !== null) || (document.msfullscreenElement && document.msfullscreenElement !== null) || (!document.mozFullScreen && !document.webkitIsFullScreen)) {
   let enableFullscreen_ = enableFullscreen === undefined || enableFullscreen === true;
   let disableFullscreen_ = enableFullscreen === undefined || enableFullscreen === false;
   if (enableFullscreen_ && ((document.fullScreenElement !== undefined && document.fullScreenElement === null) || (document.msFullscreenElement !== undefined && document.msFullscreenElement === null) || (document.mozFullScreen !== undefined && !document.mozFullScreen) || (document.webkitIsFullScreen !== undefined && !document.webkitIsFullScreen))) {
         if (elem.requestFullScreen) {
            elem.requestFullScreen();
         } else if (elem.mozRequestFullScreen) {
            elem.mozRequestFullScreen();
         } else if (elem.webkitRequestFullScreen) {
            elem.webkitRequestFullscreen(Element.ALLOW_KEYBOARD_INPUT);
         } else if (elem.msRequestFullscreen) {
            elem.msRequestFullscreen();
         }
         IS_FULLSCREEN.val = true;
   } else if (disableFullscreen_) {
         if (document.cancelFullScreen) {
            document.cancelFullScreen();
         } else if (document.mozCancelFullScreen) {
            document.mozCancelFullScreen();
         } else if (document.webkitCancelFullScreen) {
            document.webkitCancelFullScreen();
         } else if (document.msExitFullscreen) {
            document.msExitFullscreen();
         }
         IS_FULLSCREEN.val = false;
   }
   // console.log('setFullScreen', IS_FULLSCREEN.val, enableFullscreen_, disableFullscreen_);
}

function FullscreenButton({extraClasses = "", isInteractive = true, height = "80"}) {
   return FullscreenButtonImpl(document.getElementById("canvas-wrapper"),
      extraClasses, isInteractive, height)
}

function FullscreenButtonImpl(fullscreenElement, extraClasses, isInteractive, height) {
   return () => 
      div({
         class: () => extraClasses + (isInteractive ? " btn " : "") + " canvas-button",
         onclick: () => {
            if (isInteractive) setFullScreen(fullscreenElement, !IS_FULLSCREEN.val)
         }
      },
      img({
         src: () => IS_FULLSCREEN.val ? "../assets/collapse-regular-240.png" : "../assets/expand-regular-240.png",
         height: height,
      })
   );
}

function HelpButton({height = "80"}) {
   return div({class: "help-button btn canvas-button", onclick: () => IS_TUTORIAL_SHOWN.val = true },
      img({ src:"../assets/help-circle-regular-240.png", height: height })
   );
   // return null;
}

function GeoLocation() {
   return div({class: "geo-location"},
      img({src: "../assets/map-icon.svg"}),
      localizeUi('geo_location')
   );
}

function FontPicker(currentFont) {
   const FONTS = [
      "\"Share Tech\"",
      "\"JetBrains Mono\"",
      "\"Segoe UI\"",
   ];
	const options = FONTS.map((fontFamily, index) =>
      option({ value: fontFamily, selected: () => fontFamily == currentFont.val}, fontFamily));
   const labelBefore = null;
   const labelAfter = span(localizeUi('font_family'));
   return () => div(
      { class: 'font-picker ' + "flex-row" },
      labelBefore,
      select({
         class: 'interactive btn',
         value: currentFont,
         oninput: e => currentFont.val = e.target.value,
      }, options,),
      labelAfter,
   );
}

function FpsLimitPicker(currentLimit) {
   const FPS_LIMITS = [
      10, 30, 45, 60, 90, 144
   ];
	const options = FPS_LIMITS.map((fpsLimit, index) =>
      option({ value: fpsLimit, selected: () => fpsLimit == currentLimit.val}, () => `${fpsLimit} FPS`));
   const labelBefore = null;
   const labelAfter = span(localizeUi('fps_limit'));
   van.derive(() => {
      var newLimit = currentLimit.val;
      if (WASM_INSTANCE) {
         console.log("new fps", newLimit);
         WASM_INSTANCE.wasm_set_fps_limit(newLimit);
      }
   })
   return () => div(
      { class: 'fps-limit-picker ' + "flex-row" },
      labelBefore,
      select({
         class: 'interactive btn',
         value: currentLimit,
         oninput: e => currentLimit.val = e.target.value,
      }, options,),
      labelAfter,
   );
}

function LanguagePicker(currentLanguage, currentFont, isVertical, tooltipLanguage=undefined, tooltipLabelId='ui_language') {
   const LANGUAGES = {
      en: {labelId: 'english_en', icon: "../assets/flag_GB.png", emoji: "🇬🇧", font: "\"Share Tech\""},
      ru: {labelId: 'russian_en', icon: "../assets/flag_RU.png", emoji: "🇷🇺", font: "\"JetBrains Mono\""},
      kr: {labelId: 'korean_en', icon: "../assets/flag_RU.png", emoji: "🇰🇷", font: "\"Share Tech\""},
   }
   function localizePage(language)
   {
      if (! (Object.keys(LANGUAGES).includes(language))) {
         return;
      }
      console.log("Set language=" + language);
      let lang = ':lang(' + language + ')';
		let hide = '[lang]:not(' + lang + ')';
		// document.querySelectorAll(hide).forEach(function (node) {
		// 	node.style.display = 'none';
		// });
		let show = '[lang]' + lang;
		// document.querySelectorAll(show).forEach(function (node) {
		// 	node.style.display = 'unset';
		// });
   }
   if (!tooltipLanguage) {
      tooltipLanguage = currentLanguage;
   }
	van.derive(() => localizePage(currentLanguage.val));
	const options = Object.entries(LANGUAGES).map(([language, meta]) =>
      option({ value: language, selected: () => language == currentLanguage.val},
         () => (Util.isFlagEmojiSupported() ? meta.emoji + " " : "") + UI_STRINGS[meta.labelId][tooltipLanguage.val]));
   const labelBefore = isVertical ? span(() => UI_STRINGS[tooltipLabelId][tooltipLanguage.val]) : null;
   const labelAfter = !isVertical ? span(() => UI_STRINGS[tooltipLabelId][tooltipLanguage.val]) : null;
   return () => div(
      { class: 'language-picker ' + (isVertical ? "flex-column" : "flex-row") },
      labelBefore,
      select({
         class: 'interactive btn',
         value: currentLanguage,
         oninput: e => {
            currentLanguage.val = e.target.value;
            currentFont.val = LANGUAGES[currentLanguage.val].font;
         },
      }, options,),
      labelAfter,
   );
}

function GraphicsLevelPicker(currentGraphicsLevel, isVertical) {
   const meta = {};
   meta[GraphicsLevel.Low] = {labelId: 'graphics_low', emoji: "✰✰✰"};
   meta[GraphicsLevel.Medium] = {labelId: 'graphics_medium', emoji: "★✰✰"};
   meta[GraphicsLevel.High] = {labelId: 'graphics_high', emoji: "★★✰"};
   meta[GraphicsLevel.Ultra] = {labelId: 'graphics_ultra', emoji: "★★★"};
   const options = Object.entries(meta).map(([level, meta]) =>
      option({ value: level, selected: () => level == currentGraphicsLevel.val},
         () => localizeString(meta.labelId)().text + " " +  meta.emoji));
   van.derive(() => {
      console.log("Set graphics_level="+currentGraphicsLevel.val)
      if (WASM_INSTANCE) {
         WASM_INSTANCE.wasm_set_graphics_level(currentGraphicsLevel.val)
      }
   });
   const labelBefore = isVertical ? span(localizeUi('graphics_levels')) : null;
   const labelAfter = !isVertical ? span(localizeUi('graphics_levels')) : null;
   return div(
      { class: 'graphics-picker ' + (isVertical ? "flex-column" : "flex-row") },
      labelBefore,
      select({
         class: 'interactive btn',
         oninput: e => currentGraphicsLevel.val = e.target.value,
         value: currentGraphicsLevel,
         disabled: () => CURRENT_DEMO_LOADING_PROGRESS.val != null || CURRENT_GRAPHICS_SWITCHING_PROGRESS.val != null,
      }, options,),
      labelAfter,
   );
}

function ResumePdfLink() {
   return button({class:"grid-item contact interactive btn wiggle-animation", role:"button"},
      // bxs-download
      a({ class: "font-normalsize", href: localizeUi("pdf_cv_href"), target: "_blank"},
      img({ src: "./third_party/boxicons-2.1.4/svg/solid/bxs-file-pdf.svg", style: "filter: var(--filter-gmail)"}),
         // () => localizeString("cv")().text + " " + localizeString("pdf")().text,
         // label({style: "display:block;"}, ),
         // label({style: "display:block;"}, ),
      ));
}

function configureResizingBorder() {
   const resizeElement = document.getElementById("resize-border");
   const sidebar = document.getElementById("sidebar");
   let m_pos;

   function pauseEvent(e){
      if(e.stopPropagation) e.stopPropagation();
      if(e.preventDefault) e.preventDefault();
      e.cancelBubble=true;
      e.returnValue=false;
      return false;
   }

   function resize(e){
      const dx = e.x - m_pos;
      m_pos = e.x;
      const newSidebarPx = parseInt(getComputedStyle(sidebar, '').flexBasis) + dx;
      // const newSidebarRem = Util.pxToRem(newSidebarPx);
      SIDEBAR_WIDTH_OVERRIDE_PX.val = newSidebarPx;
      // shouldHide.val = true;
      // console.log("resize", getComputedStyle(document.documentElement).fontSize);
      return pauseEvent(e);
   }

   resizeElement.addEventListener("mousedown", function(e){
      console.log("resize_mousedown")
      if (e.offsetX >= 0) {
         m_pos = e.x;
         document.addEventListener("mousemove", resize, false);
      }
   }, false);

   document.addEventListener("mouseup", function(){
      document.removeEventListener("mousemove", resize, false);
   }, false);
}

function ResizeTooltip({timeoutMillisec, onclose = () => {}}) {
   const shouldHide = van.state(false);
   setTimeout(() => {
      shouldHide.val = true;
      onclose();
   }, timeoutMillisec);

   return () => BUILD_DATA.debug || shouldHide.val ? null : div({
      class: "resize-tooltip bubble shadow left zmax",
      onclick: (e) => shouldHide.val = true, onclose: onclose
   }, localizeUi("resize_tooltip"));
}

function RepositoryLink({width}) {
   return button({
         class:"btn-block interactive btn font-large",
         role:"button",
         style: () => `width:${width}; margin: 2px 0 0 0; text-align: center;`
      },
      a({class: "flex-row flex-tight", href: "https://github.com/laralex/my_web_cv", target: "_blank"},
         img({ src: "./third_party/boxicons-2.1.4/svg/logos/bxl-github.svg", style: "filter: var(--color-github)"}),
         label(localizeUi("web_cv_github")),
      ));
}

function ClearCookiesButton({width}) {
   return button({
         class:"btn-block interactive btn font-large flex-row flex-tight",
         role:"button",
         style: () => `width:${width}; margin: 2px 0 0 0; text-align: center;`,
         onclick: () => {
            clearCookies();
            location.reload();
         },
      },
      img({ src: "./third_party/boxicons-2.1.4/svg/regular/bx-refresh.svg", style: "filter: var(--color-github)"}),
      label(localizeUi("clear_cookies_button")),
   );
}

function PersonalCard() {
   return div({class: "profileinfo"},
      h1({class: "font-LARGE bold"}, localizeUi("name_surname")),
      h3(localizeUi("job_title")),
      div({class: "specialization"}, () => {
         const cg = localizeUi("specialty_computer_graphics")();
         const deepLearning = localizeUi("specialty_deep_learning", true)();
         return deepLearning != null ? ul(li(cg), li(deepLearning)) : cg;
      }),
      div({class: "techlist"},
      (["C++", "Python", "OpenGL", /*"WebGL",*/ "Android"]
         .map(text => div({class: "badge"}, text))),
      ),
      GeoLocation(),
   );
}
function MoreSkillsButton() {
   const isExpanded = van.state(false);
   return div({class: "badgescard"},
      button({
         class: "interactive btn font-Large expand-button",
         onclick: e => isExpanded.val = !isExpanded.val,
      }, img({height: "15", src: () => isExpanded.val ? "./third_party/boxicons-2.1.4/svg/solid/bxs-up-arrow.svg" : "./third_party/boxicons-2.1.4/svg/solid/bxs-down-arrow.svg"}), i(" "), localizeUi("skills_title")),
      div({
         class: "inside font-large",
         style: () => isExpanded.val ? "" : "display: none;",
         onclick: e => isExpanded.val = !isExpanded.val
       }, ul(
            li(span("Rust, C#, Java, JavaScript")),
            li(span("PyTorch, Docker, Qualcomm\xa0SNPE, Unity, ARCore, Linux, LaTeX")),
            li(span(localizeUiPostprocess("skills_languages_1", Util.substituteFlagEmoji))),
         ),
      ),
   );
}

function IntroPopup({onclose}) {
   const closed = van.state(false);
   const needAnimation = van.state(true);
   let needOnClose = true;
   van.derive(() => {if (closed.val && onclose && needOnClose) {
      onclose();
      needOnClose = false;
   }});
   return () => closed.val ? null : div({class: "popup font-large retro-box checkerboard-background"},
      div({class: "message font-Large"}, LanguagePicker(CURRENT_LANGUAGE, CURRENT_FONT_FAMILY, /* vertical */ false, undefined, 'ui_language_intro')),
      div({
         class: () => (needAnimation.val ? " animated-appear " : "") + " flex-column",
         onanimationend: () => needAnimation.val = false, // to prevent animation repeat when language switched
      },
      span({class: "message bold font-LARGE"}, localizeUi("intro_hi")),
      span({class: "message"}, localizeUi("intro_enjoy_resume")),
      span({class: "message"}, localizeUi("intro_using")), // ""
      div({class: "flex-row wide"},
         div({class: "message flex-column", style: "margin-right: 1.5em;"},
            span(a({href: "https://www.rust-lang.org/", target: "_blank"}, "Rust"), " + ", a({href: "https://www.khronos.org/webgl/", target: "_blank"}, "WebGL2")),
            span(localizeUi("intro_3d")),
            div({class: "flex-row"},
               a({href: "https://www.rust-lang.org/", target: "_blank"},
                  img({src: "../assets/rust-plain.svg", height:"80"}, "Rust")),
               a({href: "https://www.khronos.org/webgl/", target: "_blank"},
                  img({src: "../assets/WebGL_Logo.svg", height:"70", style: "padding:7px;"}, "WebGL 2"))
            ),
         ),
         div({class: "message flex-column", style: "margin-left: 1.5em;"},
            span(a({href: "https://en.wikipedia.org/wiki/JavaScript", target: "_blank"}, "JavaScript"), " + ", a({href: "https://vanjs.org/", target: "_blank"}, "VanJS")),
            span(localizeUi("intro_frontend")),
            div({class: "flex-row"},
               a({href: "https://en.wikipedia.org/wiki/JavaScript", target: "_blank"},
                  img({src: "../assets/javascript-original.svg", height:"80", style: "padding:3px;"}, "JavaScript")),
               a({href: "https://vanjs.org/", target: "_blank"},
                  img({src: "../assets/vanjs.svg", height:"80", style: "padding:3px;"}, "VanJS")),
            ),
            ),
         ),
      ),
      div({class: "controls"},
         button({class: "btn popup-btn font-large", onclick: (e) => closed.val = true }, localizeUi("intro_close")),
      ),
   )
}

function ControlsPopup({onclose}) {
   const closed = van.state(false);
   return () => closed.val ? null :div({class: "popup font-large retro-box checkerboard-background zmax"},
      div({class: "flex-column"},
         div({class: "flex-row"},
            div({class: "message flex-column flex-center ", style: "margin: 1em;"},
               img({src: "../assets/mouse-wheel-up-down.svg", height:"200"}),
               span({class: "message", style: "width: 10rem;"}, localizeUi("controls_mouse_wheel")),
               ),
            div({class: "message flex-column flex-center", style: "margin: 1em;"},
               img({src: "../assets/mouse-drag.svg", height:"200"}),
               span({class: "message", style: "width: 10rem;"}, localizeUi("controls_mouse_move")),
            )
         ),
         div({class: "flex-row"},
            div({class: "message flex-column flex-center ", style: "margin: 1em;"},
               div({class: "flex-row", style: "gap: 1rem;"},
                  img({src: "../assets/f11.png", height:"70"}),
                  FullscreenButton({isInteractive: false, height: "70"}),
               ),
               span({class: "message"}, localizeUi("controls_fullscreen_key")),
            ),
         ),
      ),
      div({class: "controls"},
         button({class: "btn popup-btn font-large", onclick: (e) => { 
            closed.val = true;
            if (onclose !== undefined) { onclose(); }
         }}, localizeUi("controls_close")))
   );
}

function LoadingScreen() {
   return () => {
      const progress = CURRENT_GRAPHICS_SWITCHING_PROGRESS.val != null
         ? CURRENT_GRAPHICS_SWITCHING_PROGRESS.val
         : CURRENT_DEMO_LOADING_PROGRESS.val;
      return progress === null ? div() : div({
            class: "loading-screen " + (progress !== null ? "" : " hide "),
            style: "background:rgb(0,0,0,"+(0.3 + 0.7 * progress)+");",
         },
         span(Math.trunc(Math.floor(progress*20)*5) + '%'),
         div({class: "bubble bar-container"},
            div({class: "bar-cutout", style: () => 'width:' + progress*100.0 + '%'},
               div({class: "bubble bar-progress"})
            ),
         )
      );
   }
}

function getFontFamilies(elements){
   let usedFonts = [];
   elements.forEach(function(el,i){
      let nodeType = el.nodeName.toLowerCase();
      let fontFamily = getComputedStyle(el).fontFamily;
      let familyArr = fontFamily.split(',');
      console.log('FONT EL', fontFamily);
     let fontApplied = false;
     let renderedFont = '';
     for(let i=0; i<familyArr.length && !fontApplied; i++){
       let currentFamily = familyArr[i];
       fontApplied = document.fonts.check(`12px ${currentFamily}`);
       if(fontApplied){
         //font is loaded - return family name
         renderedFont = currentFamily;
       }
     }
     usedFonts.push({ type:nodeType, font: renderedFont});
   })
   return usedFonts;
 }

 function getDefaultFonts() {
   var iframe = document.createElement('iframe');
   var html = '<html><body>';
   var fonts;
   document.body.appendChild(iframe);
   iframe.contentWindow.document.open();
   iframe.contentWindow.document.write(html);
   var subele = iframe.contentWindow.document.createElement(ele.tagName);
   iframe.contentWindow.document.body.appendChild(subele);
   fonts = getComputedStyle(subele)['font-family'];
   document.body.removeChild(iframe);
   return fonts;
}

function configureFromFont(fontFamily = null, currentLanguage = null) {
   document.body.style.zoom = "100%";
   let actualFontFamily = fontFamily;
   // if (actualFontFamily == null) {
   //    //actualFontFamily = getFontFamilies([para])[0].font;
   //    let familyArr = getDefaultFonts().split(',');
   //    for(let i=0; i<familyArr.length && !fontApplied; i++){
   //       let currentFamily = familyArr[i];
   //       fontApplied = document.fonts.check(`12px ${currentFamily}`);
   //       if(fontApplied){
   //         //font is loaded - return family name
   //         actualFontFamily = currentFamily;
   //         break;
   //       }
   //     }
   // }
   const fontMap = new Map();
   fontMap.set("\"Share Tech\"",
      {fontSize: '14pt', sidebarWidthRem: 27.5, cardDescriptionBasis: '11rem', relativeBasis: '1.0rem'}
   );
   fontMap.set("\"JetBrains Mono\"",
      {fontSize: '12pt', sidebarWidthRem: 32, cardDescriptionBasis: '12rem', relativeBasis: '1.0rem'}
   );
   fontMap.set("\"Segoe UI\"",
      {fontSize: '12pt', sidebarWidthRem: 31, cardDescriptionBasis: '11rem', relativeBasis: '1.0rem'}
   );
   let fontData = fontMap.get(actualFontFamily) ||
      {fontSize: '13pt', sidebarWidthRem: 30, cardDescriptionBasis: '13rem', relativeBasis: '1.0rem'};
   console.log("-- FONT " + actualFontFamily + " : " + fontData.fontSize);
   document.documentElement.style.setProperty('--font-size-main', fontData.fontSize);
   document.documentElement.style.setProperty('--sidebar-width', fontData.sidebarWidth + "rem");
   // SIDEBAR_WIDTH_OVERRIDE.val = fontData.sidebarWidth;
   SIDEBAR_WIDTH_FONT_PX.val = Util.remToPx(fontData.sidebarWidthRem);
   document.documentElement.style.setProperty('--card-description-basis', fontData.cardDescriptionBasis);
   document.documentElement.style.setProperty('--size-normalsize', fontData.relativeBasis);
}

function configureFullscreenSwitch() {
   const checkFullscreen = function(event) {
      const dw = (window.outerWidth-screen.width);
      const dh = (window.outerHeight-screen.height);
      const enable = dw == 0 && dh == 0;
      // console.log('checkFullscreen', dw, dh);
      setFullScreen(fullscreenElement, enable);
   };
   document.body.onfullscreenchange = (e) => {
      setFullScreen(fullscreenElement, !IS_FULLSCREEN.val)
   };
   window.addEventListener("keyup", function(event){
      var code = event.keyCode || event.which || event.code;
      if(code == 122){
         // console.log('F11 keyup');
         setTimeout(function(){checkFullscreen();},250);
         event.preventDefault();
      }
      else if (code == 27) {
         // console.log('ESC keyup');
         if (IS_FULLSCREEN.val) {
            event.preventDefault();
         }
      }
   });
   window.addEventListener("resize", function(){
      if (IS_FULLSCREEN.val) {
         // console.log('RESIZE checkFullscreen');
         setTimeout(function(){checkFullscreen();},250);
      }
   })
}

function addParallax({element, sensitivityXY, parallaxes, centers}) {
   document.addEventListener("mousemove", parallax);
   function parallax(e) {
       let [sx, sy] = sensitivityXY;
       let _w = window.innerWidth/2;
       let _h = window.innerHeight/2;
       let _mouseX = e.clientX;
       let _mouseY = e.clientY;
       let bgPositions = parallaxes.map(function(parallax, i) {
         let [cx, cy] = centers[i];
         return `${cx + (_mouseX - _w) * sx * parallax}px ${cy + Math.max(_mouseY * sy * parallax, 0)}px`;
       }).join(',');
      //  let _depth = `${cx + (_mouseX - _w) * sx}px ${Math.max(cy + _mouseY * sy, 0)}px`;
      //  let _depthbg = ``;
      //  let x = `${_depth}, ${_depthbg}`;
       element.style.backgroundPosition = bgPositions;
   }
}

function getCanvasConvigurationFunc(canvas_id) {
   let canvas = document.getElementById(canvas_id);
   
   function resizeCanvas() {
      canvas.width = canvas.clientWidth;
      canvas.height = window.innerHeight;
      console.log(canvas.width, canvas.height);
      if (WASM_INSTANCE) {
         let gl = canvas.getContext("webgl2");
         WASM_INSTANCE.wasm_resize(canvas.width, canvas.height);
      }
   }
   document.addEventListener("visibilitychange", resizeCanvas, false);
   window.addEventListener('resize', resizeCanvas, false);
   window.addEventListener('focus', resizeCanvas, false);
   return resizeCanvas;
}

function getScrollCallback({chapterBorderStickiness, chapterAfterBorderStickiness}) {
   let borderStickinessLeft = chapterBorderStickiness;
   let afterBorderStickinessLeft = chapterAfterBorderStickiness;
   function impl(event, scrollSpeed) {
      const curL1 = CURRENT_CV_PAGE[0].val;
      const curL2 = CURRENT_CV_PAGE[1].val;
      const curTextDiv = document.getElementById(curL2).getElementsByClassName("inside")[0];
      const isScrollable = curTextDiv.scrollHeight > curTextDiv.clientHeight || curTextDiv.scrollWidth > curTextDiv.clientWidth;

      const nextOrPrev = scrollSpeed > 0 ? "next" : "prev";
      console.assert(["next", "prev"].includes(nextOrPrev));

      if (borderStickinessLeft > 0) {
         if (curTextDiv.scrollTop + scrollSpeed <= 0) {
            // hitting top border
            --borderStickinessLeft;
            curTextDiv.scrollTop = 0;
         } else if (curTextDiv.scrollTop + scrollSpeed >= curTextDiv.scrollHeight - curTextDiv.offsetHeight) {
            // hitting bottom border
            --borderStickinessLeft;
            curTextDiv.scrollTop = curTextDiv.scrollHeight;
         } else {
            // scroll direction changed, reset hitting border
            borderStickinessLeft = chapterBorderStickiness;
            if (isScrollable && afterBorderStickinessLeft > 0) {
               --afterBorderStickinessLeft;
               // console.log("afterStickiness", afterBorderStickinessLeft);
               event.preventDefault();
            } else {
               curTextDiv.scrollTop += scrollSpeed;
            }
         }
      }

      // console.log("borderStickiness", borderStickinessLeft);
      if (borderStickinessLeft > 0) {
         return;
      }

      // hit the border through the limit, change chapter
      let [nextL1, nextL2] = CV_PAGE_ORDER[curL1][curL2][nextOrPrev];
      let infiniteLoopLimit = 64;
      while (infiniteLoopLimit-- >= 0 && ["__begin__", "__end__"].includes(nextL2)) {
         [nextL1, nextL2] = CV_PAGE_ORDER[nextL1][nextL2][nextOrPrev];
      }
      if (infiniteLoopLimit <= 0) {
         console.log('Weird chapter graph (too many jumps)');
         return;
      }
      CURRENT_CV_PAGE[0].val = nextL1;
      CURRENT_CV_PAGE[1].val = nextL2;
      curTextDiv.scrollTop = 0;
      borderStickinessLeft = chapterBorderStickiness;
      afterBorderStickinessLeft = chapterAfterBorderStickiness;
   };
   return impl;
}

function getBuildDataString() {
   return BUILD_DATA === undefined ? "" : `Commit: ${BUILD_DATA["git-commit"]}\n\
Committed: ${BUILD_DATA["git-commit-date"]}\n\
Deployed: ${BUILD_DATA["deploy-date"]}\n\
DEBUG: ${BUILD_DATA["debug"]}`;
}

function getCurrentDemoId() {
   const remap = {
      "__stub__": DemoId.Stub,
      "career_huawei": DemoId.CareerHuawei,
      "career_samsung": DemoId.CareerSamsung,
      "publications_wacv_2024": DemoId.PublicationWacv2024,
      "project_this_cv": DemoId.ProjectThisCv,
      "project_unity_4X_strategy_volunteer": DemoId.Triangle,
      "project_image_processing_tool": DemoId.ProjectTreesRuler,
      "education_master": DemoId.EducationMasters,
      "education_bachelor": DemoId.EducationBachelor,
   }
   return remap[CURRENT_CV_PAGE[1].val];
}

function getGraphicsLevel() {
   const remap = {
      0: GraphicsLevel.Minimal,
      0: GraphicsLevel.Low,
      0: GraphicsLevel.Medium,
      0: GraphicsLevel.High,
      0: GraphicsLevel.Ultra,
   }
   return remap[CURRENT_CV_PAGE[1].val];
}

window.onload = function() {
   van.derive(() => {
      sidebar.style.flexBasis = Math.max(SIDEBAR_WIDTH_OVERRIDE_PX.val || 0, SIDEBAR_WIDTH_FONT_PX.val || 0) + "px";
   });
   loadCookies();
   van.derive(() => {
      dumpCookies();
   })
   van.derive(() => {
      console.log("$$ Setting font " + CURRENT_FONT_FAMILY.val);
      const baseFamilies = getComputedStyle(document.body).getPropertyValue('--font-families-const');
      let customFamilies = CURRENT_FONT_FAMILY.val + ", ";
      // FONTS.forEach((f) => {
      //    if (f !== currentFont.val) {
      //       customFamilies += f + ", ";
      //    }
      // });
      const overrideFamilies = customFamilies + baseFamilies;
      document.documentElement.style.setProperty('--font-families', overrideFamilies);
      configureFromFont(CURRENT_FONT_FAMILY.val, CURRENT_LANGUAGE.val);
   });
   van.derive(() => {
      const demoId = getCurrentDemoId();
      if (WASM_INSTANCE) {
         console.log("Start switching demo", demoId);
         WASM_INSTANCE.wasm_start_loading_demo(demoId);
      }
   })
   configureFromFont(CURRENT_FONT_FAMILY.val, CURRENT_LANGUAGE.val); // other elements' relative sizes depend on this configuration
   configureResizingBorder();
   configureFullscreenSwitch();
   const configureCanvas = getCanvasConvigurationFunc(CANVAS_ID);
   configureCanvas();
   van.add(document.getElementById("canvas-controls"), FullscreenButton({extraClasses: "fullscreen-button", height: "80"}));
   van.add(document.getElementById("canvas-controls"), HelpButton({height: "80"}));
   van.add(document.getElementById("canvas-wrapper"), LoadingScreen());
   van.add(document.getElementById("controls_column"), LanguagePicker(CURRENT_LANGUAGE, CURRENT_FONT_FAMILY, /*vertical*/ false));
   van.add(document.getElementById("controls_column"), GraphicsLevelPicker(CURRENT_GRAPHICS_LEVEL, /*vertical*/ false));
   van.add(document.getElementById("controls_column"), FpsLimitPicker(CURRENT_FPS_LIMIT, /*vertical*/ false));
   van.add(document.getElementById("controls_row"), RepositoryLink({width: "49%"}));
   van.add(document.getElementById("controls_row"), ClearCookiesButton({width: "49%"}));
   document.querySelectorAll(".firstinfo").forEach(element => {
      van.add(element, PersonalCard())
   });
   van.add(document.getElementById("side-card-info"), MoreSkillsButton());
   van.add(document.getElementById("card-links"), ResumePdfLink());
   van.add(document.getElementById("sidebar"), CvContent(CURRENT_CV_PAGE, CV_PAGE_ORDER));
   van.derive(() => {
      if (IS_TUTORIAL_SHOWN.val == false) {
         return;
      }
      van.add(document.body, ControlsPopup({onclose: () => {
         van.add(document.getElementById("resize-tooltip"), ResizeTooltip({
            timeoutMillisec: 7000,
            onclose: () => {}
         }));
         IS_TUTORIAL_SHOWN.val = false;
      }}));
   });
   
   if (IS_INTRO_SHOWN.val == true) {
      console.log('@@@', IS_INTRO_SHOWN.val);
      van.add(document.body, IntroPopup({onclose: () => {
         IS_TUTORIAL_SHOWN.val = true;
         IS_INTRO_SHOWN.val = false;
      }}));
      const addAppearAnimation = (el) => Util.addClass(el, 'animated-appear');
      // document.querySelectorAll(".card-container")
      //    .forEach(addAppearAnimation);
      // document.querySelectorAll(".badgescard")
      //    .forEach(addAppearAnimation);
   }
   const myPhoto = document.getElementById('my-photo');
   loadBuildData().finally(() => {
      console.log("LOADED BUILD_DATA", BUILD_DATA);

      // show build data on hover of my photo
      myPhoto.title = getBuildDataString();

      // add select to font family
      if (BUILD_DATA.debug) {
         van.add(document.getElementById("controls_column"), FontPicker(CURRENT_FONT_FAMILY));
      }

      // add scroll management that controls CV chapters
      const callbackScrollSpeed = Util.computeScrollSpeed();
      const scrollCallback = getScrollCallback({
         chapterBorderStickiness: BUILD_DATA.debug ? 3 : 5,
         chapterAfterBorderStickiness: BUILD_DATA.debug ? 0 : 3,
      });
      window.addEventListener('wheel', event => {
         let wheelSpeed = callbackScrollSpeed(event);
         const ELEMENT_SCROLL_SPEED = BUILD_DATA.debug ? 0.3 : 0.15;
         scrollCallback(event, wheelSpeed * ELEMENT_SCROLL_SPEED);
      }, { capture: true, passive: false });
   });

   if (ADD_PARALLAX) {
      // myPhoto.style.backgroundImage = "url(../assets/my_photo_tiny.png), url(../assets/bg6-min.png)";
      myPhoto.style.backgroundImage = "url(../assets/my_photo_tiny.png), url(../assets/bg10-min.png)";
      addParallax({
         element: myPhoto, sensitivityXY: [0.015, 0.010],
         parallaxes: [1.0, 0.15], centers: [[0, 0], [-10, -5]]
      });
   } else {
      myPhoto.style.backgroundImage = "url(../assets/my_photo_tiny.png)";
   }

   // Use ES module import syntax to import functionality from the module
   // that we have compiled.
   //
   // Note that the `default` import is an initialization function which
   // will "boot" the module and make it ready to use. Currently browsers
   // don't support natively imported WebAssembly as an ES module, but
   // eventually the manual initialization won't be required!
   (async function run() {
      // First up we need to actually load the wasm file, so we use the
      // default export to inform it where the wasm file is located on the
      // server, and then we wait on the returned promise to wait for the
      // wasm to be loaded.
      await init('./wasm/index_bg.wasm');

      // And afterwards we can use all the functionality defined in wasm.
      WASM_INSTANCE = new WasmInterface(CANVAS_ID);
      configureCanvas();
      WASM_INSTANCE.wasm_set_fps_limit(CURRENT_FPS_LIMIT.val);
      WASM_INSTANCE.wasm_set_graphics_level(CURRENT_GRAPHICS_LEVEL.val);
      WASM_INSTANCE.wasm_start_loading_demo(getCurrentDemoId());
      WASM_INSTANCE.wasm_loop();
   })();
}