const {div, button, i, label, img, svg, path, input, details, summary, p, br, li, a, option, select, span, ul, h1, h2, h3} = van.tags
const SMOOTH = true;
const CHAPTER_COLORS = ["#e8ac65", "#dc9c63", "#d08c60", "#c38862"];
const SUBCHAPTER_COLOR = "#f2cc8f";

function getCssColor(colorString) {
   if (!colorString) {
      return "";
   }
   if (colorString[0] != '#') {
      colorString = `rgb(${colorString})`;
   }
   return colorString;
}

function getBackgroundColorStyle(rgbString, withBg=false, withBorder=false) {
   rgbString = getCssColor(rgbString);
   return (withBorder === true ? `border-color:${rgbString};` : "")
         + (withBg === true ? `background-color:${rgbString};` : "");
}

function CvButton(labelId, rgbString, onclick) {
   let bg = getBackgroundColorStyle(rgbString);
   return div({class: "cv-button"},
      button({
         class: "interactive btn font-Large expand-button",
         style: bg,
         onclick: e => onclick(),
      }, i({class: () => "bx bxs-up-arrow"}, "\t"), localizeUi(labelId)),
   );
}

function CvChapter({uniqueId, titleElement, isDefaultActive, rgbString, onclick, extraClasses = "", extraActiveClasses = "", extraInsideClasses = "", insideConstructor = () => span(localizeUi("placeholder"))}) {
   return div({id: uniqueId, class: () => "cv-chapter flex-column " + (DEBUG && !SMOOTH ? "" : " smooth ") + (isDefaultActive.val ? extraActiveClasses + " active " : " inactive ") + extraClasses},
      button({
         class: "cv-chapter-head btn font-Large expand-button ",
         //style: () => getBackgroundColorStyle(rgbString, false, false),
         onclick: e => { onclick(); },
      }, div({class:"wrappee", style: ()=>`box-shadow: inset ${isDefaultActive.val ? 60 : 3}rem 0 ${getCssColor(rgbString)};`}, titleElement)),
      div({
         class: () => extraInsideClasses + " inside flex-column " /* + (isDefaultActive.val ? " active " : " inactive ") */,
         //style: () => getBackgroundColorStyle(rgbString, false, false) /* + ` box-shadow: inset 1em 0 0 0 rgb(${rgbString});` */,
      }, insideConstructor()),
   );
}

