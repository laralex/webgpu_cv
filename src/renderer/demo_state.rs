use std::{cell::Cell, rc::Rc};

use crate::GraphicsLevel;

#[derive(Default, Clone, Copy)]
pub struct MouseState {
   pub left: f32,
   pub middle: f32,
   pub right: f32,
   pub wheel: f32,
   pub canvas_position_px: (i32, i32), // origin at top-left
}

#[derive(Default, Clone, Copy)]
pub struct KeyboardState {
   pub m: f32,
   pub comma: f32,
   pub dot: f32,
   pub shift: bool,
   pub alt: bool,
   pub ctrl: bool,
}

#[derive(Clone, Default)]
struct DerivedDynamicState {
   pub time_since_startup_sec: f64,
   pub time_now_sec:   f64,
   pub time_prev_sec:  f64,
   pub time_delta_sec: f64,
   pub frame_rate: f32,
   pub mouse_viewport_position_px: (i32, i32), // origin at bottom-left
}

#[derive(Clone, Default)]
struct DerivedStableState {
   pub aspect_ratio: f32,
}

#[derive(Clone)]
struct StableState {
   pub screen_size: (u32, u32),
   pub graphics_level: GraphicsLevel,
   pub debug_mode: Option<u16>,
}

pub struct ExternalState {
   // dynamic
   mouse: Rc<Cell<MouseState>>,
   keyboard: Rc<Cell<KeyboardState>>,
   time_delta_limit_ms: f64,
   time_of_startup_ms: f64,
   time_of_tick_ms: f64,

   time_now_ms:    f64,
   time_prev_ms:   f64,
   time_delta_ms:  f64,
   frame_idx: usize,
   derived: DerivedDynamicState,

   // stable
   stable: StableState,
   is_stable_updated: bool,
   derived_stable: DerivedStableState,
}

#[derive(Default, Clone, Copy)]
pub struct ExternalStateData {
   // dynamic
   pub mouse: MouseState,
   pub keyboard: KeyboardState,
   pub time_delta_limit_ms: f64,

   pub time_now_ms:    f64,
   pub time_prev_ms:   f64,
   pub time_delta_ms:  f64,
   pub frame_idx: usize,

   // stable
   pub screen_size: (u32, u32),
   pub graphics_level: GraphicsLevel,
   pub debug_mode: Option<u16>,
}

#[allow(unused)]
impl ExternalState {

   pub fn new(time_of_startup_ms: f64) -> Self {
      let mut state = ExternalState::default();
      state.time_of_startup_ms = time_of_startup_ms;
      state
   }

   pub fn data(&self) -> ExternalStateData {
      ExternalStateData {
         mouse: self.mouse.get(),
         keyboard: self.keyboard.get(),
         time_delta_limit_ms: self.time_delta_limit_ms.clone(),
         time_now_ms: self.time_now_ms.clone(),
         time_prev_ms: self.time_prev_ms.clone(),
         time_delta_ms: self.time_delta_ms.clone(),
         frame_idx: self.frame_idx.clone(),
         screen_size: self.stable.screen_size.clone(),
         graphics_level: self.stable.graphics_level.clone(),
         debug_mode: self.stable.debug_mode.clone(),
      }
   }

   pub fn reset(&mut self) {
      self.time_now_ms = Default::default();
      self.time_prev_ms = Default::default();
      self.time_delta_ms = Default::default();
      self.frame_idx = Default::default();
      self.update_derived_state();
   }

   pub fn mouse_unit_position(&self) -> (f32, f32) {
      let px_pos = self.mouse_viewport_position_px();
      return (
         px_pos.0 as f32 / self.stable.screen_size.0 as f32,
         px_pos.1 as f32 / self.stable.screen_size.1 as f32,
      )
   }

   pub fn update_derived_state(&mut self) {
      let now = self.time_now_ms * 0.001;
      let then = self.time_prev_ms * 0.001;
      let delta = self.derived.time_now_sec - self.derived.time_prev_sec;
      
      let current_mouse = self.mouse.get();
      self.derived = DerivedDynamicState {
         time_since_startup_sec: (self.time_of_tick_ms - self.time_of_startup_ms)*0.001,
         time_now_sec: now,
         time_prev_sec: then,
         time_delta_sec: delta,
         frame_rate: (1.0 / delta) as f32,
         mouse_viewport_position_px: (
            current_mouse.canvas_position_px.0,
            self.stable.screen_size.1 as i32 - current_mouse.canvas_position_px.1
         ),
      };
      if self.is_stable_updated {
         self.derived_stable = DerivedStableState {
            aspect_ratio: self.stable.screen_size.0 as f32 / self.stable.screen_size.1 as f32,
         }
      }
   }

