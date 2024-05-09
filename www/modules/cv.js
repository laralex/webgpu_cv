import { BUILD_DATA } from '../index.js';
import { DemoId } from '/wasm/index.js';
import { localizeString, localizeUi } from '/modules/localization.js';
import { Util } from '/modules/util.js';
import { CURRENT_GRAPHICS_SWITCHING_PROGRESS } from '/modules/exports_to_wasm.js';
import { localizeUiPostprocess } from './localization.js';

const {div, button, i, label, img, svg, path, input, details, summary, p, br, li, a, option, select, span, ul, h1, h2, h3} = van.tags
const SMOOTH = true;
// const CHAPTER_COLORS = ["#7BD3EA", "#A1EEBD", "#F6F7C4", "#F6D6D6", "#D57E7E"];
// const CHAPTER_COLORS = ["#B9BAC2", "#D4D2D5", "#BFAFA6", "#AA968A"];
// const CHAPTER_COLORS = ["#7474bf", "#667ac3", "#5680c6", "#4685c7", "#348ac7"];
// const CHAPTER_COLORS = ["#8f8fc2", "#8796c8", "#7f9ccd", "#77a3d0", "#70a9d2"];
const CHAPTER_COLORS = ["#a5a5e1", "#9bade8", "#91b4ee", "#87bcf3", "#7dc3f5"];
const CHAPTER_BORDER_COLORS = ["#9bade8", "#91b4ee", "#87bcf3", "#7dc3f5", "#7dc3f5"];
// const SUBCHAPTER_COLOR = ["#f2cc8f", "#FFCA93", "#FFC69E", "#FFC1B3"];
// const SUBCHAPTER_COLOR = ["#e8ac65", "#dc9c63", "#d08c60", "#c38862"];
// const SUBCHAPTER_COLOR = ["#CFEBE6", "#B2ECD5", "#A3EAB7", "#A6E590"];
// const SUBCHAPTER_COLOR = ["#87dceb", "#80d0f0", "#85c3f1", "#93b5ec", "#a5a5e1"];
// const SUBCHAPTER_COLOR = ["#a5a5e1", "#9bade8", "#91b4ee", "#87bcf3", "#7dc3f5"];
// const SUBCHAPTER_COLOR = ["#bec4d0", "#c5d1da", "#cedde2", "#d9e9ea", "#e6f4f1"];
// const SUBCHAPTER_COLOR = ["#e6f4f1", "#d9e9ea", "#cedde2", "#c5d1da" , "#bec4d0"];
const SUBCHAPTER_COLOR = ["#b0deff", "#b6d7f4", "#bbd0e8", "#bdcadc", "#bec4d0"];
const SUBCHAPTER_BORDER_COLOR = ["#b6d7f4", "#bbd0e8", "#bdcadc", "#bec4d0", "#bec4d0"];

const currentDate = new Date();
const huaweiSeniorEmploymentDate    = new Date(2023, 12 - 1, 7);
const samsungResignationDate        = new Date(2023, 11 - 1, 31);
const samsungMiddleEmploymentDate   = new Date(2022,  4 - 1, 26);
const samsungJuniorEmploymentDate   = new Date(2021,  9 - 1, 1);
const samsungInternResignationDate  = new Date(2021,  8 - 1, 1);
const samsungInternEmploymentDate   = new Date(2021,  6 - 1, 1);

const CHAPTERS_DEMOS = {
   "__stub__":                         DemoId.Stub,
   "career_huawei":                    DemoId.Fractal,
   "career_samsung":                   DemoId.FullBodyAvatar,
   "publications_wacv_2024":           DemoId.Fractal,
   "project_this_cv":                  DemoId.FullBodyAvatar,
   "project_will_and_reason":          DemoId.Fractal,
   "project_image_processing_tool":    DemoId.FullBodyAvatar,
   "education_master":                 DemoId.Fractal,
   "education_bachelor":               DemoId.Stub,
}

const DEMOS_DATA = {};
DEMOS_DATA[DemoId.Stub] = {description_id: "demo_STUB" };
DEMOS_DATA[DemoId.Triangle] = {description_id: "demo_triangle" };
DEMOS_DATA[DemoId.Fractal] = {description_id: "demo_fractal" };
DEMOS_DATA[DemoId.FrameGeneration] = {description_id: "demo_frame_generation" };
DEMOS_DATA[DemoId.HeadAvatar] = {description_id: "demo_head_avatar" };
DEMOS_DATA[DemoId.FullBodyAvatar] = {description_id: "demo_full_body_avatar"};
DEMOS_DATA[DemoId.ProceduralGeneration] = {description_id: "demo_procedural_generation" };