function CvContent(currentCvPage, chaptersConnections) {
   const chaptersData = [
      { id: "chapter_career",       color: CHAPTER_COLORS[0], constructor: CvCareer },
      { id: "chapter_publications", color: CHAPTER_COLORS[1], constructor: CvPublications },
      { id: "chapter_projects",     color: CHAPTER_COLORS[2], constructor: CvProjects },
      { id: "chapter_education",    color: CHAPTER_COLORS[3], constructor: CvEducation },
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
   return div({class: "cv-content flex-column"},
      Array.from(chaptersData, (x) => {
         const isActive = van.derive(() => x.id == currentCvPage[0].val);
         const onChapterActiveChange = () => {
            currentCvPage[0].val = x.id;
            currentCvPage[1].val = chaptersConnections[x.id]["__begin__"].next[1];
         };
         const chapterArgs = {
            uniqueId: x.id,
            titleElement: span({class: () => isActive.val ? " bold " : ""}, localizeUi(x.id)),
            extraActiveClasses: "vert-margin",
            isDefaultActive: isActive,
            rgbString: x.color,
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
   const data = [
      { id: "career_huawei", color: SUBCHAPTER_COLOR, constructor: CvChapter, icon: "../assets/huawei-2.svg" },
      { id: "career_samsung", color: SUBCHAPTER_COLOR, constructor: CvSamsung, icon: "../assets/samsung.svg" },
      // { id: "career_freelance", color: "#65E2E6", constructor: CvChapter },
      // #64E1E5
   ];
   populateConnections(chapterConnections, chapterId, data.map(x => x.id));

   return CvChapter({...chapterArgs,
      insideConstructor: () => {
         // if (!currentCvPage.val) {
         //    currentCvPage.val = data[0].id;
         // }
         return Array.from(data, (x) => {
            const isActive = van.derive(() => x.id == currentCvPage.val);
            const onChange = () => { currentCvPage.val = x.id; };
            const titleElement = span({class: () => isActive.val ? " bold " : ""}, localizeUi(x.id));
            const args = {
               uniqueId: x.id, extraInsideClasses: "cv-text",
               titleElement: titleElement, isDefaultActive: isActive,
               rgbString: x.color, onclick: onChange};
            const chapter = x.constructor(args);
            return chapter;
         });
         // CvButton("button_career_earliest", "#FFF", () => {
         //    activeCareer.val = ids[ids.length - 1];
         // })
      }});
}

function CvSamsung(chapterArgs) {
   chapterArgs.insideConstructor = () => {
      return div({class: "font-small"},
         div({class: "flex-row flex-center"},
            YearsBlock([2021, 2022, 2023]),
            div({class: "flex-column", style: "align-items:center;"},
               p({class: "font-Large"}, "Neural Networks R&D"),
               p("Middle engineer (1 yr 9 mos)"),
               p("Junior engineer (9 months)"),
               p("Intern (2 months)"),
            )
         ),
         div({class: "icons"},
            img({src: "https://cdn.jsdelivr.net/gh/devicons/devicon/icons/opengl/opengl-original.svg" }),
            // img({src: "https://cdn.jsdelivr.net/gh/devicons/devicon/icons/cplusplus/cplusplus-line.svg" }),
            // img({src: "https://cdn.jsdelivr.net/gh/devicons/devicon/icons/rust/rust-plain.svg" }),
            img({src: "https://cdn.jsdelivr.net/gh/devicons/devicon/icons/python/python-original-wordmark.svg"}),
            img({src: "https://cdn.jsdelivr.net/gh/devicons/devicon/icons/pytorch/pytorch-original-wordmark.svg"}),
            // img({src: "https://cdn.jsdelivr.net/gh/devicons/devicon/icons/csharp/csharp-original.svg" }),
            img({src: "https://cdn.jsdelivr.net/gh/devicons/devicon/icons/java/java-original-wordmark.svg" }),
            img({src: "https://cdn.jsdelivr.net/gh/devicons/devicon/icons/unity/unity-original-wordmark.svg" }),
            // img({src: "https://cdn.jsdelivr.net/gh/devicons/devicon/icons/docker/docker-original-wordmark.svg"}),
         ),
         ul(
            li("Solely developed an Android/Java/OpenGL techdemo, with realistic human avatars rendered via deferred neural rendering"),
            li("Achieved 60 FPS neural rendering in 512px resolution on mobile, by running on Qualcomm NPU with optimal data flow from GPU"),
            li("Ported the Android techdemo as Unity app with augmented reality"),
            li("Completed a crucial KPI, being the only production developer on a research project"),
            li("Researched the neural network architecture and published a paper on WACV 2024 conference: ", a({"href": "https://openaccess.thecvf.com/content/WACV2024/html/Bashirov_MoRF_Mobile_Realistic_Fullbody_Avatars_From_a_Monocular_Video_WACV_2024_paper.html"}, "Link")),
         ),
       )
   }
   return CvChapter(chapterArgs);
}

function CvPublications(currentCvPage, chapterConnections, chapterId, chapterArgs) {
   const data = [
      { id: "publications_wacv_2024", color: SUBCHAPTER_COLOR, constructor: CvSamsung },
      { id: "russian", color: SUBCHAPTER_COLOR, constructor: CvChapter },
      { id: "english", color: SUBCHAPTER_COLOR, constructor: CvChapter },
      // #71BC8E #428D61 #428D61
   ];
   populateConnections(chapterConnections, chapterId, data.map(x => x.id));
   return CvChapter({...chapterArgs,
      insideConstructor: () => {
         // if (!currentCvPage.val) {
         //    currentCvPage.val = data[0].id;
         // }
         return Array.from(data, (x) => {
            const isActive = van.derive(() => x.id == currentCvPage.val);
            const onChange = () => { currentCvPage.val = x.id; };
            const args = {
               uniqueId: x.id, extraInsideClasses: "cv-text",
               titleElement: span({class: () => isActive.val ? " bold " : ""}, localizeUi(x.id)),
               isDefaultActive: isActive, rgbString: x.color, onclick: onChange};
            return x.constructor(args);
         });
      }
   });
}

function CvProjects(currentCvPage, chapterConnections, chapterId, chapterArgs) {
   const data = [
      { id: "this_cv", color: SUBCHAPTER_COLOR, constructor: CvSamsung },
      { id: "unity_4X_strategy_volunteer", color: SUBCHAPTER_COLOR, constructor: CvChapter },
      { id: "image_processing_tool", color: SUBCHAPTER_COLOR, constructor: CvChapter },
      // #FFB993
   ];
   populateConnections(chapterConnections, chapterId, data.map(x => x.id));
   return CvChapter({...chapterArgs,
      insideConstructor: () => {
         // if (!currentCvPage.val) {
         //    currentCvPage.val = data[0].id;
         // }
         return Array.from(data, (x) => {
            const isActive = van.derive(() => x.id == currentCvPage.val);
            const onChange = () => { currentCvPage.val = x.id; };
            const args = {
               uniqueId: x.id, extraInsideClasses: "cv-text",
               titleElement: span({class: () => isActive.val ? " bold " : ""}, localizeUi(x.id)),
               isDefaultActive: isActive, rgbString: x.color, onclick: onChange};
            return x.constructor(args);
         });
      }
   });
}

function CvEducation(currentCvPage, chapterConnections, chapterId, chapterArgs) {
   const data = [
      { id: "education_master", color: SUBCHAPTER_COLOR, constructor: CvChapter },
      { id: "education_bachelor", color: SUBCHAPTER_COLOR, constructor: CvSamsung },
      // #FFC8F2
   ];
   populateConnections(chapterConnections, chapterId, data.map(x => x.id));
   return CvChapter({...chapterArgs,
      insideConstructor: () => {
         // if (!currentCvPage.val) {
         //    currentCvPage.val = data[0].id;
         // }
         return Array.from(data, (x) => {
            const isActive = van.derive(() => x.id == currentCvPage.val);
            const onChange = () => { currentCvPage.val = x.id; };
            const args = {
               uniqueId: x.id, extraInsideClasses: "cv-text",
               titleElement: span({class: () => isActive.val ? " bold " : ""}, localizeUi(x.id)),
               isDefaultActive: isActive, rgbString: x.color, onclick: onChange};
            return x.constructor(args);
         });
      }
   });
}

function YearsBlock(years, totalDuration1, totalDuration2) {
   return div({class: "years-block font-Large"},
      totalDuration1 ? span({class: "font-normalsize bold years-duration"}, totalDuration1) : null,
      totalDuration2 ? span({class: "font-normalsize bold years-duration"}, totalDuration2) : null,
      years.flatMap((year, i) => i < years.length - 1 ?
         [span(year), span({class: 'year-separator'})]
         : span(year)).reverse());
}