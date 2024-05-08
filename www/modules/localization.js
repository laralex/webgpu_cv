const {span} = van.tags
export const CURRENT_LANGUAGE = van.state("en");
export const UI_STRINGS = (function getLocalization() {
   return {
      placeholder: {en: "<TODO>", ru: "<TODO>", kr: "<TODO>"},
      english_en: {en: "English", ru: "English", kr: "English"},
      russian_en: {en: "Russian", ru: "Russian", kr: "Russian"},
      korean_en: {en: "Korean", ru: "Korean", kr: "Korean"},
      russian: {en: "Russian", ru: "Русский", kr: "러시아어" },
      korean: {en: "Korean", ru: "Корейский", kr: "한국어" },
      graphics_minimal: {en: "Minimal", ru: "Минимум", kr: "최소" },
      graphics_low: {en: "Low    ", ru: "Низкое ", kr: "낮음" },
      graphics_medium: {en: "Medium", ru: "Среднее", kr: "중간" },
      graphics_high: {en: "High  ", ru: "Высокое", kr: "높음" },
      graphics_ultra: {en: "Ultra ", ru: "Ультра", kr: "최대" },
      ui_language: {en: "Language", ru: "Language / Язык", kr: "Language / 언어"},
      ui_language_intro: {en: "Language", ru: "Language / Язык", kr: "Language / 언어"},
      graphics_levels: {en: "Graphics quality", ru: "Качество графики", kr: "그래픽 품질"},
      name_surname: {en: "Alexey Larionov", ru: "Алексей Ларионов", kr: "료샤 라리오노브"},
      job_title: {en: "Graphics Software Engineer", ru: "Программист графики", kr: "그래픽 프로그래머"},
      specialty_computer_graphics: {en: "Optimization, quality of graphics", ru: "Оптимизация и качество графики", kr: "저는 3D 그래픽 성능와 딥러닝 연구을 일합니다."},
      specialty_deep_learning: {en: "Deep Learning R&D", ru: "R&D нейронных сетей", /*no korean kr: ""*/},
      cv: {en: "CV", ru: "CV", kr: "이력서"},
      pdf: {en: "in PDF", ru: "в PDF", kr: "PDF"},
      pdf_cv_href: {en: "./assets/__softlink_cv_eng.pdf", ru: "./assets/__softlink_cv_rus.pdf", kr: "./assets/__softlink_cv_eng.pdf"},
      trees_ruler_href: {en: "./trees_ruler/eng", ru: "./trees_ruler/rus", kr: "./trees_ruler/eng"},
      web_cv_github: {en: "Source code", ru: "Исходный код", kr: "데모 소스 코드"},
      clear_cookies_button: {en: "Reload CV", ru: "Сбросить сайт", kr: "설정을 초기화"},
      skills_title: {en: "Extra skills", ru: "Прочие компетенции", kr: "다른 기술들"},
      skills_languages_1: {en: "English\xa0%ENG%\xa0(C1), Russian\xa0%RUS%\xa0(N), Korean\xa0%KOR%\xa0(A2), Polish\xa0%POL%\xa0(A1)", ru: "Английский\xa0C1\xa0%ENG%, Корейский\xa0A2\xa0%KOR%, Польский\xa0A1\xa0%POL%, Русский\xa0%RUS%", kr: "영어\xa0%ENG%\xa0(C1), 한국어\xa0%KOR%\xa0(А2), 폴란드어\xa0%POL%\xa0(А1), 러시아\xa0사람\xa0%RUS%"},
      chapter_career: {en: "Career", ru: "Карьера", kr: "경력"},
      chapter_publications: {en: "Publications", ru: "Публикации", kr: "연구 출판"},
      chapter_projects: {en: "Projects", ru: "Проекты", kr: "프로젝트"},
      chapter_education: {en: "Education", ru: "Образование", kr: "교육"},
      button_cv_begin: {en: "To beginning", ru: "В начало", kr: "시작에"},
      button_cv_end: {en: "To ending", ru: "В конец", kr: "끝에"},
      button_career_latest: {en: "Latest position", ru: "К вершине карьеры", kr: "최신 직업"},
      button_career_earliest: {en: "First position", ru: "К началу карьеры", kr: "첫 직업"},
      month_short:      {en: " mos"    , ru: " мес"      , kr: "월"},
      month_full:       {en: " months" , ru: " месяц"    , kr: "월"},
      months_full:      {en: " months" , ru: " месяца"   , kr: "월"},
      months_many_full: {en: " months" , ru: " месяцев"  , kr: "월"},
      year_short:       {en: " yr"     , ru: " г"        , kr: "년"},
      year_full:        {en: " year"   , ru: " год"      , kr: "년"},
      years_full:       {en: " years"  , ru: " года"     , kr: "년"},
      years_many_full:  {en: " years"  , ru: " лет"      , kr: "년"},
      career_huawei: {en: "Huawei", ru: "Huawei", kr: "Huawei"},
      career_samsung: {en: "Samsung AI Center", ru: "Samsung AI Center", kr: "삼성 AI 연구센터"},
      career_samsung: {en: "Samsung AI Center", ru: "Samsung AI Center", kr: "삼성 AI 연구센터"},
      career_freelance: {en: "Freelancing", ru: "Фриланс", kr: "프리랜소"},
      publications_wacv_2024: {en: "Scientific paper, WACV 2024", ru: "Научная статья на WACV 2024", kr: "연구 논문 (WACV 2024)"},
      project_this_cv: {en: "Interactive web-CV, you're here :)", ru: "Интерактивное веб-CV, вы тут :)", kr: "이 이력서"},
      project_image_processing_tool: {en: "Image processing web-tool", ru: "Веб инструмент для фотографий", /*kr: ""*/},
      // project_infinite_fractal: {en: "High precision fractal visualization", ru: "Визуализация фракталов высокой точности", /*kr: "이 이력서"*/},
      project_will_and_reason: {en: "GameDev volunteering", ru: "Волонтер в GameDev проекте", /*kr: ""*/},
      education_master: {en: "Master of Information Science", ru: "Магистратура", kr: "석사"},
      education_bachelor: {en: "Bachelor of Computer Science", ru: "Бакалавриат", kr: "학사"},
      demo_triangle: {en: "Triangle", /* ru: "", kr: "" */},
      demo_frame_generation: {en: "Frame generation", /* ru: "", kr: "" */},
      demo_head_avatar: {en: "Head avatar animation", /* ru: "", kr: "" */ },
      demo_full_body_avatar: {en: "Full-body avatar animation", /* ru: "", kr: "" */ },
      demo_fractal: {en: "The fractal zooming has increased precision compared to naive float32, by using Dekker's double-double arithmetic and perturbation theory", /* ru: "", kr: "" */},
      demo_procedural_generation: {en: "Procedural mesh generation", /* ru: "", kr: "" */ },
      intro_hi: {en: "Hi, I'm Alexey :)", ru: "Привет, меня зовут Лёша :)", kr: "안녕하세요! 저는 \"료샤\"라고 합니다 :)"},
      intro_enjoy_resume: {en: "Enjoy my interactive résumé !", ru: "Вы наткнулись на мое интерактивное резюме", kr: "제 이력서를 방문해줘서 반갑습니다!"},
      intro_using: {en: "I made everything from scratch, using", ru: "Все здесь разработано мной с нуля:", kr: "여기에 모두 것을 저 스스로 만들었습니다."},
      intro_3d: {en: "for 3D graphics", ru: "для 3D графики", kr: "(3D 그래픽)"},
      intro_frontend: {en: "for reactive front-end", ru: "для реактивного фронтенда", kr: "(웹사이트)"},
      intro_close: {en: "Next", ru: "Дальше", kr: "다음것!"},
      controls_mouse_wheel: {en: "Mouse wheel changes CV chapters", ru: "Колесо мыши - перемещение по резюме", /* kr: "" */},
      controls_mouse_move: {en: "Mouse movement rotates 3D scene", ru: "Движение мыши - поворот в 3D сцене", /* kr: "" */},
      controls_close: { en: "Gotcha 🚀", ru: "Понятно 🚀", kr: "알겠어 🚀" },
      controls_fullscreen_key: { en: "For fullscreen press F11 or click such pictogram", ru: "Нажатие на клавишу F11 или такую пиктограмму - полноэкранный режим", /* kr: "" */ },
      resize_tooltip: { en: "Resize by dragging the border", ru: "Потянув за границу, можно настроить ширину", kr: "테두리를 끌으면 크기가 바꿉니다" },
      font_family: { en: "Font family", ru: "Шрифт", kr: "글꼴" },
      fps_limit: { en: "Max frames per sec", ru: "Лимит частоты кадров", kr: "FPS 한계" },
      geo_location: { en: "in Russia / Relocation / Remote", ru: "Москва / СПб / Релокация", kr: "러시아 / 이주" },
      debug_mode: { en: "Debug mode", ru: "Режим отладки", kr: "디버그" },
      current: { en: "Current", ru: "Текущая", /* kr: "" */ },
      senior_engineer: { en: "Senior engineer", ru: "Сениор инженер", /* kr: "" */ },
      middle_engineer: { en: "Middle engineer", ru: "Мидл инженер", /* kr: "" */ },
      junior_engineer: { en: "Junior engineer", ru: "Джуниор инженер", /* kr: "" */ },
      intern_engineer: { en: "Intern", ru: "Стажер", /* kr: "" */ },
      link: { en: "Link", ru: "Ссылка", /* kr: "" */ },
      huawei_job_title: { en: "Frame prediction SDK for mobile games", ru: "Предсказание кадров в мобильных играх", /* kr: "" */ },
      huawei_hooks: { en: "Experimented to speed up rendering of \"Genshin Impact\" game, via hooks of OpenGL commands", ru: "Экспериментировал с ускорением рендеринга в Genshin Impact, через хуки OpenGL команд", /* kr: "" */ },
      huawei_unity_ff_sdk: { en: "Integrated an in-house frame prediction SDK as a Unity plugin (URP pipeline)", ru: "Интегрировал in-house SDK предсказания кадров в Unity плагин для URP пайплайна", /* kr: "" */ },
      huawei_ohos: { en: "Overall, assisted to develop the ecosystem of the mobile operating system OpenHarmony", ru: "Способствовал развитию графики новой операционной системы OpenHarmony", /* kr: "" */ },
      samsung_job_title: { en: "Neural Networks R&D", ru: "R&D нейронных сетей", /* kr: "" */ },
      samsung_ar_avatars: { en: "Solely created a techdemo to render realistic human avatars, with:", ru: "Один разработал приложение с реалистичными человеческими аватарами", /* kr: "" */ },
      samsung_rendering_nn: { en: "rendering by neural networks, running 60 FPS in resolution 512x512px on Qualcomm NPU", ru: "рендеринг через нейронную сеть, в 60 FPS и разрешении 512х512 пикселей на Qualcomm NPU", /* kr: "" */ },
      samsung_animation_system: { en: "my animation system and mesh skinning", ru: "реализовал систему анимации и скининга", /* kr: "" */ },
      samsung_arcore: { en: "augmented reality via ARCore", ru: "встроил дополненную реальность через ARCore", /* kr: "" */ },
      samsung_unity: { en: "Ported the techdemo as a Unity AR application", ru: "Портировал приложение в Unity", /* kr: "" */ },
      samsung_wacv: { en: "Researched and published a paper on neural networks, at WACV 2024 conference: ", ru: "Исследовал и опубликовал научную статью на WACV 2024: ", /* kr: "" */ },
      samsung_kpi: { en: "In a month completed a crucial yearly KPI of another team", ru: "На месяц включился в другую команду и закрыл для них важный годовой KPI", /* kr: "" */ },
      project_page: { en: "Project page", ru: "Обзор", /* kr: "" */ },
      proceedings: { en: "Proceedings", ru: "Конференция", /* kr: "" */ },
      abstract: { en: "Abstract", ru: "Резюме", /* kr: "" */ },
      wacv2024_abstract: { en: "The paper improves \"Deferred Neural Rendering\" approach, reducing overfitting to inconsistent training data, by learning offsets to neural texture coordinates for each training image, then discarding them to preserve real-time inference on mobile hardware", ru: "Статья улучшает подход Deferred Neural Rendering, уменьшая оверфиттинг к неконсистентным данным, через оптимизацию warping сетки для нейронной текстуры под каждый кадр в датасете. После обучения warping отключается, для сохранения real-time скорости рендеринга на мобильных устройствах", /* kr: "" */ },
      wacv2024_author: { en: "I'm the second author of the paper", ru: "Я второй автор статьи", /* kr: "" */ },
      wacv2024_morphing: { en: "Researched the \"morphing\" idea of the paper", ru: "Исследовал и реализовал warping идею статьи", /* kr: "" */ },
      wacv2024_demo: { en: "Developed the mobile phone demo that computes avatar images on mobile GPU and Qualcomm NPU in 30-60 FPS", ru: "Разработал технодемку, считающую нейронную сеть прямо на мобильном GPU и Qualcomm NPU", /* kr: "" */ },
      wacv2024_text: { en: "Prepared a big part of the paper's text and all illustrations", ru: "Подготовил значительную часть текста статьи и все иллюстрации", /* kr: "" */ },
      webcv_author: { en: "Aleksei Larionov", ru: "Алексей Ларионов", /* kr: "" */ },
      webcv_license: { en: "License: ", ru: "Лицензия: ", /* kr: "" */ },
      webcv_repo: { en: "Source code", ru: "Исходный код", /* kr: "" */ },
      webcv_scratch: { en: "Everything is designed and programmed from scratch", ru: "Все спроектировано и запрограммированно с нуля", /* kr: "" */ },
      webcv_demos: { en: "Graphics demos:", ru: "Графические демки", /* kr: "" */ },
      webcv_made_with: { en: "All made with: ", ru: "Реализованы на: ", /* kr: "" */ },
      webcv_loading: { en: "My implementation of non-blocking demo loading via ", ru: "Сделал неблокирующую загрузку каждой демки через ", /* kr: "" */ },
      webcv_web: { en: "Web UI:", ru: "Веб UI: ", /* kr: "" */ },
      webcv_plain: { en: "Plain", ru: "Только", /* kr: "" */ },
      webcv_vanjs: { en: "and tiny library VanJS for reactive UI", ru: "и маленькая библиотека VanJS для реактивного UI", /* kr: "" */ },
      webcv_wheel: { en: "The navigation over CV chapters supports mouse wheel scrolling, with transition animations in plain CSS", ru: "Перемещение по главам резюме с помощью колеса мыши, анимации на простом CSS", /* kr: "" */ },
      webcv_easy: { en: "Easy deployment, no complexity of NodeJS, no webpack", ru: "Простой деплой, без NodeJS или WebPack", /* kr: "" */ },
      webcv_deploy: { en: "Deployed on my web-server (lighttpd) via GitHub CI/CD", ru: "Хостится на моем личном сервере lighttpd, деплой через GitHub CI/CD", /* kr: "" */ },
      a: { en: "", ru: "", /* kr: "" */ },
      a: {en: "", ru: "", /* kr: "" */ },
   };
})()

