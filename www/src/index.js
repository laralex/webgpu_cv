import { wasm_startup, wasm_loop, wasm_resize } from "my-wasm";
const {div, button, i, label, img, svg, path, input, li, a, option, select, span, ul} = van.tags

const CURRENT_LANGUAGE = van.state("kr");
const ADD_PARALLAX = true;
// const DEBUG = true;
const DEBUG = false;
const CANVAS_ID = "main-canvas";
const UI_STRINGS = getLocalization()

const LANGUAGES = {
	en: {label: UI_STRINGS['english'], icon: "../assets/flag_GB.png", emoji: "🇬🇧"},
	ru: {label: UI_STRINGS['russian'], icon: "../assets/flag_RU.png", emoji: "🇷🇺"},
	kr: {label: UI_STRINGS['korean'], icon: "../assets/flag_RU.png", emoji: "🇰🇷"},
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
   if (UI_STRINGS[key] === undefined || UI_STRINGS[key][CURRENT_LANGUAGE.val] === undefined) {
      return key;
   }
   return () => UI_STRINGS[key][CURRENT_LANGUAGE.val];
}

function LanguagePicker(currentLanguage, isVertical, labelLanguage=undefined, labelStringId='ui_language') {
   function localizePage(language)
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
   if (!labelLanguage) {
      labelLanguage = currentLanguage;
   }
	van.derive(() => localizePage(currentLanguage.val));
	const options = Object.entries(LANGUAGES).map(([language, meta]) =>
      option({ value: language, selected: () => language == currentLanguage.val},
         () => meta.emoji + " " + meta.label[labelLanguage.val]));
   return () => div(
      { class: 'language-picker ' + (isVertical ? "flex-column" : "flex-row") },
      span(() => UI_STRINGS[labelStringId][labelLanguage.val]),
      select({
         class: 'interactive btn',
         value: currentLanguage,
         oninput: e => currentLanguage.val = e.target.value,
      }, options,),
   );
}

function GraphicsLevelPicker(currentGraphicsLevel, isVertical) {
   const options = Object.entries(GRAPHICS_LEVELS).map(([level, meta]) =>
      option({ value: level, selected: () => level == currentGraphicsLevel.val},
         () => localize_d(meta.label) + " " +  meta.emoji));
   van.derive(() => console.log("Set graphics_level="+currentGraphicsLevel.val));
   return div(
      { class: 'graphics-picker ' + isVertical ? "flex-column" : "flex-row" },
      span(localize_ui('graphics_levels')),
      select({
         class: 'interactive btn',
         oninput: e => currentGraphicsLevel.val = e.target.value,
         value: currentGraphicsLevel,
      }, options,),
   );
}

function ResumePdfLink() {
   return button({class:"btn-block interactive btn", role:"button", style:"width:100%"},
      // bxs-download
      i({ class: "bx bxs-file-pdf bx-tada font-Huge", style: "color: var(--color-gmail);"}),
      a({ class: "font-normalsize", href: localize_ui("pdf_cv_href"), target: "_blank"},
         () => localize_ui("cv")() + " " + localize_ui("pdf")(),
         // label({style: "display:block;"}, ),
         // label({style: "display:block;"}, ),
      ));
}

function RepositoryLink() {
   return button({class:"btn-block interactive btn font-large", role:"button"},
      a({href: "https://github.com/laralex/my_web_cv", target: "_blank"},
         i({ class: "bx bxl-github", style: "font-size:1.3rem;color: var(--color-github)"}),
         label(" "),
         label(localize_ui("web_cv_github")),
      ));
}

function MoreSkillsButton() {
   const isExpanded = van.state(false);
   return div({class: "badgescard"},
      button({
         class: "interactive btn font-Large expand-button",
         onclick: e => isExpanded.val = !isExpanded.val,
      }, i({class: () => isExpanded.val ? "bx bxs-up-arrow" : "bx bxs-down-arrow"}), i(" "), localize_ui("skills_title")),
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
            li(span("PyTorch, Docker, Qualcomm SNPE")),
            span("Unity, ARCore, Linux, LaTeX"),
            li(span(localize_ui("skills_languages_1"))),
            span(localize_ui("skills_languages_2")),
            ),
      ),
   )
}

function getBackgroundColorStyle(rgbString) {
   if (rgbString) {
      return { style: `background-color: ${rgbString};` };
   }
   return {};
}

function CvButton(labelId, rgbString, onclick) {
   let style = getBackgroundColorStyle(rgbString);
   return div({class: "cv-button"},
      button({
         class: "interactive btn font-Large expand-button",
         ...style,
         onclick: e => onclick(),
      }, i({class: () => "bx bxs-up-arrow"}, "\t"), localize_ui(labelId)),
   );
}

