const {div, button, i, label, img, svg, path, input, details, summary, p, li, a, option, select, span, ul, h1, h2, h3} = van.tags

function getBackgroundColorStyle(rgbString, withBg=false, withBorder=false) {
   if (rgbString) {
      return (withBorder === true ? `border-color:rgb(${rgbString});` : "")
         + (withBg === true ? `background-color:rgb(${rgbString});` : "");
   }
   return "";
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
   return div({id: uniqueId, class: () => "cv-chapter flex-column " + (DEBUG ? "" : " smooth ") + (isDefaultActive.val ? extraActiveClasses + " active " : " inactive ") + extraClasses},
      button({
         class: "cv-chapter-head btn font-Large expand-button ",
         style: () => getBackgroundColorStyle(rgbString, false, false),
         onclick: e => { onclick(); },
      }, div({class:"wrappee", style: ()=>`box-shadow: inset ${isDefaultActive.val ? 60 : 3}rem 0 rgb(${rgbString});`}, titleElement)),
      div({
         class: () => extraInsideClasses + " inside flex-column " /* + (isDefaultActive.val ? " active " : " inactive ") */,
         style: () => getBackgroundColorStyle(rgbString, false, false) /* + ` box-shadow: inset 1em 0 0 0 rgb(${rgbString});` */,
      }, insideConstructor()),
   );
}

function CvContent(currentCvPage, chaptersConnections) {
   const chaptersData = [
      { id: "chapter_career", color: "101, 226, 230", constructor: CvCareer },
      { id: "chapter_publications", color: "61, 238, 189", constructor: CvPublications },
      { id: "chapter_projects", color: "246, 247, 196", constructor: CvProjects },
      { id: "chapter_education", color: "246, 214, 214", constructor: CvEducation },
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
      { id: "career_huawei", color: "101, 226, 230", constructor: CvChapter, icon: "../assets/huawei-2.svg" },
      { id: "career_samsung", color: "123, 211, 234", constructor: CvSamsung, icon: "../assets/samsung.svg" },
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
         p("This detail element requires the use of the height and transition attributes to open smoothly. The downside to this is that it's less flexible since you must know how long the content will be in order to display content properly"),
         p("Max-height will only work for opening the collapsible, but not for closing it. Closure using max-height will cause the element to close mostly like the default version, but slightly collapse at the end. This is due to changes to the <code>&lt;slot&gt;</code> made when opening and closing the details."),
         p("The slot's inline styling is changed to include content-visibility: hidden, which functions similarly to display: none in that it just removes the content from the layout and removes visibility."),
         p("the slot belongs to the user-agent shadow DOM, and is seemingly unable to be targeted using the slot or ::slotted() CSS selectors. Need to do more research"),
         p("the slot belongs to the user-agent shadow DOM, and is seemingly unable to be targeted using the slot or ::slotted() CSS selectors. Need to do more research"),
         p("the slot belongs to the user-agent shadow DOM, and is seemingly unable to be targeted using the slot or ::slotted() CSS selectors. Need to do more research"),
         p("the slot belongs to the user-agent shadow DOM, and is seemingly unable to be targeted using the slot or ::slotted() CSS selectors. Need to do more research"),
       )
   }
   return CvChapter(chapterArgs);
}

function CvPublications(currentCvPage, chapterConnections, chapterId, chapterArgs) {
   const data = [
      { id: "publications_wacv_2024", color: "161, 238, 189", constructor: CvSamsung },
      { id: "russian", color: "161, 238, 189", constructor: CvChapter },
      { id: "english", color: "161, 238, 189", constructor: CvChapter },
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
      { id: "this_cv", color: "246, 247, 196", constructor: CvSamsung },
      { id: "unity_4X_strategy_volunteer", color: "255, 230, 168", constructor: CvChapter },
      { id: "image_processing_tool", color: "255, 209, 150", constructor: CvChapter },
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
      { id: "education_master", color: "246, 214, 214", constructor: CvChapter },
      { id: "education_bachelor", color: "255, 206, 221", constructor: CvSamsung },
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