use std::collections::HashMap;

use chrono::{Datelike, NaiveDate};

use crate::commit::Commit;
use crate::utils::{last_day_of_month, last_day_of_year};

// TODO: Needs tests

#[derive(Debug, Default)]
pub struct Commits {
    commits: Vec<Commit>,
}

impl From<Vec<&str>> for Commits {
    fn from(str: Vec<&str>) -> Self {
        let parser = Commits::new();
        let commits = parser._parse(str);
        Commits { commits }
    }
}

impl Commits {
    pub fn new() -> Commits {
        Commits {
            commits: Vec::new(),
        }
    }

    /// Parse git log output
    /// git log needs to be outputted with the specific format.
    fn _parse(&self, orig_commits: Vec<&str>) -> Vec<Commit> {
        let mut commits: Vec<Commit> = Vec::new();

        for commit in &orig_commits {
            let lines: Vec<&str> = commit.lines().collect();
            if lines.is_empty() {
                continue;
            }

            let hash_date: Vec<&str> = lines[0].split('|').collect();
            let hash = hash_date[0];
            let commit_date = NaiveDate::parse_from_str(hash_date[1], "%Y-%m-%dT%H:%M:%S%z");
            let date = match commit_date {
                Ok(date) => date,
                Err(_) => {
                    println!("Error parsing date: {}", hash_date[1]);
                    continue;
                }
            };

            let addition_deletion: (u32, u32) = lines[1..].iter().fold((0, 0), |acc, line| {
                let stats: Vec<&str> = line.split_whitespace().collect();

                // TODO: Hotfix for invalid stats
                if stats.len() < 2 {
                    return acc;
                }

                let addition = stats[0].parse::<u32>().unwrap_or(0);
                let deletion = stats[1].parse::<u32>().unwrap_or(0);

                (acc.0 + addition, acc.1 + deletion)
            });

            let commit = Commit::new(hash.into(), date, addition_deletion.0, addition_deletion.1);
            commits.push(commit);
        }

        commits
    }

    /// Parse the commits
    pub fn parse(&self, orig_commits: Vec<&str>) -> Self {
        let commits = self._parse(orig_commits);
        Commits { commits }
    }

    /// Group the commits by year
    pub fn group_by_year(&self) -> HashMap<NaiveDate, (u32, u32)> {
        let mut grouped_data: HashMap<NaiveDate, (u32, u32)> = HashMap::new();

        for commit in &self.commits {
            let date = commit.get_date();
            let year = date.year();
            let entry = grouped_data.entry(last_day_of_year(year)).or_insert((0, 0));
            entry.0 += commit.get_addition();
            entry.1 += commit.get_deletion();
        }

        grouped_data
    }

    /// Group the commits by month
    pub fn group_by_month(&self) -> HashMap<NaiveDate, (u32, u32)> {
        let mut grouped_data: HashMap<NaiveDate, (u32, u32)> = HashMap::new();

        for commit in &self.commits {
            let date = commit.get_date();
            let year = date.year();
            let month = date.month();
            let entry = grouped_data
                .entry(last_day_of_month(year, month))
                .or_insert((0, 0));
            entry.0 += commit.get_addition();
            entry.1 += commit.get_deletion();
        }

        grouped_data
    }
}