function CvChapter({titleElement, isDefaultActive, rgbString, onclick, extraClasses = "", insideConstructor = () => span(localize_ui("placeholder"))}) {
   let style = getBackgroundColorStyle(rgbString);
   return div({class: "cv-chapter " + extraClasses},
      button({
         class: "interactive btn font-Large expand-button ",
         ...style,
         onclick: e => { onclick(); },
      }, titleElement),
      div(
         {class: "inside", style: () => isDefaultActive.val ? "" : "display: none;"},
         div(insideConstructor)
      )
   );
}

function CvContent() {
   const chaptersData = [
      { id: "chapter_career", color: "#65E2E6", constructor: CvCareer },
      { id: "chapter_publications", color: "#A1EEBD", constructor: CvPublications },
      { id: "chapter_projects", color: "#F6F7C4", constructor: CvChapter },
      { id: "chapter_education", color: "#F6D6D6", constructor: CvEducation },
   ];
   const ids = chaptersData.map((x) => x.id);
   const colors = chaptersData.map((x) => x.color);
   const activeChapter = van.state(ids[0]);
   return div({class: "cv-content"},
      // CvButton("button_cv_begin", "#FFF", () => {
      //    activeChapter.val = ids[0];
      // }),
      () => ul(Array.from(chaptersData, (x) => {
         const isActive = van.derive(() => x.id == activeChapter.val);
         const onChapterActiveChange = () => {
            activeChapter.val = x.id;
         };
         const args = {
            titleElement: () => span({class: "bold"}, localize_ui(x.id)),
            isDefaultActive: isActive,
            rgbString: x.color,
            onclick: onChapterActiveChange,
         }
         const chapter = x.constructor(args);
         return chapter; }
      )),
      // CvButton("button_cv_end", "#FFF", () => {
      //    activeChapter.val = ids[ids.length - 1];
      // }),
   );
}

function CvCareer(chapterArgs) {
   const careersData = [
      { id: "career_huawei", color: "#65E2E6", constructor: CvChapter, icon: "../assets/huawei-2.svg" },
      { id: "career_samsung", color: "#7BD3EA", constructor: CvChapter, icon: "../assets/samsung.svg" },
      // { id: "career_freelance", color: "#65E2E6", constructor: CvChapter },
      // #64E1E5
   ];
   const ids = careersData.map((x) => x.id);
   const colors = careersData.map((x) => x.color);
   const activeCareer = van.state(ids[0]);
   return CvChapter({...chapterArgs,
      insideConstructor: () => ul(
         // CvButton("button_career_latest", "#FFF", () => {
         //    activeCareer.val = ids[0];
         // }),
         () => ul(Array.from(careersData, (x) => {
            const isActive = van.derive(() => x.id == activeCareer.val);
            const onChange = () => { activeCareer.val = x.id; };
            const titleElement = () => span(
               localize_ui(x.id),
               img({src: x.icon, style: "object-fit: contain;height:30px"})
            );
            const args = {
               titleElement: titleElement, isDefaultActive: isActive,
               rgbString: x.color, onclick: onChange};
            const chapter = x.constructor(args);
            return chapter;
         })),
         // CvButton("button_career_earliest", "#FFF", () => {
         //    activeCareer.val = ids[ids.length - 1];
         // })
      ),
      });
}

function CvPublications(chapterArgs) {
   const data = [
      { id: "publications_wacv_2024", color: "#A1EEBD", constructor: CvChapter },
      // #71BC8E #428D61 #428D61
   ];
   const ids = data.map((x) => x.id);
   const colors = data.map((x) => x.color);
   const active = van.state(ids[0]);
   return CvChapter({...chapterArgs,
      insideConstructor:() => ul(
         () => ul(Array.from(data, (x) => {
            const isActive = van.derive(() => x.id == active.val);
            const onChange = () => { active.val = x.id; };
            const args = {titleElement: localize_ui(x.id), isDefaultActive: isActive, rgbString: x.color, onclick: onChange};
            return x.constructor(args);
         })),
      ),
   });
}

function CvEducation(chapterArgs) {
   const data = [
      { id: "education_master", color: "#F6D6D6", constructor: CvChapter },
      { id: "education_bachelor", color: "#FFCEDD", constructor: CvChapter },
      // #FFC8F2
   ];
   const ids = data.map((x) => x.id);
   const colors = data.map((x) => x.color);
   const active = van.state(ids[0]);
   return CvChapter({...chapterArgs,
      insideConstructor: () => ul(
         () => ul(Array.from(data, (x) => {
            const isActive = van.derive(() => x.id == active.val);
            const onChange = () => {
               active.val = x.id;
            };
            const args = {titleElement: localize_ui(x.id), isDefaultActive: isActive, rgbString: x.color, onclick: onChange};
            return x.constructor(args);
         }))),
   });
}

