#[cfg(feature = "web")]
use crate::js_interop::performance;

pub struct ScopedTimer<'a> {
   #[cfg(feature = "web")]
   js_performance: web_sys::Performance,
   begin_ms: f64,
   label: &'a str,
}

impl<'a> ScopedTimer<'a> {
   pub fn new(label: &'a str) -> Self {
      #[cfg(feature = "web")] {
         let js_performance = performance();
         let begin_ms = js_performance.now();
         Self {
            js_performance,
            begin_ms,
            label,
         }
      }
      #[cfg(not(feature = "web"))] {
         Self {
            begin_ms: 0.0,
            label,
         }
      }
   }
}


impl<'a> Drop for ScopedTimer<'a> {
   fn drop(&mut self) {
      #[cfg(feature = "web")] {
         let elapsed_ms = self.js_performance.now() - self.begin_ms;
         web_sys::console::log_3(&"[Timer]".into(), &self.label.into(), &elapsed_ms.into());
      }
      #[cfg(not(feature = "web"))] {
         let elapsed_ms = self.begin_ms;
         println!("[Timer] {} {}", self.label, elapsed_ms);
      }
   }
}