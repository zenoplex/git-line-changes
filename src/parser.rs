use std::collections::HashMap;

use chrono::{Datelike, NaiveDate};

use crate::commit::{Commit, GroupedCommit};
use crate::utils::{last_day_of_month, last_day_of_year};

// TODO: Needs tests

/// Immutable struct to parse git log output.
#[derive(Debug, Default)]
pub struct LogParser {
    /// List of commits.
    commits: Vec<Commit>,
}

/// Parse git log output and return instance
impl From<&str> for LogParser {
    fn from(stdout: &str) -> Self {
        let raw_commits = LogParser::split_stdout_to_commits(stdout);
        let commits = LogParser::parse(&raw_commits);
        LogParser { commits }
    }
}

impl LogParser {
    // Format the output to be easily parsable
    // https://git-scm.com/docs/pretty-formats
    pub const GIT_LOG_ARGS: [&'static str; 3] = [
        "--numstat",
        "--no-merges",
        // Adding <<COMMIT>> to separate commits since stats output can be empty which makes it hard to distinguish between commits.
        // Separating commits with \n\n does not work in some cases (ie: allow-empty commits).
        "--pretty=format:<<COMMIT>>|%H|%aI",
    ];

    /// Split stdout to collections of commits
    fn split_stdout_to_commits(stdout: &str) -> Vec<&str> {
        // TODO: Make <<COMMIT>> a constant
        stdout
            .split("<<COMMIT>>")
            .map(|s| s.trim())
            .collect::<Vec<&str>>()
    }

    /// Parse git log output.
    /// git log needs to be outputted with the specific format.
    fn parse(raw_commits: &Vec<&str>) -> Vec<Commit> {
        let mut commits: Vec<Commit> = Vec::new();

        for commit in raw_commits {
            let lines: Vec<&str> = commit.lines().collect();
            if lines.is_empty() {
                continue;
            }

            let hash_date: Vec<&str> = lines[0].split('|').skip(1).collect();
            let hash = hash_date[0];
            let commit_date = NaiveDate::parse_from_str(hash_date[1], "%Y-%m-%dT%H:%M:%S%z");
            let date = match commit_date {
                Ok(date) => date,
                Err(_) => {
                    println!("Error parsing date: {}", hash_date[1]);
                    continue;
                }
            };

            // TODO: Rather than parsing output of --numstat, using output of --shortstat option would make below code simpler
            let addition_deletion: (u32, u32) = lines[1..].iter().fold((0, 0), |acc, line| {
                let stats: Vec<&str> = line.split_whitespace().collect();
                let addition = stats[0].parse::<u32>().unwrap_or(0);
                let deletion = stats[1].parse::<u32>().unwrap_or(0);

                (acc.0 + addition, acc.1 + deletion)
            });

            let commit = Commit::new(hash.into(), date, addition_deletion.0, addition_deletion.1);
            commits.push(commit);
        }

        commits
    }

    /// Get the list of commits
    pub fn get_commits(&self) -> &Vec<Commit> {
        &self.commits
    }

    // TODO: Refactor group_by_year and group_by_month to use a single function
    /// Group the commits by year
    pub fn group_by_year(&self) -> Vec<(NaiveDate, GroupedCommit)> {
        let mut grouped_data: HashMap<NaiveDate, GroupedCommit> = HashMap::new();

        for commit in &self.commits {
            let date = commit.get_date();
            let year = date.year();
            let entry = grouped_data.entry(last_day_of_year(year)).or_default();
            let a = entry.add_commits(commit.clone());
            grouped_data.insert(last_day_of_year(year), a);
        }

        let mut list: Vec<(NaiveDate, GroupedCommit)> = grouped_data.into_iter().collect();
        list.sort_by(|a, b| a.0.cmp(&b.0));
        list
    }

    /// Group the commits by month
    pub fn group_by_month(&self) -> Vec<(NaiveDate, GroupedCommit)> {
        let mut grouped_data: HashMap<NaiveDate, GroupedCommit> = HashMap::new();

        for commit in &self.commits {
            let date = commit.get_date();
            let year = date.year();
            let month = date.month();
            let entry = grouped_data
                .entry(last_day_of_month(year, month))
                .or_default();

            let a = entry.add_commits(commit.clone());
            grouped_data.insert(last_day_of_month(year, month), a);
        }

        let mut list: Vec<(NaiveDate, GroupedCommit)> = grouped_data.into_iter().collect();
        list.sort_by(|a, b| a.0.cmp(&b.0));
        list
    }
}
