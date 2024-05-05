#[allow(unused)]
pub fn log_init() {
    cfg_if::cfg_if! {
        if #[cfg(feature = "web")] {
            use console_log;
            use log;
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            console_log::init_with_level(log::Level::Warn).expect("Couldn't initialize logger");
            log::info!("web env::log_init");
        } else if #[cfg(feature = "win")] {
            use env_logger;
            env_logger::init();
            log::info!("win env::log_init");
        }
  }
}

// pub fn log_2<A>(prefix: &str, val: &A) where A: ?Sized + std::fmt::Display + wasm_bindgen::JsCast {
   // cfg_if::cfg_if! {
   //    if #[cfg(target_arch = "wasm32")] {
   //       use web_sys;
   //       web_sys::console::log_2(&prefix.into(), &val.into());
   //    } else {
   //       println!("{} {}", prefix, val);
   //    }
   // }
// }

