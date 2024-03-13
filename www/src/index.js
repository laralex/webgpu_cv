import { wasm_startup, wasm_loop, wasm_resize } from "my-wasm";
const {div, button, i, label, img, svg, path, input, details, summary, p, li, a, option, select, span, ul, h1, h2, h3} = van.tags

const ADD_PARALLAX = true;
const CANVAS_ID = "main-canvas";

const CURRENT_GRAPHICS_LEVEL = van.state("medium");
const CURRENT_FONT_FAMILY = van.state("\"Share Tech\"");
// const DEFAULT_MAIN_CHAPTER = "chapter_career";
// const DEFAULT_SUB_CHAPTER = "career_huawei";
const DEFAULT_MAIN_CHAPTER = "chapter_education";
const DEFAULT_SUB_CHAPTER = "education_master";
const CURRENT_CV_PAGE = [van.state(DEFAULT_MAIN_CHAPTER), van.state(DEFAULT_SUB_CHAPTER)];
const CV_PAGE_ORDER = {}

function FullscreenButton(fullscreenElement) {
   const IS_FULLSCREEN = van.state(false);
   // van.derive(() => console.log('full', IS_FULLSCREEN.val));
   function setFullScreen(elem, enableFullscreen) {
      // ## The below if statement seems to work better ## if ((document.fullScreenElement && document.fullScreenElement !== null) || (document.msfullscreenElement && document.msfullscreenElement !== null) || (!document.mozFullScreen && !document.webkitIsFullScreen)) {
      if (enableFullscreen && (document.fullScreenElement !== undefined && document.fullScreenElement === null) || (document.msFullscreenElement !== undefined && document.msFullscreenElement === null) || (document.mozFullScreen !== undefined && !document.mozFullScreen) || (document.webkitIsFullScreen !== undefined && !document.webkitIsFullScreen)) {
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
      } else if (!enableFullscreen) {
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
  }
   document.body.onfullscreenchange = (e) => {
      setFullScreen(fullscreenElement, !IS_FULLSCREEN.val)
   };
   const checkFullscreen = function(event) {
      const enable = (window.outerWidth-screen.width) ==0 && (window.outerHeight-screen.height) ==0;
      setFullScreen(fullscreenElement, enable);
   };
   window.addEventListener("keyup", function(event){
      var code = event.keyCode || event.which;
      if(code == 122){
          setTimeout(function(){checkFullscreen();},250);
      }
   });
   return div({class: "fullscreen-button", onclick: () => setFullScreen(fullscreenElement, !IS_FULLSCREEN.val)},
      img({
         src: () => IS_FULLSCREEN.val ? "../assets/collapse-regular-240.png" : "../assets/expand-regular-240.png",
      })
   );
}

function GeoLocation() {
   return div({class: "geo-location"},
      img({src: "../assets/1globe_tss.png"}),
      localizeUi('geo_location')
   );
}

function FontPicker(currentFont) {
   const FONTS = [
      "\"Share Tech\"",
      "\"JetBrains Mono\"",
      "\"Segoe UI\"",
   ];
   van.derive(() => {
      const baseFamilies = getComputedStyle(document.body).getPropertyValue('--font-families-const');
      let customFamilies = currentFont.val + ", ";
      // FONTS.forEach((f) => {
      //    if (f !== currentFont.val) {
      //       customFamilies += f + ", ";
      //    }
      // });
      const overrideFamilies = customFamilies + baseFamilies;
      document.documentElement.style.setProperty('--font-families', overrideFamilies);
      configureFontSize(currentFont.val);
   });
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
         () => meta.emoji + " " + UI_STRINGS[meta.labelId][tooltipLanguage.val]));
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
   const GRAPHICS_LEVELS = {
      low: {labelId: 'graphics_low', emoji: "✰✰✰"},
      medium: {labelId: 'graphics_medium', emoji: "★✰✰"},
      high: {labelId: 'graphics_high', emoji: "★★✰"},
      ultra: {labelId: 'graphics_ultra', emoji: "★★★"},
   }
   const options = Object.entries(GRAPHICS_LEVELS).map(([level, meta]) =>
      option({ value: level, selected: () => level == currentGraphicsLevel.val},
         () => localizeString(meta.labelId)().text + " " +  meta.emoji));
   van.derive(() => console.log("Set graphics_level="+currentGraphicsLevel.val));
   const labelBefore = isVertical ? span(localizeUi('graphics_levels')) : null;
   const labelAfter = !isVertical ? span(localizeUi('graphics_levels')) : null;
   return div(
      { class: 'graphics-picker ' + (isVertical ? "flex-column" : "flex-row") },
      labelBefore,
      select({
         class: 'interactive btn',
         oninput: e => currentGraphicsLevel.val = e.target.value,
         value: currentGraphicsLevel,
      }, options,),
      labelAfter,
   );
}

