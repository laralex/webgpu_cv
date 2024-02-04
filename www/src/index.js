import { wasm_startup, wasm_loop, wasm_resize } from "my-wasm";
const {div, button, i, label, img, svg, path, input, li, a, option, select, span, ul, h1, h2, h3} = van.tags

const CURRENT_LANGUAGE = van.state("en");
const ADD_PARALLAX = true;
const DEBUG = true;
// const DEBUG = false;
const CANVAS_ID = "main-canvas";
const UI_STRINGS = getLocalization()
const CURRENT_GRAPHICS_LEVEL = van.state("medium");
const CURRENT_CV_PAGE = van.state({ level1: "chapter_publications", level2: {
   "chapter_publications": van.state("publications_wacv_2024"),
}});
const CV_PAGE_ORDER = {}

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

function LanguagePicker(currentLanguage, isVertical, tooltipLanguage=undefined, tooltipLabelId='ui_language') {
   const LANGUAGES = {
      en: {labelId: 'english', icon: "../assets/flag_GB.png", emoji: "🇬🇧"},
      ru: {labelId: 'russian', icon: "../assets/flag_RU.png", emoji: "🇷🇺"},
      kr: {labelId: 'korean', icon: "../assets/flag_RU.png", emoji: "🇰🇷"},
   }
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
         oninput: e => currentLanguage.val = e.target.value,
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
         () => localizeUi(meta.labelId)() + " " +  meta.emoji));
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
      a({ class: "font-normalsize", href: localizeUi("pdf_cv_href"), target: "_blank"},
         () => localizeUi("cv")() + " " + localizeUi("pdf")(),
         // label({style: "display:block;"}, ),
         // label({style: "display:block;"}, ),
      ));
}

function RepositoryLink() {
   return button({class:"btn-block interactive btn font-large", role:"button", style: "width:100%;"},
      a({href: "https://github.com/laralex/my_web_cv", target: "_blank"},
         i({ class: "bx bxl-github", style: "font-size:1.3rem;color: var(--color-github)"}),
         label(" "),
         label(localizeUi("web_cv_github")),
      ));
}

function PersonalCard() {
   return div({class: "profileinfo"},
      h1({class: "font-LARGE bold"}, localizeUi("name_surname")),
      h3(localizeUi("specialty")),
      div(() => ul(
         li(localizeUi("specialty_computer_graphics")),
         (localizeUi("specialty_deep_learning", /*nullIfMissing*/ true) ? li(localizeUi("specialty_deep_learning")) : null),
      )),
      div({class: "techlist"},
         (["C++", "Python", "OpenGL", "WebGL", "Android"]
            .map(text => div({class: "badge"}, text))),
      ),
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
            li(span("PyTorch, Docker, Qualcomm SNPE")),
            span("Unity, ARCore, Linux, LaTeX"),
            li(span(localizeUi("skills_languages_1"))),
            span(localizeUi("skills_languages_2")),
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
      }, i({class: () => "bx bxs-up-arrow"}, "\t"), localizeUi(labelId)),
   );
}