export function getDemoId(currentCvChapter) {
   return CHAPTERS_DEMOS[currentCvChapter] || CHAPTERS_DEMOS["__stub__"];
}

function getCssColor(colorString) {
   if (!colorString) {
      return "";
   }
   if (colorString[0] != '#') {
      colorString = `rgb(${colorString})`;
   }
   return colorString;
}

function getBackgroundColorStyle(bgValue, withBg=false, withBorder=false) {
   bgValue = getCssColor(bgValue);
   return (withBorder === true ? `border-color:${bgValue};` : "")
         + (withBg === true ? `background-color:${bgValue};` : "");
}

function formatYearsMonths({years, months, yearsFullWord = true, monthsFullWord = true}) {
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


function formatDateDiff(d1, d2, {yearsFullWord, monthsFullWord} = { yearsFullWord: true, monthsFullWord: true }) {
   const diff = Util.yearMonthDiff(Util.monthDiff(d1, d2));
   return formatYearsMonths({
     years: diff.yearDiff,
     months: diff.monthRemainder,
     yearsFullWord: yearsFullWord,
     monthsFullWord: monthsFullWord,
   });
}

export function DemoDescription(currentCvPage) {
   return () => {
      const demoData = DEMOS_DATA[getDemoId(currentCvPage[1].val)];
      const isHidden = CURRENT_GRAPHICS_SWITCHING_PROGRESS.val !== null
         || demoData === undefined || demoData.description_id === undefined;
      return isHidden ? div() : div({
         class: "demo-description" /*bubble shadow zmax */
      }, localizeUi(demoData.description_id));
   }
}

function Highlight(text) {
   return span({class: "bold"}, text)
}

function CvChapter({uniqueId, titleElement, isDefaultActive, bgValue, borderBgValue, onclick, extraClasses = "", extraActiveClasses = "", extraInsideClasses = "", insideConstructor = () => span(localizeUi("placeholder"))}) {
   const border = `background: ${borderBgValue}`;
   return div({id: uniqueId, class: () => "cv-chapter flex-column " + (BUILD_DATA.debug && !SMOOTH ? "" : " smooth ") + (isDefaultActive.val ? extraActiveClasses + " active " : " inactive ") + extraClasses},
      button({
         class: "cv-chapter-button btn expand-button ",
         style: "text-align: left; padding-left: 1rem;",
         onclick: e => { onclick(); },
      }, div({
         class:"bg",
         style: () => `background: ${bgValue};`,
      }),
      titleElement,
      div({
         class:"border",
         style: () => `background: ${borderBgValue};`
      })),
      div({
         class: () => extraInsideClasses + " inside flex-column "
      }, insideConstructor()),
   );
}

function CvChapterTitle(isActive, text, isCentered = true) {
   return span({class: () => " cv-title " + (isCentered ? " text-center " : " text-left ") + (isActive.val ? " bold " : "")}, text);
}

export function CvContent(currentCvPage, chaptersConnections) {
   const data = [
      { id: "chapter_career",       color: CHAPTER_COLORS[0], borderColor: CHAPTER_BORDER_COLORS[0], constructor: CvCareer },
      { id: "chapter_publications", color: CHAPTER_COLORS[1], borderColor: CHAPTER_BORDER_COLORS[1], constructor: CvPublications },
      { id: "chapter_projects",     color: CHAPTER_COLORS[2], borderColor: CHAPTER_BORDER_COLORS[2], constructor: CvProjects },
      { id: "chapter_education",    color: CHAPTER_COLORS[3], borderColor: CHAPTER_BORDER_COLORS[3], constructor: CvEducation },
   ];
   Object.keys(chaptersConnections).forEach(key => { delete chaptersConnections[key]; })
   for (let i = 0; i < data.length; i++) {
      if (!chaptersConnections[data[i].id]) {
         chaptersConnections[data[i].id] = {};
      }
      let curChapterCon = chaptersConnections[data[i].id];
      if (i > 0) {
         curChapterCon["__begin__"] = {prev: [data[i - 1].id, "__end__"]};
      } else {
         curChapterCon["__begin__"] = {prev: [data[i].id, "__begin__"]};
      }
      if (i < data.length - 1) {
         curChapterCon["__end__"] = {next: [data[i + 1].id, "__begin__"]};
      } else {
         curChapterCon["__end__"] = {next: [data[i].id, "__end__"]};
      }
   }
   return div({class: "cv-content flex-column"},
      Array.from(data, (x, i) => {
         const isActive = van.derive(() => x.id == currentCvPage[0].val);
         const onChapterActiveChange = () => {
            currentCvPage[0].val = x.id;
            currentCvPage[1].val = chaptersConnections[x.id]["__begin__"].next[1];
         };
         const borderBgValue = `${getCssColor(x.borderColor)}`;
         const bgValue = `linear-gradient(${getCssColor(x.color)}, ${getCssColor((data[i+1] || data[i]).color)})`;
         const chapterArgs = {
            uniqueId: x.id,
            titleElement: CvChapterTitle(isActive, localizeUi(x.id), /*center*/ false),
            extraActiveClasses: "vert-margin",
            extraClasses: "toplevel",
            isDefaultActive: isActive,
            bgValue: bgValue,
            borderBgValue: borderBgValue,
            onclick: onChapterActiveChange,
         }
         const chapter = x.constructor(currentCvPage[1], chaptersConnections[x.id], x.id, chapterArgs);
         return chapter;
      }),
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

   const huaweiSpan = Util.yearMonthDiff(Util.monthDiff(huaweiSeniorEmploymentDate, currentDate));
   const samsungSpan = Util.yearMonthDiff([
      Util.monthDiff(samsungInternEmploymentDate, samsungInternResignationDate),
      Util.monthDiff(samsungJuniorEmploymentDate, samsungMiddleEmploymentDate),
      Util.monthDiff(samsungMiddleEmploymentDate, samsungResignationDate),
   ].reduce((prev, curr) => prev + curr));
   const data = [
      { id: "career_huawei",  color: SUBCHAPTER_COLOR[0], borderColor: SUBCHAPTER_BORDER_COLOR[0], 
         employmentSpan: huaweiSpan, constructor: CvHuawei },
      { id: "career_samsung", color: SUBCHAPTER_COLOR[1], borderColor: SUBCHAPTER_BORDER_COLOR[1], 
         employmentSpan: samsungSpan, constructor: CvSamsung },
      // { id: "career_freelance", color: "#65E2E6", constructor: CvChapter },
      // #64E1E5
   ];
   populateConnections(chapterConnections, chapterId, data.map(x => x.id));

   return CvChapter({...chapterArgs,
      insideConstructor: () => {
         return Array.from(data, (x, i) => {
            const isActive = van.derive(() => x.id == currentCvPage.val);
            const onChange = () => { currentCvPage.val = x.id; };
            const bgValue = `linear-gradient(${getCssColor(x.color)}, ${getCssColor((data[i+1] || data[i]).color)})`;
            const args = {
               uniqueId: x.id,
               extraInsideClasses: "cv-text", extraClasses: "toplevel",
               titleElement: () => {
                  const employmentSpan = formatYearsMonths({
                     years: x.employmentSpan.yearDiff,
                     months: x.employmentSpan.monthRemainder,
                     yearsFullWord: false,
                     monthsFullWord: true,
                  });
                  return CvChapterTitle(isActive, localizeString(x.id)().text + ", " + employmentSpan.yearStr + " " + employmentSpan.monthStr);
               },
               isDefaultActive: isActive,
               bgValue: bgValue, borderBgValue: x.borderColor,
               onclick: onChange};
            const chapter = x.constructor(args);
            return chapter;
         });
      }});
}

function CvPublications(currentCvPage, chapterConnections, chapterId, chapterArgs) {
   const data = [
      { id: "publications_wacv_2024", color: SUBCHAPTER_COLOR[0], borderColor: SUBCHAPTER_BORDER_COLOR[0], constructor: CvWacv2024 },
      // #71BC8E #428D61 #428D61
   ];
   populateConnections(chapterConnections, chapterId, data.map(x => x.id));
   return CvChapter({...chapterArgs,
      insideConstructor: () => {
         // if (!currentCvPage.val) {
         //    currentCvPage.val = data[0].id;
         // }
         return Array.from(data, (x, i) => {
            const isActive = van.derive(() => x.id == currentCvPage.val);
            const onChange = () => { currentCvPage.val = x.id; };
            const bgValue = `linear-gradient(${getCssColor(x.color)}, ${getCssColor((data[i+1] || data[i]).color)})`;
            const args = {
               uniqueId: x.id, extraInsideClasses: "cv-text", extraClasses: "toplevel",
               titleElement: CvChapterTitle(isActive, localizeUi(x.id)),
               isDefaultActive: isActive, bgValue: bgValue,
               borderBgValue: x.borderColor, onclick: onChange};
            return x.constructor(args);
         });
      }
   });
}

function CvProjects(currentCvPage, chapterConnections, chapterId, chapterArgs) {
   const data = [
      { id: "project_this_cv", color: SUBCHAPTER_COLOR[0], borderColor: SUBCHAPTER_BORDER_COLOR[0], constructor: CvProjectWebcv },
      { id: "project_image_processing_tool", color: SUBCHAPTER_COLOR[1], borderColor: SUBCHAPTER_BORDER_COLOR[1], constructor: CvProjectTreesRuler },
      { id: "project_will_and_reason", color: SUBCHAPTER_COLOR[2], borderColor: SUBCHAPTER_BORDER_COLOR[2], constructor: CvProjectWillAndReason },
      // { id: "project_infinite_fractal"       , color: SUBCHAPTER_COLOR[2], borderColor: SUBCHAPTER_BORDER_COLOR[2], constructor: CvChapter },
      // #FFB993
   ];
   populateConnections(chapterConnections, chapterId, data.map(x => x.id));
   return CvChapter({...chapterArgs,
      insideConstructor: () => {
         // if (!currentCvPage.val) {
         //    currentCvPage.val = data[0].id;
         // }
         return Array.from(data, (x, i) => {
            const isActive = van.derive(() => x.id == currentCvPage.val);
            const onChange = () => { currentCvPage.val = x.id; };
            const bgValue = `linear-gradient(${getCssColor(x.color)}, ${getCssColor((data[i+1] || data[i]).color)})`;
            const args = {
               uniqueId: x.id, extraInsideClasses: "cv-text", extraClasses: "toplevel",
               titleElement: CvChapterTitle(isActive, localizeUi(x.id)),
               isDefaultActive: isActive, bgValue: bgValue,
               borderBgValue: x.borderColor, onclick: onChange};
            return x.constructor(args);
         });
      }
   });
}

function CvEducation(currentCvPage, chapterConnections, chapterId, chapterArgs) {
   const data = [
      { id: "education_master"   , color: SUBCHAPTER_COLOR[0], borderColor: SUBCHAPTER_BORDER_COLOR[0], constructor: CvMaster },
      { id: "education_bachelor" , color: SUBCHAPTER_COLOR[1], borderColor: SUBCHAPTER_BORDER_COLOR[1], constructor: CvBachelor },
      // #FFC8F2
   ];
   populateConnections(chapterConnections, chapterId, data.map(x => x.id));
   return CvChapter({...chapterArgs,
      insideConstructor: () => {
         return Array.from(data, (x, i) => {
            const isActive = van.derive(() => x.id == currentCvPage.val);
            const onChange = () => { currentCvPage.val = x.id; };
            const bgValue = `linear-gradient(${getCssColor(x.color)}, ${getCssColor((data[i+1] || data[i]).color)})`;
            const args = {
               uniqueId: x.id, extraInsideClasses: "cv-text",
               titleElement: CvChapterTitle(isActive, localizeUi(x.id)),
               isDefaultActive: isActive, bgValue: bgValue,
               borderBgValue: x.borderColor, onclick: onChange};
            return x.constructor(args);
         });
      }
   });
}

function CvHuawei(chapterArgs) {
   chapterArgs.insideConstructor = () => {
      return div({class: "font-normalsize"},
         div({class: "flex-row flex-center", style: "margin-bottom: 0.5rem;"},
            YearsBlock(Util.getYearsSpan(huaweiSeniorEmploymentDate, currentDate).concat([localizeUi("current")])),
            div({class: "flex-column header"},
               div({class: "flex-row", style: "gap:0.9rem;margin-bottom: 0.5rem;"},
                  img({id: "cv-huawei-logo", src: "../assets/huawei-small.svg"}),
                  p({class: "font-Large bold"}, localizeUi("huawei_job_title")),
               ),
               LeftRightAlignedList({
                  leftItems: [ () => p(localizeUi("senior_engineer")), ],
                  rightItems: [ () => {
                     const seniorStr = formatDateDiff(huaweiSeniorEmploymentDate, currentDate);
                     return p(seniorStr.yearStr + " " + seniorStr.monthStr);
                  }],
               }),
            )
         ),
         // div({class: "icons"},
         //    img({class: "huge", src: "https://cdn.jsdelivr.net/gh/devicons/devicon/icons/opengl/opengl-original.svg" }),
         //    img({src: "https://cdn.jsdelivr.net/gh/devicons/devicon@latest/icons/cplusplus/cplusplus-plain.svg" }),
         //    img({class: "huge", src: "https://cdn.jsdelivr.net/gh/devicons/devicon/icons/unity/unity-original-wordmark.svg" }),
         // ),
         ul(
            li(Highlight("OpenGL / C++: "), localizeUi("huawei_hooks")),
            li(Highlight("Unity / C# / C++: "), localizeUi("huawei_unity_ff_sdk")),
            li(localizeUi("huawei_ohos")),
         ),
       )
   }
   return CvChapter(chapterArgs);
}

function CvSamsung(chapterArgs) {
   chapterArgs.insideConstructor = () => {
      return div({class: "font-normalsize"},
         div({class: "flex-row flex-center", style: "margin-bottom: 0.5rem;"},
         YearsBlock(Util.getYearsSpan(samsungInternEmploymentDate, samsungResignationDate)),
         div({class: "flex-column header"},
            div({class: "flex-row", style: "gap:1rem;margin-bottom: 0.5rem;"},
               img({id: "cv-samsung-logo", src: "../assets/samsung.svg"}),
               p({class: "font-Large bold"}, localizeUi("samsung_job_title")),
            ),
               LeftRightAlignedList({
                  leftItems: [
                     () => p(localizeUi("middle_engineer")),
                     () => p(localizeUi("junior_engineer")),
                     () => p(localizeUi("intern_engineer")),],
                  rightItems: [
                     () => {
                        const middleStr = formatDateDiff(samsungMiddleEmploymentDate, samsungResignationDate, {yearsFullWord: false, monthsFullWord: false});
                        return p(middleStr.yearStr + " " + middleStr.monthStr); // p("1 yr 8 mos"),
                     },
                     () => {
                        const juniorStr = formatDateDiff(samsungJuniorEmploymentDate, samsungMiddleEmploymentDate);
                        return p(juniorStr.yearStr + " " + juniorStr.monthStr); // p("7 months"),
                     },
                     () => {
                        const internStr = formatDateDiff(samsungInternEmploymentDate, samsungInternResignationDate);
                        return p(internStr.yearStr + " " + internStr.monthStr); // p("2 months"),
                     }
                  ],
               }),
            )
         ),
         // div({class: "icons"},
         //    span("OpenGL"),
         //    span("Python"),
         //    span("PyTorch"),
         //    span("Java"),
         //    span("Unity"),
         //    // img({class: "large", src: "https://cdn.jsdelivr.net/gh/devicons/devicon/icons/opengl/opengl-original.svg" }),
         //    // img({src: "https://cdn.jsdelivr.net/gh/devicons/devicon/icons/python/python-original-wordmark.svg"}),
         //    // img({class: "large", src: "https://cdn.jsdelivr.net/gh/devicons/devicon/icons/pytorch/pytorch-original-wordmark.svg"}),
         //    // img({src: "https://cdn.jsdelivr.net/gh/devicons/devicon/icons/java/java-original-wordmark.svg" }),
         //    // img({class: "large", src: "https://cdn.jsdelivr.net/gh/devicons/devicon/icons/unity/unity-original-wordmark.svg" }),
         // ),
         ul(
            li(Highlight("OpenGL / Java / Android: "), localizeUi("samsung_ar_avatars"), ul(
                li(localizeUi("samsung_rendering_nn")),
                li(localizeUi("samsung_animation_system")),
                li(localizeUi("samsung_arcore")),
                )),
            li(Highlight("Unity / C#: "), localizeUi("samsung_unity")),
            li(Highlight("Python / PyTorch: "), localizeUi("samsung_wacv"), a({"href": "https://openaccess.thecvf.com/content/WACV2024/html/Bashirov_MoRF_Mobile_Realistic_Fullbody_Avatars_From_a_Monocular_Video_WACV_2024_paper.html"}, localizeUi("link"))),
            li(localizeUi("samsung_kpi")),
         )
      );
   };
   return CvChapter(chapterArgs);
}

function CvWacv2024(chapterArgs) {
   chapterArgs.insideConstructor = () => {
      return div({class: "font-normalsize"},
         div({class: "flex-column flex-center header", style: "margin-bottom:0.5rem;gap:0.5rem"},
            p({class: "italic"}, "2024 Winter Conference on Applications of Computer Vision"),
            p({class: "font-Large bold"}, "MoRF: Mobile Realistic Fullbody Avatars From a Monocular Video"),
            // div({class: "flex-row flex-wrap flex-center", style: "gap: 0 1rem;"},
            //    span("Renat Bashirov"),
            //    span({class: "bold"}, "*Alexey Larionov*"),
            //    span("Evgeniya Ustinova"),
            //    span("Mikhail Sidorenko"),
            //    span("David Svitov"),
            //    span("Ilya Zakharkin"),
            //    span("Victor Lempitsky"),
            // ),
         ),

         div({class: "flex-row flex-center", style: "margin-bottom:0.5rem;gap:1rem;"},
            span(
               span(localizeUi("project_page")), ": ",
               a({"href": "https://samsunglabs.github.io/MoRF-project-page/"}, localizeUi("link"))
            ),
            span(
               span(localizeUi("proceedings")), ": ",
               a({"href": "https://openaccess.thecvf.com/content/WACV2024/html/Bashirov_MoRF_Mobile_Realistic_Fullbody_Avatars_From_a_Monocular_Video_WACV_2024_paper.html"}, localizeUi("link"))
            ),
            span(
               span("arXiv"), ": ",
               a({"href": "https://arxiv.org/abs/2303.10275"}, localizeUi("link"))
            ),
         ),
         // LeftRightAlignedList({
         //       leftItems: [
         //          () => p("Project page"),
         //          () => p("Proceedings"),
         //          () => p("Arxiv"),
         //       ],
         //       rightItems: [
         //          () => a({"href": "https://samsunglabs.github.io/MoRF-project-page/"}, "samsunglabs.github.io/MoRF-project-page"),
         //          () => a({"href": "https://openaccess.thecvf.com/content/WACV2024/html/Bashirov_MoRF_Mobile_Realistic_Fullbody_Avatars_From_a_Monocular_Video_WACV_2024_paper.html"}, "link"),
         //          () => a({"href": "https://arxiv.org/abs/2303.10275"}, "link"),
         //       ],
         // }),
         p(Highlight(localizeUi("abstract")), ": ", localizeUi("wacv2024_abstract")),
         ul({style: "margin-top:0.5rem;"},
            li(localizeUi("wacv2024_author")),
            li(localizeUi("wacv2024_morphing")),
            li(localizeUi("wacv2024_demo")),
            li(localizeUi("wacv2024_text")),
         ),
      )
   };
   return CvChapter(chapterArgs);
}

function CvProjectWebcv(chapterArgs) {
   chapterArgs.insideConstructor = () => {
      return div({class: "font-normalsize"},
         div({class: "flex-row flex-center", style: "margin-bottom:0.5rem;gap:1.0rem;"},
            YearsBlock([localizeUiPostprocess("months_many_full", (s) => "6 " + s), "2024"]),
            div({class: "flex-column", style:"width:min-content;flex-grow:0.5;"},
               localizeUi("webcv_author"),
               a({href: "https://creativecommons.org/licenses/by/4.0/legalcode.en", target:"_blank", rel:"license noopener noreferrer", style: "display:inline-block;"}, localizeUi("webcv_license"), "CC BY 4.0"),
            ),
            div({class: "flex-column"},
               img({
                  src: "third_party/boxicons-2.1.4/svg/logos/bxl-github.svg",
                  style: "filter: var(--filter-github);",
                  width: "40",
               }),
               div(
                  // span("Source code"), ": ",
                  a({"href": "https://github.com/laralex/my_web_cv"}, localizeUi("webcv_repo"))
               ),
            ),
            div({class: "flex-column"},
               img({
                  src: "./third_party/boxicons-2.1.4/svg/solid/bxs-file-pdf.svg",
                  style: "filter: var(--filter-gmail)",
                  width: "40",
               }),
               div(
                  // span("PDF CV"), ": ",
                  a({"href": localizeUi("pdf_cv_href")}, "PDF CV")
               ),
            ),
            
         ),
         ul(li(localizeUi("webcv_scratch")),
            li(localizeUi("webcv_demos"),
               ul(
                  li(localizeUi("webcv_made_with"), Highlight("Rust, WebAssembly, WebGPU")),
                  li(localizeUi("webcv_loading"), a({href: "https://developer.mozilla.org/en-US/docs/Web/API/window/requestAnimationFrame"}, "requestAnimationFrame"), " API")
               ),
            ),
            li(localizeUi("webcv_web"),
               ul(
                  li(localizeUi("webcv_plain"), Highlight(" JS, HTML, CSS "), localizeUi("webcv_vanjs")),
                  li(localizeUi("webcv_wheel")),
                  li(localizeUi("webcv_easy")),
               )
            ),
            li(localizeUi("webcv_deploy")),
         ),
      )
   }
   return CvChapter(chapterArgs);
}

function CvProjectTreesRuler(chapterArgs) {
   chapterArgs.insideConstructor = () => {
      return div({class: "font-normalsize"},
         div({class: "flex-row flex-center", style: "margin-bottom:0.5rem;gap:0.8rem;"},
            YearsBlock([localizeUiPostprocess("month_full", (s) => "1 " + s), "2023"]),
            div({class: "flex-column", style: "width:min-content;flex-grow:0.5;"},
               localizeUi("webcv_author"),
               a({href: "https://github.com/laralex/TreesRuler?tab=MIT-1-ov-file", target:"_blank", rel:"license noopener noreferrer", style: "display:inline-block;"}, localizeUi("webcv_license"), "MIT"),
            ),
            div({class: "flex-column"},
               img({
                  src: "third_party/boxicons-2.1.4/svg/logos/bxl-github.svg",
                  style: "filter: var(--filter-github);",
                  width: "40",
               }),
               div(
                  // span("Source code"), ": ",
                  a({"href": "https://github.com/laralex/TreesRuler"}, localizeUi("webcv_repo"))
               ),
            ),
            div({class: "flex-column"},
               img({
                  src: "./third_party/boxicons-2.1.4/svg/regular/bx-link.svg",
                  style: "filter: var(--filter-gmail)",
                  width: "40",
               }),
               div(
                  // span("PDF CV"), ": ",
                  a({"href": localizeUi("treesruler_href")}, localizeUi("link"))
               ),
            ),
         ),
         // Highlight("Description: "),
         p({style: "margin-bottom:0.5rem;"}, localizeUi("treesruler_tool"), " ", span({class:"bold"}, localizeUi("treesruler_collect"))),
         p(localizeUi("treesruler_audience")),
         ul(
            li(localizeUi("treesruler_adjust")),
            li(localizeUi("treesruler_yaml")),
            li(localizeUi("treesruler_grouping")),
            li(localizeUi("treesruler_settings")),
         ),
         p({style: "margin-top:0.5rem;"}, localizeUi("treesruler_libs"), Highlight("JavaScript / p5.js"))
      )
   }
   return CvChapter(chapterArgs);
}

function CvProjectWillAndReason(chapterArgs) {
   chapterArgs.insideConstructor = () => {
      return div({class: "font-normalsize"},
         div({class: "flex-row flex-center", style: "margin-bottom:0.5rem;gap:0.8rem;"},
            YearsBlock([localizeUiPostprocess("months_full", (s) => "4 " + s), "2017"]),
            // div({class: "flex-column"},
            //    "Volunteering",
            //    // a({href: "https://github.com/laralex/TreesRuler?tab=MIT-1-ov-file", target:"_blank", rel:"license noopener noreferrer", style: "display:inline-block;"}, "License: MIT"),
            // ),
            div({class: "flex-column"},
               img({
                  src: "./third_party/boxicons-2.1.4/svg/regular/bx-link.svg",
                  style: "filter: var(--filter-gmail)",
                  width: "40",
               }),
               div(
                  // span("Source code"), ": ",
                  a({"href": "https://vk.com/willreason"}, localizeUi("willreason_page"))
               ),
            ),
            div({class: "flex-column"},
               img({
                  src: "./third_party/boxicons-2.1.4/svg/solid/bxs-video.svg",
                  style: "filter: var(--filter-gmail)",
                  width: "40",
               }),
               div(
                  a({"href": "https://vk.com/video/@willreason"}, localizeUi("willreason_videos"))
               )
            ),
         ),
         // Highlight("Description: "),
         p({class: "flex-row flex-center font-Large bold"}, "Will & Reason"),
         p({class: "flex-row flex-center font-large italic"}, localizeUi("willreason_strategy")),
         p({style: "margin-top:0.5rem;"}, localizeUi("willreason_volunteer")),
         ul(
            li(localizeUi("willreason_pathfinding")),
            li(localizeUi("willreason_ui")),
            li(localizeUi("willreason_game_design")),
         ),
         p({style: "margin-top:0.5rem;"}, localizeUi("willreason_libs"), Highlight("Unity / C#"))
      )
   }
   return CvChapter(chapterArgs);
}

function CvMaster(chapterArgs) {
   chapterArgs.insideConstructor = () => {
      return div({class: "font-normalsize"},
         div({class: "flex-row flex-center", style: "margin-bottom: 0.5rem;"},
            YearsBlock(Util.getYearsSpan(new Date(2020, 0), new Date(2022, 0))),
            div({class: "flex-column", style: "align-items:center; gap:0.2rem;"},
               p({class: "font-Large"}, localizeUi("master_title")),
               LeftRightAlignedList({leftItems: [ () => p(localizeUi("master_honors")), ], rightItems: [ () => p(localizeUi("master_gpa")), ], }),
               div({style: "height:0.5rem;"}),
               img({id: "cv-skoltech-logo", src: "../assets/Skoltech_Logo.svg"}),
               p(localizeUi("master_university")),
            )
         ),
         // div({class: "icons"},
         //    img({src: "https://cdn.jsdelivr.net/gh/devicons/devicon/icons/python/python-original-wordmark.svg"}),
         //    img({class: "huge", src: "https://cdn.jsdelivr.net/gh/devicons/devicon/icons/pytorch/pytorch-original-wordmark.svg"}),
         //    img({class: "huge", src: "https://cdn.jsdelivr.net/gh/devicons/devicon/icons/unity/unity-original-wordmark.svg" }),
         // ),
         ul(
            li(localizeUi("master_research"), a({href: "https://scholar.google.com/citations?user=gYYVokYAAAAJ&hl=en"}, localizeUi("master_victor"))),
            li(localizeUi("master_thesis")),
            li(localizeUi("master_courses"), ul(li(localizeUi("master_deeplearning")),li(localizeUi("master_3dcv")),li(localizeUi("master_parallel")),li(localizeUi("master_unity")),)
            ),
         ),
       )
   }
   return CvChapter(chapterArgs);
}

function CvBachelor(chapterArgs) {
   chapterArgs.insideConstructor = () => {
      return div({class: "font-normalsize"},
         div({class: "flex-row flex-center", style: "margin-bottom: 0.5rem;"},
            // YearsBlock([2016, '•••', 2020]),
            YearsBlock(Util.getYearsSpan(new Date(2016, 0), new Date(2020, 0))),
            div({class: "flex-column", style: "align-items:center; gap:0.2rem;"},
               p({class: "font-Large"}, localizeUi("bachelor_title")),
               LeftRightAlignedList({leftItems: [ () => p(localizeUi("master_honors")), ], rightItems: [ () => p(localizeUi("master_gpa")), ], }),
               div({style: "height:0.5rem;"}),
               img({id: "cv-polytech-logo", src: "../assets/polytech_logo_small.svg"}),
               p(localizeUi("bachelor_university")),
            )
         ),
         // div({class: "icons"},
         //    img({src: "https://cdn.jsdelivr.net/gh/devicons/devicon@latest/icons/cplusplus/cplusplus-plain.svg" }),
         //    img({src: "https://cdn.jsdelivr.net/gh/devicons/devicon@latest/icons/csharp/csharp-plain.svg" }),
         //    img({src: "https://cdn.jsdelivr.net/gh/devicons/devicon/icons/python/python-original-wordmark.svg"}),
         //    img({class: "huge", src: "https://cdn.jsdelivr.net/gh/devicons/devicon@latest/icons/oracle/oracle-original.svg" }),
         // ),
         ul(
            li(localizeUi("bachelor_austria")),
            li(localizeUi("bachelor_icpc")),
            li(localizeUi("bachelor_badge")),
            li(localizeUi("bachelor_thesis")),
            li(localizeUi("master_courses"), ul(li(localizeUi("bachelor_math")),li(localizeUi("bachelor_architecture")),li(localizeUi("master_parallel")),li(localizeUi("bachelor_oracle")))),
         ),
       )
   }
   return CvChapter(chapterArgs);
}

function YearsBlock(years, totalDuration1, totalDuration2) {
   return div({class: "years-block font-Large"},
      totalDuration1 ? span({class: "year-duration"}, totalDuration1) : null,
      totalDuration2 ? span({class: "year-duration"}, totalDuration2) : null,
      years.flatMap((year, i) =>
         [
            span({class: 'year-value'}, year),
            i < years.length - 1 ? span({class: 'year-separator'}) : null
         ]).reverse());
}

function LeftRightAlignedList({leftItems, rightItems, separator=() => span("·")}) {
   console.assert(leftItems.length === rightItems.length);
   return div({class: "flex-row left-right-aligned"},
      div({class: "flex-column", style: "align-items:end;"}, leftItems, ),
      div({class: "flex-column", style: "margin: 0 0.5rem 0 0.5rem; align-items:center;"}, 
         Array(leftItems.length).fill(separator)),
      div({class: "flex-column", style: "align-items:start;"}, rightItems),
   )
}