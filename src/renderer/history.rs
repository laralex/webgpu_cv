use super::ExternalStateData;

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
       let state = self.history[state_idx];
       log::info!("sample_state {} {}: t {}", offset_back, self.history.len(), state.absolute_time_tick_ms);
       Some(state)
   }

   pub fn store_state(&mut self, state: ExternalStateData) {
       self.history[self.history_head_idx] = state;
       self.history_head_idx += 1;
       if self.history_head_idx >= self.history.len() {
           self.history_head_idx = 0;
       }
       self.history_size = self.history.len().min(self.history_size + 1);
   }

   pub fn reset_history(&mut self) {
      self.history_size = 0;
   }
}

pub struct DemoHistoryPlayback {
   history_playback_offset: usize,
   frame_lock_timestamp_ms: Option<f64>,
   frame_lock_time_ms: Option<f64>,
}

#[allow(unused)]
impl DemoHistoryPlayback {
   pub fn new() -> Self {
      Self {
         history_playback_offset: 0,
         frame_lock_timestamp_ms: None,
         frame_lock_time_ms: None,
      }
   }

   pub fn is_playing_back(&self) -> bool { self.frame_lock_timestamp_ms.is_some() }
   pub fn playback_timestamp_ms(&self) -> Option<f64> { self.frame_lock_timestamp_ms }
   pub fn playback_time_ms(&self) -> Option<f64> { self.frame_lock_time_ms }

   // returns time when the lock was locked: (global_timer_timestamp_millisec, local_time_millisec)
   pub fn toggle_frame_lock(&mut self, lock_at_timestamp_ms: f64, lock_at_time_ms: f64) -> Option<(f64, f64)> {
      if self.frame_lock_timestamp_ms.is_none() {
         self.start_playback(lock_at_timestamp_ms, lock_at_time_ms);
         None
     } else {
         self.cancel_playback()
     }
   }

   pub fn start_playback(&mut self, lock_at_timestamp_ms: f64, lock_at_time_ms: f64) {
      // entering frame lock mode - remember current time, and don't advance it
      self.frame_lock_timestamp_ms = Some(lock_at_timestamp_ms);
      self.frame_lock_time_ms = Some(lock_at_time_ms);
      self.history_playback_offset = 0;
      log::info!("Frame lock mode ON t:{}", lock_at_timestamp_ms);
   }

   pub fn cancel_playback(&mut self) -> Option<(f64, f64)> {
      // exiting frame lock mode - set current time to the previous real time (which advanced even in lock)
      // log::info!("Frame lock mode OFF t:{}", self.frame_lock_timestamp_ms.unwrap());
      match (self.frame_lock_timestamp_ms.take(), self.frame_lock_time_ms.take()) {
         (Some(lock_timestamp_ms), Some(lock_time_ms)) => Some((lock_timestamp_ms, lock_time_ms)),
         _ => None,
      }
   }

   pub fn play_back(&mut self, state_history: &DemoStateHistory) {
      if self.is_playing_back() {
         if let Some(state) = state_history.sample_state(self.history_playback_offset + 1) {
             self.frame_lock_timestamp_ms.replace(state.absolute_time_tick_ms);
             self.frame_lock_time_ms.replace(state.time_now_ms);
             self.history_playback_offset += 1;
         }
      }
   }

   pub fn play_forward(&mut self, state_history: &DemoStateHistory) {
      if self.is_playing_back() {
         let new_history_offset = self.history_playback_offset.saturating_sub(1);
         if let Some(state) = state_history.sample_state(new_history_offset) {
            self.frame_lock_timestamp_ms.replace(state.absolute_time_tick_ms);
            self.frame_lock_time_ms.replace(state.time_now_ms);
            self.history_playback_offset = new_history_offset;
         }
      }
   }
}