   pub fn screen_size(&self) -> (u32, u32) { self.stable.screen_size }
   pub fn graphics_level(&self) -> GraphicsLevel { self.stable.graphics_level }
   pub fn debug_mode(&self) -> Option<u16> { self.stable.debug_mode }
   pub fn aspect_ratio(&self) -> f32 { self.derived_stable.aspect_ratio }
   pub fn is_stable_updated(&self) -> bool { self.is_stable_updated }

   pub fn mouse(&self) -> &Rc<Cell<MouseState>> { &self.mouse }
   pub fn keyboard(&self) -> &Rc<Cell<KeyboardState>> { &self.keyboard }
   pub fn time_of_startup_ms(&self) -> f64 { self.time_of_startup_ms }
   pub fn time_of_tick_ms(&self) -> f64 { self.time_of_tick_ms }
   pub fn time_now_ms(&self) -> f64 { self.time_now_ms }
   pub fn time_prev_ms(&self) -> f64 { self.time_prev_ms }
   pub fn time_delta_ms(&self) -> f64 { self.time_delta_ms }
   pub fn time_delta_limit_ms(&self) -> f64 { self.time_delta_limit_ms }
   pub fn frame_idx(&self) -> usize { self.frame_idx }
   
   pub fn time_since_startup_ms(&self) -> f64 { self.derived.time_since_startup_sec }
   pub fn time_now_sec(&self) -> f64 { self.derived.time_now_sec }
   pub fn time_prev_sec(&self) -> f64 { self.derived.time_prev_sec }
   pub fn time_delta_sec(&self) -> f64 { self.derived.time_delta_sec }
   pub fn frame_rate(&self) -> f32 { self.derived.frame_rate }
   pub fn mouse_viewport_position_px(&self) -> (i32, i32) { self.derived.mouse_viewport_position_px }

   pub fn set_time_delta_limit_ms(&mut self, time_delta_limit_ms: f64) {
      self.time_delta_limit_ms = time_delta_limit_ms;
   }

   pub fn set_screen_size(&mut self, (width_px, height_px): (u32, u32)) {
      self.stable.screen_size = (width_px, height_px);
      self.is_stable_updated = true;
   }

   pub fn set_graphics_level(&mut self, graphics_level: GraphicsLevel) {
      self.stable.graphics_level = graphics_level;
      self.is_stable_updated = true;
   }

   pub fn set_debug_mode(&mut self, debug_mode: Option<u16>) {
      self.stable.debug_mode = debug_mode;
      self.is_stable_updated = true;
   }

   pub fn override_time(&mut self, timestamp_ms: f64, frame_idx: usize) {
      self.frame_idx = frame_idx;
      self.time_delta_ms = 0.0; // .max(1)
      self.time_prev_ms  = timestamp_ms;
      self.time_now_ms   = timestamp_ms;
      self.update_derived_state();
   }

   pub fn tick(&mut self, tick_timestamp_ms: f64) {
      self.frame_idx += 1;
      self.time_delta_ms = tick_timestamp_ms - self.time_of_tick_ms;
      self.time_of_tick_ms = tick_timestamp_ms;
      self.time_prev_ms  = self.time_now_ms;
      self.time_now_ms   += self.time_delta_ms;
      self.update_derived_state();
   }

   pub fn tick_from_delta(&mut self, tick_delta_ms: f64) {
      self.tick(self.time_prev_ms + tick_delta_ms);
   }

   pub fn dismiss_events(&mut self) {
      self.is_stable_updated = false;

      let mut current_mouse_state = self.mouse.get();
      ExternalState::dismiss_input_event(&mut current_mouse_state.left);
      ExternalState::dismiss_input_event(&mut current_mouse_state.middle);
      ExternalState::dismiss_input_event(&mut current_mouse_state.right);
      self.mouse.set(current_mouse_state);

      let mut current_keyboard_state = self.keyboard.get();
      ExternalState::dismiss_input_event(&mut current_keyboard_state.m);
      ExternalState::dismiss_input_event(&mut current_keyboard_state.comma);
      ExternalState::dismiss_input_event(&mut current_keyboard_state.dot);
      self.keyboard.set(current_keyboard_state);
   }

   fn dismiss_input_event(input_axis: &mut f32) {
      if *input_axis < 0.0 { *input_axis = 0.0; }
   }
}

