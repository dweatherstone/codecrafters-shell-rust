use std::path::PathBuf;

#[derive(Debug, PartialEq)]
pub struct Pipeline {
    pub commands: Vec<Command>,
}

#[derive(Debug, PartialEq)]
pub struct Command {
    pub words: Vec<String>,
    pub redirects: Vec<Redirect>,
}

impl Command {
    pub fn program(&self) -> Option<&str> {
        self.words.first().map(String::as_str)
    }

    pub fn args(&self) -> &[String] {
        self.words.get(1..).unwrap_or_default()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Redirect {
    pub fd: u8,
    pub target: PathBuf,
    pub mode: RedirectMode,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RedirectMode {
    Write,
    Append,
}
