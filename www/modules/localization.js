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
      treesruler_href: {en: "./trees_ruler/eng", ru: "./trees_ruler/rus", kr: "./trees_ruler/eng"},
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
      project_this_cv: {en: "Interactive web-CV, you're here :)", ru: "Интерактивное веб-резюме, Вы тут :)", kr: "이 이력서"},
      project_image_processing_tool: {en: "Image processing web-tool", ru: "Веб инструмент для фотографий", kr: "Image processing web-tool"},
      // project_infinite_fractal: {en: "High precision fractal visualization", ru: "Визуализация фракталов высокой точности", /*kr: "이 이력서"*/},
      project_will_and_reason: {en: "GameDev volunteering", ru: "Волонтер в GameDev проекте", kr: "GameDev volunteering"},
      education_master: {en: "Master of Information Science", ru: "Магистратура", kr: "석사"},
      education_bachelor: {en: "Bachelor of Computer Science", ru: "Бакалавриат", kr: "학사"},
      demo_triangle: {en: "Triangle", ru: "Просто треугольник", kr: "Triangle"},
      demo_frame_generation: {en: "Frame generation", ru: "Генерация кадров", kr: "Frame generation"},
      demo_head_avatar: {en: "Head avatar animation", ru: "Анимированный аватар головы", kr: "Head avatar animation" },
      demo_full_body_avatar: {en: "Full-body avatar animation",  ru: "Анимированный аватар тела человека", kr: "Full-body avatar animation" },
      demo_fractal: {en: "The fractal zooming has increased precision compared to naive float32, by using Dekker's double-double arithmetic and perturbation theory", ru: "Зум во фрактал, при этом точность выше чем у наивного подсчета во float32, за счет подходов: Dekker double-double arithmetic, perturbation theory",  kr: "The fractal zooming has increased precision compared to naive float32, by using Dekker's double-double arithmetic and perturbation theory"},
      demo_procedural_generation: {en: "Procedural mesh generation",  ru: "Процедурная генерация меша", kr: "Procedural mesh generation" },
      intro_hi: {en: "Hi, I'm Alexey :)", ru: "Привет, меня зовут Лёша :)", kr: "안녕하세요! 저는 \"료샤\"라고 합니다 :)"},
      intro_enjoy_resume: {en: "Enjoy my interactive résumé !", ru: "Вы наткнулись на мое интерактивное резюме", kr: "제 이력서를 방문해줘서 반갑습니다!"},
      intro_using: {en: "I made everything from scratch, using", ru: "Все здесь разработано мной с нуля:", kr: "여기에 모두 것을 저 스스로 만들었습니다."},
      intro_3d: {en: "for 3D graphics", ru: "для 3D графики", kr: "(3D 그래픽)"},
      intro_frontend: {en: "for reactive front-end", ru: "для реактивного фронтенда", kr: "(웹사이트)"},
      intro_close: {en: "Next", ru: "Дальше", kr: "다음것!"},
      controls_mouse_wheel: {en: "Mouse wheel changes CV chapters", ru: "Колесо мыши - перемещение по резюме", },
      controls_mouse_move: {en: "Mouse movement rotates 3D scene", ru: "Движение мыши - поворот в 3D сцене", },
      controls_close: { en: "Gotcha 🚀", ru: "Понятно 🚀", kr: "알겠어 🚀" },
      controls_fullscreen_key: { en: "For fullscreen press F11 or click such pictogram", ru: "Нажатие на клавишу F11 или такую пиктограмму - полноэкранный режим",  },
      resize_tooltip: { en: "Resize by dragging the border", ru: "Потянув за границу, можно настроить ширину", kr: "테두리를 끌으면 크기가 바꿉니다" },
      font_family: { en: "Font family", ru: "Шрифт", kr: "글꼴" },
      fps_limit: { en: "Max frames per sec", ru: "Лимит частоты кадров", kr: "FPS 한계" },
      geo_location: { en: "in Russia / Relocation / Remote", ru: "Москва / СПб / Релокация", kr: "러시아 / 이주" },
      debug_mode: { en: "Debug mode", ru: "Режим отладки", kr: "디버그" },
      current: { en: "Current", ru: "Текущая", kr: "오늘" },
      senior_engineer: { en: "Senior engineer", ru: "Сениор инженер", kr: "고위 기관사" },
      middle_engineer: { en: "Middle engineer", ru: "Мидл инженер", kr: "기관사" },
      junior_engineer: { en: "Junior engineer", ru: "Джуниор инженер", kr: "하급 기관사" },
      intern_engineer: { en: "Intern", ru: "Стажер", kr: "인턴" },
      link: { en: "Link", ru: "Ссылка", kr: "링크" },
      huawei_job_title: { en: "Frame prediction SDK for mobile games", ru: "Предсказание кадров в мобильных играх", kr: "렌더링 프레임 예측" },
      huawei_hooks: { en: "Experimented to speed up rendering of \"Genshin Impact\" game, via hooks of OpenGL commands", ru: "Экспериментировал с ускорением рендеринга в Genshin Impact, через хуки OpenGL команд", kr: "저는 Genshin Impact 게임에서 렌더링 속도를 높이는 실험을 했어요. 저는 OpenGL 함수 후킹을 사용했어요." },
      huawei_unity_ff_sdk: { en: "Integrated an in-house frame prediction SDK as a Unity plugin (URP pipeline)", ru: "Интегрировал in-house SDK предсказания кадров в Unity плагин для URP пайплайна", kr: "저는 C++ 렌더링 프레임 예측 SDK를 Unity에 통합했어요" },
      huawei_ohos: { en: "Overall, assisted to develop the ecosystem of the mobile operating system OpenHarmony", ru: "Способствовал развитию графики новой операционной системы OpenHarmony", kr: "저는 세로운 OS OpenHarmony 그래픽 체계의 공헌했어요." },
      samsung_job_title: { en: "Neural Networks R&D", ru: "R&D нейронных сетей", kr: "신경망 연구 개발" },
      samsung_ar_avatars: { en: "Solely created a mobile techdemo to render realistic human avatars, with:", ru: "Один разработал мобильное приложение с реалистичными человеческими аватарами", kr: "저는 사람 아바탈들 있는 모바일 앱을 혼자 만드렀어요:" },
      samsung_rendering_nn: { en: "rendering by neural networks, running 60 FPS in resolution 512x512px on Qualcomm NPU", ru: "рендеринг через нейронную сеть, в 60 FPS и разрешении 512х512 пикселей на Qualcomm NPU", kr: "Qualcomm NPU에서 신경망으로 렌더링이 60 FPS 512x512화소 이었어요." },
      samsung_animation_system: { en: "my animation system and mesh skinning", ru: "реализовал систему анимации и скининга", kr: "저는 애니메이션과 skinning 체계을 만들었어요." },
      samsung_arcore: { en: "augmented reality via ARCore", ru: "встроил дополненную реальность через ARCore", kr: "저는 ARCore로 증강현실을 통합했어요." },
      samsung_unity: { en: "Ported the techdemo as a Unity AR application", ru: "Портировал приложение в Unity", kr: "그 모바일 앱은 제가 Unity으로 바꿨어요." },
      samsung_wacv: { en: "Researched and published a paper on neural networks, at WACV 2024 conference: ", ru: "Исследовал и опубликовал научную статью на WACV 2024: ", kr: "저는 연구 논문을 WACV 2024 컨퍼런스 에서 출판하다 -- " },
      samsung_kpi: { en: "In a month completed a crucial yearly KPI of another team", ru: "На месяц включился в другую команду и закрыл для них важный годовой KPI", kr: "저는 한개 월 동안 다른 팀에서 일하고 KPI를 완료했어요." },
      project_page: { en: "Project page", ru: "Обзор", kr: "개요"  },
      proceedings: { en: "Proceedings", ru: "Конференция", kr: "컨퍼런스" },
      abstract: { en: "Abstract", ru: "Резюме", kr: "개념" },
      wacv2024_abstract: { en: "The paper improves \"Deferred Neural Rendering\" approach, reducing overfitting to inconsistent training data, by learning offsets to neural texture coordinates for each training image, then discarding them to preserve real-time inference on mobile hardware", ru: "Статья улучшает подход Deferred Neural Rendering, уменьшая оверфиттинг к неконсистентным данным, через оптимизацию warping сетки для нейронной текстуры под каждый кадр в датасете. После обучения warping отключается, для сохранения real-time скорости рендеринга на мобильных устройствах", kr: "The paper improves \"Deferred Neural Rendering\" approach, reducing overfitting to inconsistent training data, by learning offsets to neural texture coordinates for each training image, then discarding them to preserve real-time inference on mobile hardware"  },
      wacv2024_author: { en: "I'm the second author of the paper", ru: "Я второй автор статьи", kr: "I'm the second author of the paper" },
      wacv2024_morphing: { en: "Researched the \"morphing\" idea of the paper", ru: "Исследовал и реализовал warping идею статьи",  kr: "Researched the \"morphing\" idea of the paper" },
      wacv2024_demo: { en: "Developed the mobile phone demo that computes avatar images on mobile GPU and Qualcomm NPU in 30-60 FPS", ru: "Разработал технодемку, считающую нейронную сеть прямо на мобильном GPU и Qualcomm NPU",  kr: "Developed the mobile phone demo that computes avatar images on mobile GPU and Qualcomm NPU in 30-60 FPS" },
      wacv2024_text: { en: "Prepared a big part of the paper's text and all illustrations", ru: "Подготовил значительную часть текста статьи и все иллюстрации",  kr: "Prepared a big part of the paper's text and all illustrations" },
      webcv_author: { en: "by Aleksei Larionov", ru: "Алексей Ларионов", kr: "by Aleksei Larionov" },
      webcv_license: { en: "License: ", ru: "Лицензия: ", kr: "사용권: "},
      webcv_repo: { en: "Source code", ru: "Исходный код", kr: "소스 코드" },
      webcv_scratch: { en: "Everything is designed and programmed from scratch", ru: "Все спроектировано и запрограммированно с нуля",  kr: "Everything is designed and programmed from scratch" },
      webcv_demos: { en: "Graphics demos:", ru: "Графические демки",  kr: "Graphics demos:" },
      webcv_made_with: { en: "All made with: ", ru: "Реализованы на: ",  kr: "All made with: " },
      webcv_loading: { en: "My implementation of non-blocking demo loading via ", ru: "Сделал неблокирующую загрузку каждой демки через ",  kr: "My implementation of non-blocking demo loading via " },
      webcv_web: { en: "Web UI:", ru: "Веб UI: ",  kr: "Web UI:" },
      webcv_plain: { en: "Plain", ru: "Только",  kr: "Plain" },
      webcv_vanjs: { en: "and tiny library VanJS for reactive UI", ru: "и маленькая библиотека VanJS для реактивного UI",  kr: "and tiny library VanJS for reactive UI" },
      webcv_wheel: { en: "The navigation over CV chapters supports mouse wheel scrolling, with transition animations in plain CSS", ru: "Перемещение по главам резюме с помощью колеса мыши, анимации на простом CSS", kr: "The navigation over CV chapters supports mouse wheel scrolling, with transition animations in plain CSS" },
      webcv_easy: { en: "Easy deployment, no complexity of NodeJS, no webpack", ru: "Простой деплой, без NodeJS или WebPack",  kr: "Easy deployment, no complexity of NodeJS, no webpack" },
      webcv_deploy: { en: "Deployed on my web-server (lighttpd) via GitHub CI/CD", ru: "Хостится на моем личном сервере lighttpd, деплой через GitHub CI/CD", kr: "Deployed on my web-server (lighttpd) via GitHub CI/CD" },
      treesruler_tool: {en: "The tool helps to", ru: "Позволяет", kr: "The tool helps to"},
      treesruler_collect: {en: "collect measurements of trees from photos", ru: "замерять деревья по фото", kr: "collect measurements of trees from photos"},
      treesruler_audience: {en: "I designed it for forest scientists. It allows to:", ru: "Я разработал его для ученых лесотехников. Можно:", kr: "I designed it for forest scientists. It allows to:"},
      treesruler_adjust: {en: "Freely adjust measurements by mouse, snap them to a grid, or set precisely in UI", ru: "Свободно размещать линии, выравнивать по сетке, или точно настраивать в UI", kr: "Freely adjust measurements by mouse, snap them to a grid, or set precisely in UI"},
      treesruler_yaml: {en: "Export or import the measurements from a local YAML file", ru: "Делать экспорт/импорт в YAML файлы", kr: "Export or import the measurements from a local YAML file"},
      treesruler_grouping: {en: "Add, delete, duplicate, group measurements", ru: "Клонировать, удалять, группировать замеры", kr: "Add, delete, duplicate, group measurements"},
      treesruler_settings: {en: "Adjusting visualization and localization", ru: "Конфигурировать стиль отрисовки", kr: "Adjusting visualization and localization"},
      treesruler_libs: {en: "Made with: ", ru: "Сделал с помощью: ", kr: "Made with: "},
      willreason_page: {en: "Project page", ru: "Страница проекта", kr: "프로젝트 페이지" },
      willreason_videos: {en: "Demo videos", ru: "Демо ролики", kr: "Demo videos"},
      willreason_strategy: {en: "(4X strategy game, unreleased)", ru: "4X стратегическая игра, не выпущена",  kr: "(4X strategy game, unreleased)"},
      willreason_volunteer: {en: "I volunteered during inception of the project:", ru: "Волонтерствовал с самого начала проекта", kr: "I volunteered during inception of the project:"},
      willreason_pathfinding: {en: "Implemented pathfinding prototype (A* algorithm)", ru: "Реализовал прототип поиска путей на гексогональной карте (А*)", kr: "Implemented pathfinding prototype (A* algorithm)"},
      willreason_ui: {en: "Developed parts of HUD and game logic of units", ru: "Разрабатывал части UI и игровую логику юнитов", kr: "Developed parts of HUD and game logic of units"},
      willreason_game_design: {en: "Engaged in the game design process", ru: "Обсуждал детали гейм-дизайна", kr: "Engaged in the game design process"},
      willreason_libs: {en: "Made with: ", ru: "Сделано на: ", kr: "Made with: "},
      master_title: {en: "MSc of Information Science", ru: "Магистр: Информационные науки и технологии", kr: "MSc of Information Science"},
      master_honors: {en: "with Honors", ru: "с отличием", kr: "우등으로" },
      master_gpa: {en: "GPA 5/5", ru: "балл 5/5", kr: "GPA 5/5"  },
      master_university: {en: "Skolkovo Institute of Science & Technology", ru: "Сколковский Институт Науки и Технологий", kr: "Skolkovo Institute of Science & Technology", kr: ""},
      master_research: {en: "Researched augmented reality based on neural networks, supervised by renowned ", ru: "Исследовал дополненную реальность с рендерингом нейронными сетями, под руководством ", kr: "Researched augmented reality based on neural networks, supervised by renowned "},
      master_victor: {en: "Dr. Victor Lempitsky", ru: "Виктора Лемпитского", kr: "비크토르 렘비트스키" },
      master_thesis: {en: "Defended the thesis on real-time rendering via neural networks on mobile hardware", ru: "Дипломная работа про рендеринг нейронными сетями в реальном времени на мобильных устройствах", kr: "Defended the thesis on real-time rendering via neural networks on mobile hardware"},
      master_courses: {en: "I've taken courses on:", ru: "Прослушал курсы про", kr: "I've taken courses on:"},
      master_deeplearning: {en: "Machine Learning / Deep Learning", ru: "Машинное / Глубинное обучение", kr: "Machine Learning / Deep Learning"},
      master_3dcv: {en: "3D Computer Vision", ru: "3D компьютерное зрение", kr: "3D Computer Vision"},
      master_parallel: {en: "Parallel computing", ru: "Параллельные вычисления", kr: "Parallel computing"},
      master_unity: {en: "Unity game engine", ru: "Игровой движок Unity", kr: "Unity game engine"},
      bachelor_title: {en: "BSc of Computer Science", ru: "Бакалавр: Математическое обеспечение информационных систем", kr: "BSc of Computer Science"},
      bachelor_university: {en: "Peter The Great St. Petersburg Polytechnic University", ru: "Санкт-Петербургский Политехнический Университет Петра Великого", kr: "Peter The Great St. Petersburg Polytechnic University" },
      bachelor_austria: {en: "Winner of a project marathon jointly with TU Graz (Austria)", ru: "Победитель проектного марафона совместно с TU Graz (Австрия)", kr: "Winner of a project marathon jointly with TU Graz (Austria)"},
      bachelor_icpc: {en: "Twice half-finalist of ICPC world olympiad", ru: "Дважды полуфиналист всемирной олимпиады ICPC", kr: "Twice half-finalist of ICPC world olympiad"},
      bachelor_badge: {en: "\"Student of the year\" badge", ru: "Именные стипендии и звание \"Студент года\"", kr: "\"Student of the year\" badge"},
      bachelor_thesis: {en: "Defended the thesis on procedural generation of 3D meshes", ru: "Дипломная работа про процедурную генерацию 3D моделей зданий", kr: "Defended the thesis on procedural generation of 3D meshes"},
      bachelor_math: {en: "Math (all core fields)", ru: "Основные разделы математики", kr: "Math (all core fields)"},
      bachelor_architecture: {en: "Computer Architecture", ru: "Архитектуру компьютера", kr: "Computer Architecture"},
      bachelor_oracle: {en: "Oracle Database administration", ru: "Администрирование Oracle баз данных", kr: "Oracle Database administration"},
      // a: {en: "", ru: "",  },
   };
})()

export function reportMissingLocalization() {
  let missing = {
    en: [], 
    ru: [], 
    kr: [], 
    // fr: [],
  }
  Object.entries(UI_STRINGS).forEach((kv) => {
    let key = kv[0];
    let localizations = kv[1];
    Object.keys(missing).forEach((lang) => {
      if (!(lang in localizations) || !localizations[lang]) {
        missing[lang].push(key);
      }
    })
  });
  Object.entries(missing).forEach((kv) =>
    (kv[1].length > 0) ?
    console.log("! missing localization lang=", kv[0], "keys=", kv[1]) : {});
}

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