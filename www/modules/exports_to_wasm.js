export const CURRENT_DEMO_LOADING_PROGRESS = van.state(null);
export const CURRENT_GRAPHICS_SWITCHING_PROGRESS = van.state(null);

// normalized progress from 0.0 to 1.0
export function demo_loading_apply_progress(progress) {
   CURRENT_DEMO_LOADING_PROGRESS.val = progress;
}

export function demo_loading_finish() {
   CURRENT_DEMO_LOADING_PROGRESS.val = null;
}

// normalized progress from 0.0 to 1.0
export function graphics_switching_apply_progress(progress) {
   CURRENT_GRAPHICS_SWITCHING_PROGRESS.val = progress;
}

export function graphics_switching_finish() {
   CURRENT_GRAPHICS_SWITCHING_PROGRESS.val = null;
}