export function localizeString(key, nullIfMissing = false) {
  return function() {
    let localized = null;
    let lang = 'en';
    if (!(key in UI_STRINGS)) {
       console.log("Missing UI string=" + key);
       localized = nullIfMissing ? null : key;
    } else if (!(CURRENT_LANGUAGE.val in UI_STRINGS[key])) {
       console.log("Missing UI string=" + key + " for language=" + CURRENT_LANGUAGE.val);
       localized = nullIfMissing ? null : UI_STRINGS[key]['en']
    } else {
       localized = UI_STRINGS[key][CURRENT_LANGUAGE.val]
       lang = CURRENT_LANGUAGE.val;
    }
    return {text: localized, lang: lang};
  }
}

export function localizeUiUnsafe(key) {
   return () => UI_STRINGS[key][CURRENT_LANGUAGE.val];
 }

export function localizeUi(key, nullIfMissing = false) {
  return () => (key in UI_STRINGS
    ? (CURRENT_LANGUAGE.val in UI_STRINGS[key]
      ? UI_STRINGS[key][CURRENT_LANGUAGE.val]
      : nullIfMissing ? null : span({class: "missing"}, key))
    : span({class: "missing"}, key));
}

export function localizeUiPostprocess(key, postprocess, nullIfMissing = false) {
   return () => (key in UI_STRINGS
     ? (CURRENT_LANGUAGE.val in UI_STRINGS[key]
       ? postprocess(UI_STRINGS[key][CURRENT_LANGUAGE.val])
       : nullIfMissing ? null : key)
     : key);
 }