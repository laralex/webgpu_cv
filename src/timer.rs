
pub use details::*;

#[cfg(feature = "web")] mod details {

use crate::js_interop::performance;
pub struct ScopedTimer<'a> {
   js_performance: web_sys::Performance,
   begin_ms: f64,
   label: &'a str,
}

impl<'a> ScopedTimer<'a> {
   pub fn new(label: &'a str) -> Self {
      let js_performance = performance();
      let begin_ms = js_performance.now();
      Self {
         js_performance,
         begin_ms,
         label,
      }
   }
}
impl<'a> Drop for ScopedTimer<'a> {
   fn drop(&mut self) {
      let elapsed_ms = self.js_performance.now() - self.begin_ms;
      log::info!("[Timer] {} {} ms", self.label, elapsed_ms);
   }
}

} // mod details

#[cfg(not(feature = "web"))] mod details {

use std::time::Instant;
pub struct ScopedTimer<'a> {
   begin: Instant,
   label: &'a str,
}

impl<'a> ScopedTimer<'a> {
   pub fn new(label: &'a str) -> Self {
      Self {
         begin: Instant::now(),
         label,
      }
   }
}

impl<'a> Drop for ScopedTimer<'a> {
   fn drop(&mut self) {
      let elapsed_ms = (Instant::now() - self.begin).as_millis();
      log::info!("[Timer] {} {} ms", self.label, elapsed_ms);
   }
}

} // mod details