function ResumePdfLink() {
   return button({class:"btn-block interactive btn", role:"button", style:"width:100%"},
      // bxs-download
      i({ class: "bx bxs-file-pdf bx-tada font-Huge", style: "color: var(--color-gmail);"}),
      a({ class: "font-normalsize", href: localizeString("pdf_cv_href")().text, target: "_blank"},
         () => localizeString("cv")().text + " " + localizeString("pdf")().text,
         // label({style: "display:block;"}, ),
         // label({style: "display:block;"}, ),
      ));
}

function ResizeTooltip({timeoutMillisec}) {
   const resizeElement = document.getElementById("resize-border");
   const sidebar = document.getElementById("sidebar");
   const shouldHide = van.state(false);
   setTimeout(() => shouldHide.val = true, timeoutMillisec);
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
      sidebar.style.flexBasis = (parseInt(getComputedStyle(sidebar, '').flexBasis) + dx) + "px";
      shouldHide.val = true;
      return pauseEvent(e);
   }

   resizeElement.addEventListener("mousedown", function(e){
      if (e.offsetX >= 0) {
         m_pos = e.x;
         document.addEventListener("mousemove", resize, false);
      }
   }, false);

   document.addEventListener("mouseup", function(){
      console.log("mouseup")
      document.removeEventListener("mousemove", resize, false);
   }, false);
   
   return () => DEBUG || shouldHide.val ? null : div({class: "bubble shadow left", onclick: (e) => shouldHide.val = true}, localizeUi("resize_tooltip"));
}

function RepositoryLink() {
   return button({class:"btn-block interactive btn font-large", role:"button", style: "width:100%; margin: 2px 0 0 0;"},
      a({href: "https://github.com/laralex/my_web_cv", target: "_blank"},
         i({ class: "bx bxl-github", style: "font-size:1.3rem;color: var(--color-github)"}),
         label(" "),
         label(localizeUi("web_cv_github")),
      ));
}

function PersonalCard() {
   return div({class: "profileinfo"},
      h1({class: "font-LARGE bold"}, localizeUi("name_surname")),
      h3(localizeUi("job_title")),
      div({class: "specialization"}, () => {
         const cg = localizeUi("specialty_computer_graphics")();
         const deepLearning = localizeUi("specialty_deep_learning", true)();
         console.log("@@@", deepLearning);
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
      }, i({class: () => isExpanded.val ? "bx bxs-up-arrow" : "bx bxs-down-arrow"}), i(" "), localizeUi("skills_title")),
      div({class: "inside", style: () => isExpanded.val ? "" : "display: none;" },
         // div({class: "icons"},
         //    // img({src: "https://cdn.jsdelivr.net/gh/devicons/devicon/icons/opengl/opengl-original.svg" }),
         //    // img({src: "https://cdn.jsdelivr.net/gh/devicons/devicon/icons/cplusplus/cplusplus-line.svg" }),
         //    img({src: "https://cdn.jsdelivr.net/gh/devicons/devicon/icons/rust/rust-plain.svg" }),
         //    // img({src: "https://cdn.jsdelivr.net/gh/devicons/devicon/icons/python/python-original-wordmark.svg"}),
         //    img({src: "https://cdn.jsdelivr.net/gh/devicons/devicon/icons/csharp/csharp-original.svg" }),
         //    img({src: "https://cdn.jsdelivr.net/gh/devicons/devicon/icons/java/java-original-wordmark.svg" }),
         //    img({src: "https://cdn.jsdelivr.net/gh/devicons/devicon/icons/unity/unity-original-wordmark.svg" }),
         //    img({src: "https://cdn.jsdelivr.net/gh/devicons/devicon/icons/pytorch/pytorch-original-wordmark.svg"}),
         //    img({src: "https://cdn.jsdelivr.net/gh/devicons/devicon/icons/docker/docker-original-wordmark.svg"}),
         //    ),
         ul(
            li(span("Rust, C#, Java, JavaScript")),
            li(span("PyTorch, Docker, Qualcomm\xa0SNPE, Unity, ARCore, Linux, LaTeX")),
            li(span(localizeUi("skills_languages_1"))),
            ),
      ),
   )
}

