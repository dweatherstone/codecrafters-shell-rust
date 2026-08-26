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

pub struct DisplayJob {
    pub number: usize,
    pub command: String,
    pub marker: Option<char>,
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
        let mut all_jobs = self.reap()?;
        all_jobs.extend(self.jobs.iter().map(|job| DisplayJob {
            number: job.number,
            command: format!("{} &", job.command),
            marker: None,
            status: JobStatus::Running,
        }));
        all_jobs.sort_by_key(|job| job.number);
        let job_count = all_jobs.len();
        for (index, job) in all_jobs.iter().enumerate() {
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
            writeln!(
                out,
                "[{}]{}  {:<24} {}",
                job.number, marker, status, job.command
            )?;
        }

        Ok(())
    }

    pub fn reap(&mut self) -> io::Result<Vec<DisplayJob>> {
        let mut done_jobs = Vec::new();
        let job_count = self.jobs.len();
        for (index, job) in self.jobs.iter_mut().enumerate() {
            let marker = if index == job_count - 1 {
                Some('+')
            } else if index == job_count - 2 {
                Some('-')
            } else {
                None
            };
            if job.child.try_wait()?.is_some() {
                job.status = JobStatus::Done;
                done_jobs.push(DisplayJob {
                    number: job.number,
                    command: job.command.clone(),
                    marker,
                    status: JobStatus::Done,
                });
            }
        }
        self.jobs.retain(|job| job.status == JobStatus::Running);
        Ok(done_jobs)
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