impl Default for ExternalState {
   fn default() -> Self {
      Self {
         mouse: Rc::new(Cell::new(Default::default())),
         keyboard: Rc::new(Cell::new(Default::default())),
         time_delta_limit_ms: Default::default(),
         time_of_startup_ms: Default::default(),
         time_of_tick_ms: Default::default(),
         time_delta_ms: Default::default(),
         time_now_ms: Default::default(),
         time_prev_ms: Default::default(),
         frame_idx: Default::default(),
         derived: Default::default(),
         is_stable_updated: true,
         stable: StableState {
            screen_size: (1, 1),
            graphics_level: Default::default(),
            debug_mode: Default::default(),
         } ,
         derived_stable: Default::default(),
      }
   }
}

pub struct DemoStateHistory {
   history: Vec<ExternalStateData>,
   history_size: usize,
   history_head_idx: usize,
}

#[allow(unused)]
impl DemoStateHistory {
   pub fn new() -> Self {
       let history = vec![Default::default(); 256];
       Self {
           history,
           history_size: 0,
           history_head_idx: 0,
       }
   }

   pub fn sample_state(&self, offset_back: usize) -> Option<ExternalStateData> {
       if offset_back >= self.history_size {
           return None;
       }
       let state_idx = (self.history_head_idx + self.history.len()) - offset_back - 1;
       let state_idx = state_idx % self.history.len();
       log::info!("sample_state {} {}", offset_back, self.history.len());
       Some(self.history[state_idx])
   }

   pub fn store_state(&mut self, state: ExternalStateData) {
       self.history[self.history_head_idx] = state;
       self.history_head_idx += 1;
       if self.history_head_idx >= self.history.len() {
           self.history_head_idx = 0;
       }
       self.history_size = self.history.len().min(self.history_size + 1);
   }
}

pub struct DemoHistoryPlayback {
   history_playback_offset: usize,
   frame_lock_timestamp_ms: Option<f64>,
}

#[allow(unused)]
impl DemoHistoryPlayback {
   pub fn new() -> Self {
      Self {
         history_playback_offset: 0,
         frame_lock_timestamp_ms: None,
      }
   }

   pub fn is_playing_back(&self) -> bool { self.frame_lock_timestamp_ms.is_some() }
   pub fn playback_timestamp_ms(&self) -> Option<f64> { self.frame_lock_timestamp_ms }

   pub fn toggle_frame_lock(&mut self, lock_at_timestamp_ms: f64) -> bool {
      if self.frame_lock_timestamp_ms.is_none() {
         // entering frame lock mode - remember current time, and don't advance it
         self.frame_lock_timestamp_ms = Some(lock_at_timestamp_ms);
         self.history_playback_offset = 0;
         log::info!("Frame lock mode ON");
         true
     } else {
         // exiting frame lock mode - set current time to the previous real time (which advanced even in lock)
         self.frame_lock_timestamp_ms.take();
         log::info!("Frame lock mode OFF");
         false
     }
   }

   pub fn advance_back(&mut self, state_history: &DemoStateHistory) {
      if self.frame_lock_timestamp_ms.is_some() {
         if let Some(state) = state_history.sample_state(self.history_playback_offset + 1) {
             self.frame_lock_timestamp_ms.replace(state.time_now_ms);
             self.history_playback_offset += 1;
         }
      }
   }

   pub fn advance_forward(&mut self, state_history: &DemoStateHistory) {
      if self.frame_lock_timestamp_ms.is_some() && self.history_playback_offset > 0 {
         if let Some(state) = state_history.sample_state(self.history_playback_offset - 1) {
            self.frame_lock_timestamp_ms.replace(state.time_now_ms);
            self.history_playback_offset -= 1;
         }
      }
   }
}


#[allow(unused)]
pub struct FrameStateRef<'a> {
    pub demo_state_history: &'a mut DemoStateHistory,
    pub demo_history_playback: &'a mut DemoHistoryPlayback,
    pub demo_state: &'a mut ExternalState,
    pub previous_timestamp_ms: f64,
    pub now_timestamp_ms: f64,
}

pub fn handle_keyboard<'a>(keyboard: KeyboardState, state: FrameStateRef<'a>) {
   if keyboard.m < 0.0 {
       if state.demo_history_playback.toggle_frame_lock(state.previous_timestamp_ms) == false {
           // canceling frame lock, resume time
           let frame_idx = 0;
           state.demo_state.override_time(state.previous_timestamp_ms, frame_idx);
       }
   }
   if keyboard.comma < 0.0 || keyboard.comma > 0.0 && keyboard.shift {
       state.demo_history_playback.advance_back(&state.demo_state_history);
   }
   if keyboard.dot < 0.0 || keyboard.dot > 0.0 && keyboard.shift {
       state.demo_history_playback.advance_forward(&state.demo_state_history);
   }
}