function IntroPopup({onclose}) {
   const closed = van.state(false);
   const needAnimation = van.state(true);
   van.derive(() => { if (closed.val && onclose) onclose(); });
   return () => closed.val ? null :div({class: "popup font-large retro-box checkerboard-background"},
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
         button({class: "btn popup-btn font-large", onclick: (e) => closed.val = true }, localizeUi("intro_close")))
   )
}

function ControlsPopup({onclose}) {
   const closed = van.state(false);
   van.derive(() => { if (closed.val && onclose !== undefined) onclose(); });
   return () => closed.val ? null :div({class: "popup font-large retro-box checkerboard-background"},
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
      div({class: "controls"},
         button({class: "btn popup-btn font-large", onclick: (e) => closed.val = true }, localizeUi("controls_close")))
   );
}

function getFontFamilies(elements){
   let usedFonts = [];
   elements.forEach(function(el,i){
     let nodeType = el.nodeName.toLowerCase();
     let fontFamily = getComputedStyle(el).fontFamily;
     let familyArr = fontFamily.split(',');
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

function configureFontSize(fontFamily = null) {
   let para = document.querySelector('p');
   let actualFontFamily = fontFamily || getFontFamilies([para])[0].font;
   const fontMap = new Map();
   fontMap.set("\"Share Tech\"",
      {fontSize: '16pt', sidebarWidth: '26rem', cardDescriptionBasis: '11rem', relativeBasis: '0.95rem'}
   );
   fontMap.set("\"JetBrains Mono\"",
      {fontSize: '13pt', sidebarWidth: '31rem', cardDescriptionBasis: '13rem', relativeBasis: '1.0rem'}
   );
   fontMap.set("\"Segoe UI\"",
      {fontSize: '13pt', sidebarWidth: '31rem', cardDescriptionBasis: '11rem', relativeBasis: '1.0rem'}
   );
   let fontData = fontMap.get(actualFontFamily) ||
      {fontSize: '15pt', sidebarWidth: '30rem', cardDescriptionBasis: '13rem', relativeBasis: '1.0rem'};
   console.log("-- FONT " + actualFontFamily + " : " + fontData.fontSize);
   document.documentElement.style.setProperty('--font-size-main', fontData.fontSize);
   document.documentElement.style.setProperty('--sidebar-width', fontData.sidebarWidth);
   document.documentElement.style.setProperty('--card-description-basis', fontData.cardDescriptionBasis);
   document.documentElement.style.setProperty('--size-normalsize', fontData.relativeBasis);
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

function configureCanvas() {
   let canvas = document.getElementById(CANVAS_ID);
   let gl = canvas.getContext("webgl2");

   function resizeCanvas() {
      canvas.width = canvas.clientWidth;
      canvas.height = window.innerHeight;
      console.log(canvas.width, canvas.height);
      wasm_resize(gl, canvas.width, canvas.height);
   }
   document.addEventListener("visibilitychange", resizeCanvas, false);
   window.addEventListener('resize', resizeCanvas, false);
   window.addEventListener('focus', resizeCanvas, false);
   resizeCanvas();
}

function getScrollCallback() {
   const MAX_SCROLL_BORDER_HITS = DEBUG ? 3 : 6;
   let borderHitsLeft = MAX_SCROLL_BORDER_HITS;
   function impl(scrollSpeed) {
      const nextOrPrev = scrollSpeed > 0 ? "next" : "prev";
      console.assert(["next", "prev"].includes(nextOrPrev));
      const curL1 = CURRENT_CV_PAGE[0].val;
      const curL2 = CURRENT_CV_PAGE[1].val;
      const curTextDiv = document.getElementById(curL2).getElementsByClassName("inside")[0];
      if (borderHitsLeft > 0) {
         if (curTextDiv.scrollTop + scrollSpeed <= 0) {
            --borderHitsLeft;
            curTextDiv.scrollTop = 0;
         } else if (curTextDiv.scrollTop + scrollSpeed >= curTextDiv.scrollHeight - curTextDiv.offsetHeight) {
            --borderHitsLeft;
            curTextDiv.scrollTop = curTextDiv.scrollHeight;
         } else {
            // TODO: change scroll
            curTextDiv.scrollTop += scrollSpeed;
            borderHitsLeft = MAX_SCROLL_BORDER_HITS;
         }
      }
      if (borderHitsLeft > 0) {
         return;
      }
      let [nextL1, nextL2] = CV_PAGE_ORDER[curL1][curL2][nextOrPrev];
      let maxJumps = 64;
      while (maxJumps-- >= 0 && ["__begin__", "__end__"].includes(nextL2)) {
         [nextL1, nextL2] = CV_PAGE_ORDER[nextL1][nextL2][nextOrPrev];
      }
      if (maxJumps <= 0) {
         return;
      }
      // console.log(curL2, "@@@", CURRENT_CV_PAGE.val.level2[curL1]);
      // CURRENT_CV_PAGE.val.level2[curL1].val = {};
      CURRENT_CV_PAGE[0].val = nextL1;
      CURRENT_CV_PAGE[1].val = nextL2;
      borderHitsLeft = MAX_SCROLL_BORDER_HITS;
   };
   return impl;
}

window.onload = function() {
   van.add(document.getElementById("canvas-wrapper"), FullscreenButton(document.getElementById("canvas-wrapper")));
   van.add(document.getElementById("side-top__1"), LanguagePicker(CURRENT_LANGUAGE, CURRENT_FONT_FAMILY, /*vertical*/ false));
   if (DEBUG || true) {
      van.add(document.getElementById("side-top__1"), FontPicker(CURRENT_FONT_FAMILY));
   }
   van.add(document.getElementById("side-top__1"), GraphicsLevelPicker(CURRENT_GRAPHICS_LEVEL, /*vertical*/ false));
   van.add(document.getElementById("side-links__1"), ResumePdfLink());
   van.add(document.getElementById("side-top__1"), RepositoryLink());
   document.querySelectorAll(".firstinfo").forEach(element => {
      // console.log(element);
      van.add(element, PersonalCard())
   });
   van.add(document.getElementById("side-card-info"), MoreSkillsButton());
   // van.add(document.getElementById("side-content"), CvChapter("chapter_career", "#7BD3EA"));
   // van.add(document.getElementById("side-content"), CvChapter("chapter_publications", "#A1EEBD"));
   // van.add(document.getElementById("side-content"), CvChapter("chapter_projects", "#F6F7C4"));
   // van.add(document.getElementById("side-content"), CvChapter("chapter_education", "#F6D6D6"));
   van.add(document.getElementById("sidebar"), CvContent(CURRENT_CV_PAGE, CV_PAGE_ORDER));
   // CURRENT_CV_PAGE[0].val = DEFAULT_MAIN_CHAPTER;
   // CURRENT_CV_PAGE[1].val = DEFAULT_SUB_CHAPTER;
   if (!DEBUG) {
      van.add(document.body, IntroPopup({onclose: () => {
         van.add(document.body, ControlsPopup({onclose: () => {
            van.add(document.getElementById("resize-tooltip"), ResizeTooltip({timeoutMillisec: 7000}));
         }}));
      }}));
      const addAppearAnimation = (el) => Util.addClass(el, 'animated-appear');
      document.querySelectorAll(".card-container")
         .forEach(addAppearAnimation);
      document.querySelectorAll(".badgescard")
         .forEach(addAppearAnimation);
   }
   const myPhoto = document.getElementById('my-photo');
   if (ADD_PARALLAX) {
      // myPhoto.style.backgroundImage = "url(../assets/my_photo_tiny.png), url(../assets/bg6-min.png)";
      myPhoto.style.backgroundImage = "url(../assets/my_photo_tiny.png), url(../assets/clouds.png), url(../assets/cloud2.png), url(../assets/moon.png), url(../assets/bg10-min.png)";
      myPhoto.style.backgroundSize = "cover, 0%, 0%, 25%, 150%";
      myPhoto.style.backgroundScale = "cover, 100%, 60%, 25%, 150%";
      // myPhoto.style.backgroundImage = "url(../assets/my_photo_tiny.png)";
      addParallax({
         element: myPhoto, sensitivityXY: [0.015, 0.010],
         parallaxes: [1.0, 0.6, 0.4, 0.3, 0.15], centers: [[0, 0], [0, -10], [75, 10], [/*100*/10000, 5], [-10, -5]]
      });
   } else {
      myPhoto.style.backgroundImage = "url(../assets/my_photo_tiny.png)";
   }

   // listen to "scroll" event
   const callbackScrollSpeed = Util.computeScrollSpeed();
   const scrollCallback = getScrollCallback();
   window.addEventListener('wheel', event => {
      let wheelSpeed = callbackScrollSpeed(event);
      const ELEMENT_SCROLL_SPEED = DEBUG ? 0.3 : 0.15;
      scrollCallback(wheelSpeed * ELEMENT_SCROLL_SPEED);
   }, { capture: true });

   configureFontSize();
   configureCanvas();
   wasm_startup();
   wasm_loop(CANVAS_ID);
}