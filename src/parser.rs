use std::collections::HashMap;

use chrono::{Datelike, NaiveDate};
use regex::Regex;

use crate::commit::{Commit, GroupedCommit};
use crate::utils::{last_day_of_month, last_day_of_year};

// Not sure this is the right way to do it trying to store regex
struct LogParserRegex {
    insertion: Regex,
    deletion: Regex,
}

impl LogParserRegex {
    pub fn new() -> Self {
        Self {
            insertion: Regex::new(r"(?P<insertions>\d+) insertions\(\+\)").unwrap(),
            deletion: Regex::new(r"(?P<deletions>\d+) deletions\(-\)").unwrap(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum LogGroupBy {
    Year,
    Month,
}

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
    const COMMIT_SEPARATOR: &'static str = "<<COMMIT>>";

    /// Get git log arguments.
    /// This is required to make git log parsable.
    pub fn get_git_log_args() -> [String; 3] {
        [
            "--shortstat".to_string(),
            "--no-merges".to_string(),
            // Adding <<COMMIT>> to separate commits since stats output can be empty which makes it hard to distinguish between commits.
            // Separating commits with \n\n does not work in some cases (ie: allow-empty commits).
            // https://git-scm.com/docs/pretty-formats
            format!("--pretty=format:{}|%H|%aI", Self::COMMIT_SEPARATOR),
        ]
    }

    /// Split stdout to collections of commits
    fn split_stdout_to_commits(stdout: &str) -> Vec<&str> {
        stdout
            .split(Self::COMMIT_SEPARATOR)
            .map(|s| s.trim())
            .collect::<Vec<&str>>()
    }

    /// Parse insertions and deletions from git log output
    fn parse_insertions_deletions(regex: &LogParserRegex, str: &str) -> (u32, u32) {
        let insertions = regex
            .insertion
            .captures(str)
            // It's unclear from the code why capture name is used here.
            // Intention was not to compile the Regex inside the loop so it's taken out as LogParserRegex.
            // A simple memoization might be better?
            .and_then(|cap| cap.name("insertions")?.as_str().parse().ok())
            .unwrap_or(0);

        let deletions = regex
            .deletion
            .captures(str)
            .and_then(|cap| cap.name("deletions")?.as_str().parse().ok())
            .unwrap_or(0);

        (insertions, deletions)
    }

    /// Parse git log output.
    /// git log needs to be outputted with the specific format.
    fn parse(raw_commits: &Vec<&str>) -> Vec<Commit> {
        let mut commits: Vec<Commit> = Vec::new();
        let regex = LogParserRegex::new();

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

            // Drop commits without stats
            let Some(stats) = lines.get(1) else {
                continue;
            };

            let insertions_deletions = Self::parse_insertions_deletions(&regex, stats);
            let commit = Commit::new(
                hash.into(),
                date,
                insertions_deletions.0,
                insertions_deletions.1,
            );
            commits.push(commit);
        }

        commits
    }

    /// Get the list of commits
    pub fn get_commits(&self) -> &Vec<Commit> {
        &self.commits
    }

    fn create_hash_key(&self, commit: &Commit, group: &LogGroupBy) -> NaiveDate {
        match group {
            LogGroupBy::Year => {
                let date = commit.get_date();
                let year = date.year();
                last_day_of_year(year)
            }
            LogGroupBy::Month => {
                let date = commit.get_date();
                let year = date.year();
                let month = date.month();
                last_day_of_month(year, month)
            }
        }
    }

    /// Group the commits by the given group
    pub fn group_by(&self, group: &LogGroupBy) -> Vec<(NaiveDate, GroupedCommit)> {
        let mut grouped_data: HashMap<NaiveDate, GroupedCommit> = HashMap::new();

        for commit in &self.commits {
            let key = self.create_hash_key(commit, group);
            let entry = grouped_data.entry(key).or_default();
            let grouped_commit = entry.add_commits(commit.clone());
            grouped_data.insert(key, grouped_commit);
        }

        let mut list: Vec<(NaiveDate, GroupedCommit)> = grouped_data.into_iter().collect();
        list.sort_by(|a, b| a.0.cmp(&b.0));
        list
    }
}
