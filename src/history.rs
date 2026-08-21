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
}
