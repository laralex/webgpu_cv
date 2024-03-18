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
const huaweiSeniorEmploymentDate = new Date(2023, 12 - 1, 7);
const samsungResignationDate = new Date(2023, 11 - 1, 31);
const samsungMiddleEmploymentDate = new Date(2022, 4 - 1, 26);
const samsungJuniorEmploymentDate = new Date(2021, 9 - 1, 1);
const samsungInternResignationDate = new Date(2021, 8 - 1, 1);
const samsungInternEmploymentDate = new Date(2021, 6 - 1, 1);

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

function CvButton(labelId, bgValue, onclick) {
   let bg = getBackgroundColorStyle(bgValue);
   return div({class: "cv-button"},
      button({
         class: "interactive btn font-Large expand-button",
         style: bg,
         onclick: e => onclick(),
      }, i({class: () => "bx bxs-up-arrow"}, "\t"), span(localizeUi(labelId))),
   );
}

function CvChapter({uniqueId, titleElement, isDefaultActive, bgValue, borderBgValue, onclick, extraClasses = "", extraActiveClasses = "", extraInsideClasses = "", insideConstructor = () => span(localizeUi("placeholder"))}) {
   const border = `background: ${borderBgValue}`;
   return div({id: uniqueId, class: () => "cv-chapter flex-column " + (DEBUG && !SMOOTH ? "" : " smooth ") + (isDefaultActive.val ? extraActiveClasses + " active " : " inactive ") + extraClasses},
      button({
         class: "cv-chapter-head btn expand-button ",
         style: "text-align: left; padding-left: 1rem;",
         //style: () => getBackgroundColorStyle(bgValue, false, false),
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
         class: () => extraInsideClasses + " inside flex-column " /* + (isDefaultActive.val ? " active " : " inactive ") */,
         //style: () => getBackgroundColorStyle(bgValue, false, false) /* + ` box-shadow: inset 1em 0 0 0 rgb(${bgValue});` */,
      }, insideConstructor()),
   );
}

function CvChapterTitle(isActive, text, isCentered = true) {
   return span({class: () => " cv-title " + (isCentered ? " text-center " : " text-left ") + (isActive.val ? " bold " : "")}, text);
}