function IntroPopup() {
   const closed = van.state(false);
   return () => closed.val ? null :div({class: "popup retro-box checkerboard-background"},
      div({class: "message font-LARGE"}, LanguagePicker(CURRENT_LANGUAGE, /* vertical */ false, van.state('en'), 'ui_language_intro')),
      span({class: "message bold font-LARGE"}, localize_ui("intro_hi")),
      span({class: "message"}, localize_ui("intro_enjoy_resume")),
      span({class: "message"}, localize_ui("intro_using")), // ""
      div({class: "flex-row wide"},
         div({class: "message flex-column", style: "margin-right: 1.5em;"},
            span(a({href: "https://www.rust-lang.org/", target: "_blank"}, "Rust"), " + ", a({href: "https://www.khronos.org/webgl/", target: "_blank"}, "WebGL2")),
            span(localize_ui("intro_3d")),
            div({class: "flex-row"},
               a({href: "https://www.rust-lang.org/", target: "_blank"},
                  img({src: "../assets/rust-plain.svg", height:"80"}, "Rust")),
               a({href: "https://www.khronos.org/webgl/", target: "_blank"},
                  img({src: "../assets/WebGL_Logo.svg", height:"60", style: "padding:7px;"}, "WebGL 2"))
            ),
         ),
         div({class: "message flex-column", style: "margin-left: 1.5em;"},
            span(a({href: "https://en.wikipedia.org/wiki/JavaScript", target: "_blank"}, "JavaScript"), " + ", a({href: "https://vanjs.org/", target: "_blank"}, "VanJS")),
            span(localize_ui("intro_frontend")),
            div({class: "flex-row"},
               a({href: "https://en.wikipedia.org/wiki/JavaScript", target: "_blank"},
                  img({src: "../assets/javascript-original.svg", height:"80", style: "padding:3px;"}, "JavaScript")),
               a({href: "https://vanjs.org/", target: "_blank"},
                  img({src: "../assets/vanjs.svg", height:"80", style: "padding:3px;"}, "VanJS")),
            ),
         ),
      ),
      div({class: "controls"},
         button({class: "btn popup-btn font-large", onclick: (e) => closed.val = true }, localize_ui("intro_close")))
   )
}

const addParallax = function({element, sensitivityXY, bgParallax, centerPx, centerBgPx}) {
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
       element.style.backgroundPosition = x;
       element.style.backgroundScale = "cover, 200%";
   }
}

function js_setup_canvas() {
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


van.add(document.getElementById("side-top__1"), LanguagePicker(CURRENT_LANGUAGE, /*vertical*/ true, van.state('en')));
van.add(document.getElementById("side-top__2"), GraphicsLevelPicker(CURRENT_GRAPHICS_LEVEL, /*vertical*/ true));
van.add(document.getElementById("side-links__1"), ResumePdfLink());
van.add(document.getElementById("side-links__2"), RepositoryLink());
van.add(document.getElementById("side-card"), MoreSkillsButton());
// van.add(document.getElementById("side-content"), CvChapter("chapter_career", "#7BD3EA"));
// van.add(document.getElementById("side-content"), CvChapter("chapter_publications", "#A1EEBD"));
// van.add(document.getElementById("side-content"), CvChapter("chapter_projects", "#F6F7C4"));
// van.add(document.getElementById("side-content"), CvChapter("chapter_education", "#F6D6D6"));
van.add(document.getElementById("sidebar"), CvContent());
if (!DEBUG) {
   van.add(document.body, IntroPopup());
}
const myPhoto = document.getElementById('my-photo');
if (ADD_PARALLAX) {
   // myPhoto.style.backgroundImage = "url(../assets/my_photo_tiny.png), url(../assets/bg6-min.png)";
   myPhoto.style.backgroundImage = "url(../assets/my_photo_tiny.png), url(../assets/bg10-min.png)";
   myPhoto.style.backgroundSize = "cover, 150%";
   // myPhoto.style.backgroundImage = "url(../assets/my_photo_tiny.png)";
   addParallax({
      element: myPhoto, sensitivityXY: [0.012, 0.007], bgParallax: 0.25,
      centerPx: [0, 0], centerBgPx: [-10, -5]});
} else {
   myPhoto.style.backgroundImage = "url(../assets/my_photo_tiny.png)";
}
js_setup_canvas();
wasm_startup();
wasm_loop(CANVAS_ID);