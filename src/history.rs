#[derive(Default)]
pub struct History {
    entries: Vec<String>,
}

impl History {
    pub fn add(&mut self, command: String) {
        self.entries.push(command);
    }

    pub fn iter(&self) -> impl Iterator<Item = (usize, &str)> {
        self.entries
            .iter()
            .enumerate()
            .map(|(i, command)| (i + 1, command.as_str()))
    }
}
