use std::{
    io::{self, Write},
    process::Child,
};

pub struct Job {
    pub number: usize,
    pub child: Child,
    pub command: String,
    status: JobStatus,
}

pub struct Jobs {
    jobs: Vec<Job>,
    next_number: usize,
}

impl Jobs {
    pub fn add(&mut self, child: Child, command_string: &str) -> usize {
        let number = self.next_number;
        self.next_number += 1;
        let command = command_string
            .strip_suffix(" &")
            .unwrap_or(command_string)
            .to_string();

        self.jobs.push(Job {
            number,
            child,
            command,
            status: JobStatus::Running,
        });
        number
    }

    pub fn list(&mut self, out: &mut dyn Write) -> io::Result<()> {
        let job_count = self.jobs.len();
        for (index, job) in self.jobs.iter_mut().enumerate() {
            if job.child.try_wait()?.is_some() {
                job.status = JobStatus::Done;
            }
            let marker = if index == job_count - 1 {
                "+"
            } else if index == job_count - 2 {
                "-"
            } else {
                " "
            };
            let status = match job.status {
                JobStatus::Running => "Running",
                JobStatus::Done => "Done",
            };
            let command = match job.status {
                JobStatus::Running => format!("{} &", job.command),
                JobStatus::Done => job.command.clone(),
            };
            writeln!(
                out,
                "[{}]{}  {:<24} {}",
                job.number, marker, status, command
            )?;
        }
        self.jobs.retain(|job| job.status == JobStatus::Running);
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

#[derive(PartialEq)]
enum JobStatus {
    Running,
    Done,
}
