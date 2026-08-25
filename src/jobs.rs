use std::process::Child;

pub struct Job {
    pub number: usize,
    pub pid: u32,
    pub child: Child,
}

pub struct Jobs {
    jobs: Vec<Job>,
    next_number: usize,
}

impl Jobs {
    pub fn add(&mut self, child: Child) -> usize {
        let number = self.next_number;
        self.next_number += 1;

        self.jobs.push(Job {
            number,
            pid: child.id(),
            child,
        });
        number
    }
}

impl Default for Jobs {
    fn default() -> Self {
        Self {
            jobs: Vec::new(),
            next_number: 1,
        }
    }
}
