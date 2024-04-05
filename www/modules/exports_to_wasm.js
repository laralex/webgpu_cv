export const CURRENT_DEMO_LOADING_PROGRESS = van.state(null);

// normalized progress from 0.0 to 1.0
export function demo_loading_apply_progress(progress) {
   CURRENT_DEMO_LOADING_PROGRESS.val = progress;
}

export function demo_loading_finish() {
   CURRENT_DEMO_LOADING_PROGRESS.val = null;
}