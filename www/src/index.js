import { wasm_startup, wasm_loop, wasm_resize } from "my-wasm";
const {div, button, i, label, img, svg, path, input, li, a, option, select, span, ul} = van.tags

const ARROW_SVG_PATH = '<svg viewBox="0 0 16 16"><polygon points="3,5 8,11 13,5 "></polygon></svg>';
const GLOBE_SVG_PATH = '<svg viewBox="0 0 16 16"><path d="M8,0C3.6,0,0,3.6,0,8s3.6,8,8,8s8-3.6,8-8S12.4,0,8,0z M13.9,7H12c-0.1-1.5-0.4-2.9-0.8-4.1 C12.6,3.8,13.6,5.3,13.9,7z M8,14c-0.6,0-1.8-1.9-2-5H10C9.8,12.1,8.6,14,8,14z M6,7c0.2-3.1,1.3-5,2-5s1.8,1.9,2,5H6z M4.9,2.9 C4.4,4.1,4.1,5.5,4,7H2.1C2.4,5.3,3.4,3.8,4.9,2.9z M2.1,9H4c0.1,1.5,0.4,2.9,0.8,4.1C3.4,12.2,2.4,10.7,2.1,9z M11.1,13.1 c0.5-1.2,0.7-2.6,0.8-4.1h1.9C13.6,10.7,12.6,12.2,11.1,13.1z"></path></svg>';
const SIDEBAR = document.getElementById("sidebar");
const SIDEBAR_TOP = document.getElementById("side-top")
const CANVAS_ID = "main-canvas";

const UI_STRINGS = {
   english: {en: "English", ru: "English" },
   russian: {en: "Russian", ru: "Русский" },
   low: {en: "Low    ", ru: "Низкое " },
   medium: {en: "Medium", ru: "Среднее" },
   high: {en: "High  ", ru: "Высокое" },
   ultra: {en: "Ultra ", ru: "Ультра" },
   ui_language: {en: "UI Language", ru: "Язык UI"},
   graphics_levels: {en: "Graphics quality", ru: "Качество графики"},
   pdf: {en: "PDF", ru: "PDF"},
   cv: {en: "CV", ru: "CV"},
   pdf_cv_href: {en: "./assets/larionov_rendering_cv_eng_112023.pdf", ru: "./assets/larionov_rendering_cv_rus_112023.pdf"},
   web_cv_github: {en: "Source code of this demo", ru: "Исходный код демки"},
   skills_title: {en: "Extra skills", ru: "Прочие компетенции"},
   skills_languages_1: {en: "English 🇬🇧 (C1), Russian 🇷🇺 (N)", ru: "Английский 🇬🇧 (C1), Корейский 🇰🇷 (А2)"},
   skills_languages_2: {en: "Korean 🇰🇷 (A2), Polish 🇵🇱 (A1)", ru: "Польский 🇵🇱 (А1), Русский 🇷🇺"},
}

const CURRENT_LANGUAGE = van.state("en");
const LANGUAGES = {
	en: {label: UI_STRINGS['english'], icon: "../assets/flag_GB.png", emoji: "🇬🇧"},
	ru: {label: UI_STRINGS['russian'], icon: "../assets/flag_RU.png", emoji: "🇷🇺"},
}
const CURRENT_GRAPHICS_LEVEL = van.state("medium");
const GRAPHICS_LEVELS = {
	low: {label: UI_STRINGS['low'], emoji: "✰✰✰"},
	medium: {label: UI_STRINGS['medium'], emoji: "★✰✰"},
	high: {label: UI_STRINGS['high'], emoji: "★★✰"},
	ultra: {label: UI_STRINGS['ultra'], emoji: "★★★"},
}

function localize_d(dict) {
   return dict[CURRENT_LANGUAGE.val];
}

function localize_ui(key) {
   return () => localize_d(UI_STRINGS[key]);
}

function LanguagePicker(currentLanguage) {
   function localize_page(language)
   {
      if (! (Object.keys(LANGUAGES).includes(language))) {
         return;
      }
      console.log("Set language=" + language);
      let lang = ':lang(' + language + ')';
		let hide = '[lang]:not(' + lang + ')';
		document.querySelectorAll(hide).forEach(function (node) {
			node.style.display = 'none';
		});
		let show = '[lang]' + lang;
		document.querySelectorAll(show).forEach(function (node) {
			node.style.display = 'unset';
		});
   }

	van.derive(() => localize_page(currentLanguage.val));
	const options = Object.entries(LANGUAGES).map(([language, meta]) =>
      option({ value: language, selected: () => language == currentLanguage.val},
         () => meta.emoji + " " + meta.label[currentLanguage.val]));
   return () => div(
      { class: 'language-picker' },
      span(localize_ui('ui_language')),
      select({
         class: 'interactive btn',
         value: currentLanguage,
         oninput: e => currentLanguage.val = e.target.value,
      }, options,),
   );
}

function GraphicsLevelPicker(currentGraphicsLevel) {
   const options = Object.entries(GRAPHICS_LEVELS).map(([level, meta]) =>
      option({ value: level, selected: () => level == currentGraphicsLevel.val},
         () => localize_d(meta.label) + " " +  meta.emoji));
   van.derive(() => console.log("Set graphics_level="+currentGraphicsLevel.val));
   return div(
      { class: 'graphics-picker' },
      span(localize_ui('graphics_levels')),
      select({
         class: 'interactive btn',
         oninput: e => currentGraphicsLevel.val = e.target.value,
         value: currentGraphicsLevel,
      }, options,),
   );
}

