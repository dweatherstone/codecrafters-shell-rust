use std::{
    io::{self, Write},
    process::Child,
};

pub struct Job {
    pub number: usize,
    pub child: Child,
    pub command: String,
}

pub struct Jobs {
    jobs: Vec<Job>,
    next_number: usize,
}

impl Jobs {
    pub fn add(&mut self, child: Child, command: String) -> usize {
        let number = self.next_number;
        self.next_number += 1;

        self.jobs.push(Job {
            number,
            child,
            command,
        });
        number
    }

    pub fn list(&self, out: &mut dyn Write) -> io::Result<()> {
        let job_count = self.jobs.len();
        for (index, job) in self.jobs.iter().enumerate() {
            let marker = if index >= job_count - 1 && index == job_count - 1 {
                "+"
            } else if index >= job_count - 2 && index == job_count - 2 {
                "-"
            } else {
                " "
            };
            writeln!(
                out,
                "[{}]{}  {:<24} {}",
                job.number, marker, "Running", job.command
            )?;
        }
        Ok(())
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
