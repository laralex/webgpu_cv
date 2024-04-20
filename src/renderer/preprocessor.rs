use gpp;
use std::hash::Hash;
pub struct Preprocessor {
   context: gpp::Context,
}

impl Preprocessor {
   pub fn new() -> Self {
      Self {
         context: gpp::Context::new(),
      }
   }

   pub fn define(&mut self, key: &str, value: &str) {
      self.context.macros.insert(key.to_owned(), value.to_owned());
   }

   pub fn process(&mut self, str: &str) -> Option<String> {
      match gpp::process_str(str, &mut self.context) {
         Ok(out) => Some(out),
         Err(e) => {
            eprintln!("Error running preprocessor on shader: {}", e);
            None
         }
         // Err(gpp::Error::ChildFailed { status }) => None,
         // Err(gpp::Error::FileError { filename, line, error }) => None,
         // Err(gpp::Error::FromUtf8Error(e)) => None,
         // Err(gpp::Error::InvalidCommand { command_name }) => None,
         // Err(gpp::Error::PipeFailed) => None,
         // Err(gpp::Error::TooManyParameters { command }) => None,
         // Err(gpp::Error::UnexpectedCommand { command }) => None,
         // Err(gpp::Error::IoError(e)) => None,
      }
   }
}

impl Hash for Preprocessor {
   fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
      // NOTE: doesn't consider order of values in macros
      // which might be different for different instsantiations of Preprocessor (idk)
      self.context.macros.iter()
         .for_each(|(k, v)| {
            k.hash(state);
            v.hash(state);
         });
   }
}