function CvContent(currentCvPage, chaptersConnections) {
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
         // if (!currentCvPage.val) {
         //    currentCvPage.val = data[0].id;
         // }
         return Array.from(data, (x, i) => {
            const isActive = van.derive(() => x.id == currentCvPage.val);
            const onChange = () => { currentCvPage.val = x.id; };
            const bgValue = `linear-gradient(${getCssColor(x.color)}, ${getCssColor((data[i+1] || data[i]).color)})`;
            const args = {
               uniqueId: x.id,
               extraInsideClasses: "cv-text", extraClasses: "toplevel",
               titleElement: () => {
                  const employmentSpan = Util.formatYearsMonths({
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
         // CvButton("button_career_earliest", "#FFF", () => {
         //    activeCareer.val = ids[ids.length - 1];
         // })
      }});
}

function CvPublications(currentCvPage, chapterConnections, chapterId, chapterArgs) {
   const data = [
      { id: "publications_wacv_2024", color: SUBCHAPTER_COLOR[0], borderColor: SUBCHAPTER_BORDER_COLOR[0], constructor: CvSamsung },
      { id: "russian"               , color: SUBCHAPTER_COLOR[1], borderColor: SUBCHAPTER_BORDER_COLOR[1], constructor: CvChapter },
      { id: "english"               , color: SUBCHAPTER_COLOR[2], borderColor: SUBCHAPTER_BORDER_COLOR[2], constructor: CvChapter },
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
      { id: "this_cv"                     , color: SUBCHAPTER_COLOR[0], borderColor: SUBCHAPTER_BORDER_COLOR[0], constructor: CvSamsung },
      { id: "unity_4X_strategy_volunteer" , color: SUBCHAPTER_COLOR[1], borderColor: SUBCHAPTER_BORDER_COLOR[1], constructor: CvChapter },
      { id: "image_processing_tool"       , color: SUBCHAPTER_COLOR[2], borderColor: SUBCHAPTER_BORDER_COLOR[2], constructor: CvChapter },
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
         // if (!currentCvPage.val) {
         //    currentCvPage.val = data[0].id;
         // }
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
   const seniorStr = Util.formatDateDiff(huaweiSeniorEmploymentDate, currentDate);
   chapterArgs.insideConstructor = () => {
      return div({class: "font-normalsize"},
         div({class: "flex-row flex-center", style: "margin-bottom: 0.5rem;"},
            YearsBlock(Util.getYearsSpan(huaweiSeniorEmploymentDate, currentDate).concat(["Current"])),
            div({class: "flex-column", style: "align-items:center; gap:0.5rem;"},
               img({id: "cv-huawei-logo", src: "../assets/huawei-small.svg"}),
               p({class: "font-Large bold"}, "Frame prediction SDK for mobile games"),
               LeftRightAlignedList({
                  leftItems: [ p("Senior engineer"), ],
                  rightItems: [ p(seniorStr.yearStr + " " + seniorStr.monthStr), ],
               }),
            )
         ),
         div({class: "icons"},
            img({class: "huge", src: "https://cdn.jsdelivr.net/gh/devicons/devicon/icons/opengl/opengl-original.svg" }),
            img({src: "https://cdn.jsdelivr.net/gh/devicons/devicon@latest/icons/cplusplus/cplusplus-plain.svg" }),
            img({class: "huge", src: "https://cdn.jsdelivr.net/gh/devicons/devicon/icons/unity/unity-original-wordmark.svg" }),
         ),
         ul(
            li("Researched OpenGL hook-based rendering speed up in \"Genshin Impact\" game"),
            li("Integrated in-house frame prediction SDK as a Unity plugin (URP pipeline)"),
            li("Proudly helped to develop the next generation mobile operating system OpenHarmony"),
         ),
       )
   }
   return CvChapter(chapterArgs);
}

function CvSamsung(chapterArgs) {
   const internStr = Util.formatDateDiff(samsungInternEmploymentDate, samsungInternResignationDate, {yearsFullWord: false, monthsFullWord: false});
   const juniorStr = Util.formatDateDiff(samsungJuniorEmploymentDate, samsungMiddleEmploymentDate);
   const middleStr = Util.formatDateDiff(samsungMiddleEmploymentDate, samsungResignationDate);
   chapterArgs.insideConstructor = () => {
      return div({class: "font-normalsize"},
         div({class: "flex-row flex-center", style: "margin-bottom: 0.5rem;"},
            YearsBlock(Util.getYearsSpan(samsungInternEmploymentDate, samsungResignationDate)),
            div({class: "flex-column", style: "align-items:center;gap: 0.5rem;"},
               img({id: "cv-samsung-logo", src: "../assets/samsung.svg"}),
               p({class: "font-Large bold"}, "Neural Networks R&D"),
               LeftRightAlignedList({
                  leftItems: [p("Middle engineer"), p("Junior engineer"), p("Intern"),],
                  rightItems: [
                     p(middleStr.yearStr + " " + middleStr.monthStr), // p("1 yr 8 mos"),
                     p(juniorStr.yearStr + " " + juniorStr.monthStr), // p("7 months"),
                     p(internStr.yearStr + " " + internStr.monthStr), // p("2 months"),
                  ],
               }),
            )
         ),
         div({class: "icons"},
            img({class: "large", src: "https://cdn.jsdelivr.net/gh/devicons/devicon/icons/opengl/opengl-original.svg" }),
            // img({src: "https://cdn.jsdelivr.net/gh/devicons/devicon/icons/cplusplus/cplusplus-line.svg" }),
            // img({src: "https://cdn.jsdelivr.net/gh/devicons/devicon/icons/rust/rust-plain.svg" }),
            img({src: "https://cdn.jsdelivr.net/gh/devicons/devicon/icons/python/python-original-wordmark.svg"}),
            img({class: "large", src: "https://cdn.jsdelivr.net/gh/devicons/devicon/icons/pytorch/pytorch-original-wordmark.svg"}),
            img({src: "https://cdn.jsdelivr.net/gh/devicons/devicon/icons/java/java-original-wordmark.svg" }),
            img({class: "large", src: "https://cdn.jsdelivr.net/gh/devicons/devicon/icons/unity/unity-original-wordmark.svg" }),
            // img({src: "https://cdn.jsdelivr.net/gh/devicons/devicon/icons/docker/docker-original-wordmark.svg"}),
         ),
         ul(
            li("Solely created an Android techdemo to render realistic human avatars, with:", ul(
                li("implementation with plain Java/OpenGL"),
                li("rendering via neural networks, running 60 FPS in 512x512px on Qualcomm NPU"),
                li("augmented reality via ARCore"),
                li("my animation system and skinning"),
                li("also ported into Unity application"),
            )),
            li("Completed a crucial KPI, being a solo developer on a research project"),
            li("Published a paper on neural networks, at WACV 2024 conference: ", a({"href": "https://openaccess.thecvf.com/content/WACV2024/html/Bashirov_MoRF_Mobile_Realistic_Fullbody_Avatars_From_a_Monocular_Video_WACV_2024_paper.html"}, "Link")),
         ),
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
               p({class: "font-Large"}, "MSc of Information Science"),
               LeftRightAlignedList({leftItems: [ p("with Honors"), ], rightItems: [ p("GPA 5/5"), ], }),
               img({id: "cv-skoltech-logo", src: "../assets/Skoltech_Logo.svg"}),
               p("Skolkovo Institute of Science & Technology"),
            )
         ),
         div({class: "icons"},
            img({src: "https://cdn.jsdelivr.net/gh/devicons/devicon/icons/python/python-original-wordmark.svg"}),
            img({class: "huge", src: "https://cdn.jsdelivr.net/gh/devicons/devicon/icons/pytorch/pytorch-original-wordmark.svg"}),
            img({class: "huge", src: "https://cdn.jsdelivr.net/gh/devicons/devicon/icons/unity/unity-original-wordmark.svg" }),
         ),
         ul(
            li("Researched augmented and virtual reality based on neural networks, supervised by renowned ", a({href: "https://scholar.google.com/citations?user=gYYVokYAAAAJ&hl=en"}, "Dr. Victor Lempitsky")),
            li("Thesis on real-time rendering via neural networks on mobile hardware"),
            li("Took courses on:", ul(li("Machine Learning / Deep Learning"),li("3D Computer Vision"),li("Parallel computing"),li("Unity game engine"),)
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
            YearsBlock(Util.getYearsSpan(new Date(2016, 0), new Date(2020, 0))),
            div({class: "flex-column", style: "align-items:center; gap:0.2rem;"},
               p({class: "font-Large"}, "BSc of Computer Science"),
               LeftRightAlignedList({leftItems: [ p("with Honors"), ], rightItems: [ p("GPA 5/5"), ], }),
               img({id: "cv-polytech-logo", src: "../assets/polytech_logo_small.svg"}),
               p("Peter The Great St. Petersburg Polytechnic University"),
            )
         ),
         div({class: "icons"},
            img({src: "https://cdn.jsdelivr.net/gh/devicons/devicon@latest/icons/cplusplus/cplusplus-plain.svg" }),
            img({src: "https://cdn.jsdelivr.net/gh/devicons/devicon@latest/icons/csharp/csharp-plain.svg" }),
            img({src: "https://cdn.jsdelivr.net/gh/devicons/devicon/icons/python/python-original-wordmark.svg"}),
            img({class: "huge", src: "https://cdn.jsdelivr.net/gh/devicons/devicon@latest/icons/oracle/oracle-original.svg" }),
         ),
         ul(
            li("Winner of project marathon jointly with TU Graz (Austria); visited Austria"),
            li("ICPC world olympiad, twice half-finalist"),
            li("\"Student of the year\" badge"),
            li("Thesis on procedural generation of 3D meshes for buildings"),
            li("Took courses on:", ul(li("Core math"),li("Computer Architecture"),li("Oracle Database  administration"))),
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
   return div({class: "flex-row"},
      div({class: "flex-column", style: "align-items:end;"}, leftItems, ),
      div({class: "flex-column", style: "margin: 0 0.5rem 0 0.5rem; align-items:center;"}, 
         Array(leftItems.length).fill(separator)),
      div({class: "flex-column", style: "align-items:start;"}, rightItems),
   )
}