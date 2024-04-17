use gpp;

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
         Err(gpp::Error::ChildFailed { status }) => None,
         Err(gpp::Error::FileError { filename, line, error }) => None,
         Err(gpp::Error::FromUtf8Error(e)) => None,
         Err(gpp::Error::InvalidCommand { command_name }) => None,
         Err(gpp::Error::PipeFailed) => None,
         Err(gpp::Error::TooManyParameters { command }) => None,
         Err(gpp::Error::UnexpectedCommand { command }) => None,
         Err(gpp::Error::IoError(e)) => None,
      }
   }
}
pub struct ShaderMacro<'a> {
   name: &'a str,
   value: &'a str,
}

impl<'a> ShaderMacro<'a> {
   pub fn new(name: &'a str, value: &'a str) -> Option<ShaderMacro<'a>> {
      (name.len() >= value.len())
         .then_some(ShaderMacro{ name, value })
   }
}

// pub fn preprocess_shader(shader_code: &str, macros: &[ShaderMacro]) -> String {
//    shader_code
//       .lines()
//       .filter_map(|line|
//          macros.iter()
//             .any(|m| line.contains(m.name))
//       )
//       map(|line| line)
//       .collect::<String>()
// }