function ResumePdfLink() {
   // return button({class:"btn-block interactive btn font-normalsize", role:"button"},
   //    a({href: localize_ui("pdf_cv_href"), target: "_blank"},
   //       i({ class: "bx bxs-file-pdf bx-tada font-Huge", style: "color: var(--color-gmail);margin:0;padding:0;"}),
   //       label(localize_ui("pdf_cv")),
   //    ));
   return button({class:"btn-block interactive btn font-normalsize", role:"button", style:"width:100%"},
      // bxs-download
      i({ class: "bx bxs-file-pdf bx-tada font-Huge", style: "color: var(--color-gmail);"}),
      a({href: localize_ui("pdf_cv_href"), target: "_blank", class: "btn"},
         label({style: "display:block;"}, localize_ui("pdf")),
         label({style: "display:block;"}, localize_ui("cv")),
      ));
}

function RepositoryLink() {
   return button({class:"btn-block interactive btn font-large", role:"button"},
      a({href: "https://github.com/laralex/my_web_cv", target: "_blank"},
         i({ class: "bx bxl-github", style: "font-size:1.3rem;color: var(--color-github)"}),
         label(localize_ui("web_cv_github")),
      ));
}

function MoreSkillsButton() {
   const isExpanded = van.state(false);
   return div({class: "badgescard"},
      button({
         class: "interactive btn font-Large expand-button",
         onclick: e => isExpanded.val = !isExpanded.val,
      }, i({class: () => isExpanded.val ? "bx bxs-up-arrow" : "bx bxs-down-arrow"}, "\t"), localize_ui("skills_title")),
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
            li("Rust, C#, Java, JavaScript"),
            li("PyTorch, Docker, Qualcomm SNPE"),
            span("Unity, ARCore, Linux, LaTeX"),
            li(localize_ui("skills_languages_1")),
            span(localize_ui("skills_languages_2")),
            ),
      ),
   )
}

const add_parallax = function({element, sensitivityXY, bgParallax, centerPx, centerBgPx}) {
   document.addEventListener("mousemove", parallax);
   function parallax(e) {
       let [cx, cy] = centerPx;
       let [cbx, cby] = centerBgPx;
       let [sx, sy] = sensitivityXY;
       let _w = window.innerWidth/2;
       let _h = window.innerHeight/2;
       let _mouseX = e.clientX;
       let _mouseY = e.clientY;
       let _depth = `${cx + (_mouseX - _w) * sx}px ${Math.max(cy + _mouseY * sy, 0)}px`;
       let _depthbg = `${cbx + (_mouseX - _w) * sx * bgParallax}px ${cby + Math.max(_mouseY * sy * bgParallax, 0)}px`;
       let x = `${_depth}, ${_depthbg}`;
       console.log(x);
       element.style.backgroundPosition = x;
   }
}

function js_setup_canvas() {
   let canvas = $("#"+CANVAS_ID)[0];
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

function js_setup_scrollify() {
   $(document).ready(function () {
      screenCheck();

      $('.scroll-control .one').click(function () {
         $.scrollify.move('#s-one');
      });
      $('.scroll-control .two').click(function () {
         $.scrollify.move('#s-two');
      });
      $('.scroll-control .three').click(function () {
         $.scrollify.move('#s-three');
      });
   });

   $(window).on('resize', function () {
      screenCheck();
   });

   function applyScroll() {
      $.scrollify({
         section: '.scroll',
         sectionName: 'section-name',
         standardScrollElements: 'canvas',
         easing: 'easeOutExpo',
         scrollSpeed: 200,
         offset: 0,
         scrollbars: true,
         setHeights: false,
         overflowScroll: true,
         updateHash: false,
         touchScroll: true,
      });
   }

   function screenCheck() {
      var deviceAgent = navigator.userAgent.toLowerCase();
      var agentID = deviceAgent.match(/(iphone|ipod|ipad)/);
      if (agentID || $(window).width() <= 1024) {
         // its mobile screen
         $.scrollify.destroy();
         $('section').removeClass('scroll').removeAttr('style');
         $.scrollify.disable();
      } else {
         // its desktop
         $('section').addClass('scroll');
         applyScroll();
         $.scrollify.enable();
      }
   }
}


van.add(document.getElementById("side-top__1"), LanguagePicker(CURRENT_LANGUAGE));
van.add(document.getElementById("side-top__2"), GraphicsLevelPicker(CURRENT_GRAPHICS_LEVEL));
van.add(document.getElementById("side-links__1"), ResumePdfLink());
van.add(document.getElementById("side-links__2"), RepositoryLink());
van.add(document.getElementById("side-card"), MoreSkillsButton());
document.querySelectorAll('.parallax').forEach(
   el => add_parallax({
      element: el, sensitivityXY: [0.01, 0.005], bgParallax: 0.5,
      centerPx: [0, 0], centerBgPx: [-20, -10]}));
js_setup_canvas();
wasm_startup();
wasm_loop(CANVAS_ID);