function CvChapter({titleElement, isDefaultActive, rgbString, onclick, extraClasses = "", insideConstructor = () => span(localizeUi("placeholder"))}) {
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

function CvContent(currentCvPage, chaptersConnections) {
   const chaptersData = [
      { id: "chapter_career", color: "#65E2E6", constructor: CvCareer },
      { id: "chapter_publications", color: "#A1EEBD", constructor: CvPublications },
      { id: "chapter_projects", color: "#F6F7C4", constructor: CvEducation },
      { id: "chapter_education", color: "#F6D6D6", constructor: CvEducation },
   ];
   Object.keys(chaptersConnections).forEach(key => { delete chaptersConnections[key]; })
   for (let i = 0; i < chaptersData.length; i++) {
      if (!chaptersConnections[chaptersData[i].id]) {
         chaptersConnections[chaptersData[i].id] = {};
      }
      let curChapterCon = chaptersConnections[chaptersData[i].id];
      if (i > 0) {
         curChapterCon["__begin__"] = {prev: [chaptersData[i - 1].id, "__end__"]};
      } else {
         curChapterCon["__begin__"] = {prev: [chaptersData[i].id, "__begin__"]};
      }
      if (i < chaptersData.length - 1) {
         curChapterCon["__end__"] = {next: [chaptersData[i + 1].id, "__begin__"]};
      } else {
         curChapterCon["__end__"] = {next: [chaptersData[i].id, "__end__"]};
      }
   }
   return div({class: "cv-content"},
      () => ul(Array.from(chaptersData, (x) => {
         const isActive = van.derive(() => x.id == currentCvPage.val.level1);
         const onChapterActiveChange = () => {
            currentCvPage.val = { level1: x.id, level2: currentCvPage.val.level2 };
         };
         const chapterArgs = {
            titleElement: () => span({class: "bold"}, localizeUi(x.id)),
            isDefaultActive: isActive,
            rgbString: x.color,
            onclick: onChapterActiveChange,
         }
         if (!(x.id in currentCvPage.val.level2)) {
            currentCvPage.val.level2[x.id] = van.state();
         }
         const chapter = x.constructor(currentCvPage.val.level2[x.id], chaptersConnections[x.id], x.id, chapterArgs);
         return chapter;
      })),
   );
}

function populateConnections(destinationConnections, chapterId, subchapterIds) {
   destinationConnections["__begin__"].next = [chapterId, subchapterIds[0]];
   for (let i = 0; i < subchapterIds.length; i++) {
      if (!destinationConnections[subchapterIds[i]]) {
         destinationConnections[subchapterIds[i]] = {
            next: [chapterId, i < subchapterIds.length - 1 ? subchapterIds[i + 1] : "__end__"],
            prev: [chapterId, i > 0 ? subchapterIds[i - 1] : "__begin__"],
         };
      }
   }
   destinationConnections["__end__"].prev = [chapterId, subchapterIds[subchapterIds.length - 1]];
}

function CvCareer(currentCvPage, chapterConnections, chapterId, chapterArgs) {
   const data = [
      { id: "career_huawei", color: "#65E2E6", constructor: CvChapter, icon: "../assets/huawei-2.svg" },
      { id: "career_samsung", color: "#7BD3EA", constructor: CvChapter, icon: "../assets/samsung.svg" },
      // { id: "career_freelance", color: "#65E2E6", constructor: CvChapter },
      // #64E1E5
   ];
   populateConnections(chapterConnections, chapterId, data.map(x => x.id));
   if (!currentCvPage.val) {
      currentCvPage.val = data[0].id;
   }
   return CvChapter({...chapterArgs,
      insideConstructor: () => ul(
         // CvButton("button_career_latest", "#FFF", () => {
         //    activeCareer.val = ids[0];
         // }),
         () => ul(Array.from(data, (x) => {
            const isActive = van.derive(() => x.id == currentCvPage.val);
            const onChange = () => { currentCvPage.val = x.id; };
            const titleElement = () => span(
               localizeUi(x.id),
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

function CvPublications(currentCvPage, chapterConnections, chapterId, chapterArgs) {
   const data = [
      { id: "publications_wacv_2024", color: "#A1EEBD", constructor: CvChapter },
      { id: "russian", color: "#A1EEBD", constructor: CvChapter },
      { id: "english", color: "#A1EEBD", constructor: CvChapter },
      // #71BC8E #428D61 #428D61
   ];
   populateConnections(chapterConnections, chapterId, data.map(x => x.id));
   if (!currentCvPage.val) {
      currentCvPage.val = data[0].id;
   }
   return CvChapter({...chapterArgs,
      insideConstructor:() => ul(
         () => ul(Array.from(data, (x) => {
            const isActive = van.derive(() => x.id == currentCvPage.val);
            const onChange = () => { currentCvPage.val = x.id; };
            const args = {titleElement: localizeUi(x.id), isDefaultActive: isActive, rgbString: x.color, onclick: onChange};
            return x.constructor(args);
         })),
      ),
   });
}

function CvEducation(currentCvPage, chapterConnections, chapterId, chapterArgs) {
   const data = [
      { id: "education_master", color: "#F6D6D6", constructor: CvChapter },
      { id: "education_bachelor", color: "#FFCEDD", constructor: CvChapter },
      // #FFC8F2
   ];
   populateConnections(chapterConnections, chapterId, data.map(x => x.id));
   if (!currentCvPage.val) {
      currentCvPage.val = data[0].id;
   }
   return CvChapter({...chapterArgs,
      insideConstructor: () => ul(
         () => ul(Array.from(data, (x) => {
            const isActive = van.derive(() => x.id == currentCvPage.val);
            const onChange = () => { currentCvPage.val = x.id; };
            const args = {titleElement: localizeUi(x.id), isDefaultActive: isActive, rgbString: x.color, onclick: onChange};
            return x.constructor(args);
         }))),
   });
}

function IntroPopup({onclose}) {
   const closed = van.state(false);
   van.derive(() => { if (closed.val && onclose) onclose(); });
   return () => closed.val ? null :div({class: "popup font-large retro-box checkerboard-background"},
      div({class: "message font-Large"}, LanguagePicker(CURRENT_LANGUAGE, /* vertical */ false, van.state('en'), 'ui_language_intro')),
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

function addParallax({element, sensitivityXY, bgParallax, centerPx, centerBgPx}) {
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

function changeCvPage(nextOrPrev) {
   console.assert(["next", "prev"].includes(nextOrPrev));
   const curL1 = CURRENT_CV_PAGE.val.level1;
   const curL2 = CURRENT_CV_PAGE.val.level2[curL1].val;
   let [nextL1, nextL2] = CV_PAGE_ORDER[curL1][curL2][nextOrPrev];
   let maxJumps = 100;
   while (maxJumps-- && ["__begin__", "__end__"].includes(nextL2)) {
      [nextL1, nextL2] = CV_PAGE_ORDER[nextL1][nextL2][nextOrPrev];
   }
   console.log('!!!', curL2, nextOrPrev, nextL2);
   CURRENT_CV_PAGE.val.level2[nextL1].val = nextL2;
   CURRENT_CV_PAGE.val = { level1: nextL1, level2: CURRENT_CV_PAGE.val.level2};
}

van.add(document.getElementById("side-top__1"), LanguagePicker(CURRENT_LANGUAGE, /*vertical*/ false, van.state('en')));
van.add(document.getElementById("side-top__2"), GraphicsLevelPicker(CURRENT_GRAPHICS_LEVEL, /*vertical*/ false));
van.add(document.getElementById("side-links__1"), ResumePdfLink());
van.add(document.getElementById("side-links__2"), RepositoryLink());
van.add(document.getElementById("side-card"), MoreSkillsButton());
document.querySelectorAll(".firstinfo").forEach(element => {
   // console.log(element);
   van.add(element, PersonalCard())
});
// van.add(document.getElementById("side-content"), CvChapter("chapter_career", "#7BD3EA"));
// van.add(document.getElementById("side-content"), CvChapter("chapter_publications", "#A1EEBD"));
// van.add(document.getElementById("side-content"), CvChapter("chapter_projects", "#F6F7C4"));
// van.add(document.getElementById("side-content"), CvChapter("chapter_education", "#F6D6D6"));
van.add(document.getElementById("sidebar"), CvContent(CURRENT_CV_PAGE, CV_PAGE_ORDER));
console.log('@@@', CV_PAGE_ORDER);
if (!DEBUG) {
   van.add(document.body, IntroPopup({onclose: () => {
      van.add(document.body, ControlsPopup({}));
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
   myPhoto.style.backgroundImage = "url(../assets/my_photo_tiny.png), url(../assets/bg10-min.png)";
   myPhoto.style.backgroundSize = "cover, 150%";
   // myPhoto.style.backgroundImage = "url(../assets/my_photo_tiny.png)";
   addParallax({
      element: myPhoto, sensitivityXY: [0.012, 0.007], bgParallax: 0.25,
      centerPx: [0, 0], centerBgPx: [-10, -5]});
} else {
   myPhoto.style.backgroundImage = "url(../assets/my_photo_tiny.png)";
}

// listen to "scroll" event
const computScrollSpeed = Util.computeScrollSpeed();
window.addEventListener('wheel', event => {
   let wheelSpeed = computScrollSpeed(event);
   changeCvPage(wheelSpeed > 0 ? "next" : "prev");
},  { capture: true });
js_setup_canvas();
wasm_startup();
wasm_loop(CANVAS_ID);