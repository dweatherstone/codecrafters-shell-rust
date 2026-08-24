use std::{
    collections::HashMap,
    fs::{OpenOptions, read_to_string},
    io::{self, Write},
    path::{Path, PathBuf},
};

#[derive(Default)]
pub struct History {
    entries: Vec<String>,
    append_positions: HashMap<PathBuf, usize>,
}

impl History {
    pub fn load_from_file(&mut self, path: &Path) -> io::Result<()> {
        let contents = read_to_string(path)?;

        for line in contents.lines() {
            self.add(line.to_string());
        }

        Ok(())
    }

    pub fn add(&mut self, command: String) {
        self.entries.push(command);
    }

    pub fn iter(&self) -> impl Iterator<Item = (usize, &str)> {
        self.entries
            .iter()
            .enumerate()
            .map(|(i, command)| (i + 1, command.as_str()))
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn last(&self, count: usize) -> impl Iterator<Item = (usize, &str)> {
        let start = self.entries.len().saturating_sub(count);

        self.entries
            .iter()
            .enumerate()
            .skip(start)
            .map(|(idx, command)| (idx + 1, command.as_str()))
    }

    pub fn append_to_file(&mut self, path: &Path) -> io::Result<()> {
        let start = self.append_positions.get(path).copied().unwrap_or(0);

        let mut file = OpenOptions::new().create(true).append(true).open(path)?;

        for command in self.entries.iter().skip(start) {
            writeln!(file, "{}", command)?;
        }

        self.mark_appended(path.to_owned());

        Ok(())
    }

    pub fn mark_appended(&mut self, path: PathBuf) {
        self.append_positions.insert(path, self.entries.len());
    }
}
