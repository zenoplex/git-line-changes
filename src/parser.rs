use std::collections::HashMap;

use chrono::{Datelike, NaiveDate};
use once_cell::sync::Lazy;
use regex::Regex;

use crate::commit::{Commit, GroupedCommit};
use crate::utils::{last_day_of_month, last_day_of_year};

static INSERTION_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?P<insertions>\d+) insertion(s)?\(\+\)").unwrap());
static DELETION_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?P<deletions>\d+) deletion(s)?\(-\)").unwrap());

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
    fn parse_insertions_deletions(str: &str) -> (u32, u32) {
        let insertions = INSERTION_REGEX
            .captures(str)
            .and_then(|cap| cap.name("insertions")?.as_str().parse().ok())
            .unwrap_or(0);

        let deletions = DELETION_REGEX
            .captures(str)
            .and_then(|cap| cap.name("deletions")?.as_str().parse().ok())
            .unwrap_or(0);

        (insertions, deletions)
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

            // Drop commits without stats
            let Some(stats) = lines.get(1) else {
                continue;
            };

            let insertions_deletions = Self::parse_insertions_deletions(stats);
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

#[cfg(test)]
mod tests {
    use super::*;

    const STDOUT: &str =
        "<<COMMIT>>|fc557a8860c77a7d9f762c4d64bcc7b1e9352356|2024-04-08T11:24:15+09:00
 1 file changed, 6 deletions(-)

<<COMMIT>>|88521e6ea795fe68b7b8f0389f31c725da768511|2024-04-08T11:24:01+09:00
 1 file changed, 44 insertions(+)

<<COMMIT>>|9ce6f7bba98296aaf20cd05e51f645e16e2ceb30|2024-04-03T22:05:08+09:00
 2 files changed, 6 insertions(+), 44 deletions(-)

<<COMMIT>>|d69fd5b443b12f48380a1752b835222986094eb7|2024-05-02T19:02:27+09:00
 1 file changed, 1 insertion(+), 1 deletion(-)

<<COMMIT>>|a3a975eedb3fffe82a079f8ac5301c445f1a5056|2022-06-02T19:02:27+09:00
 1 file changed, 66 insertions(+), 2 deletions(-)
";

    #[test]
    fn parses_stdout() {
        let parser = LogParser::from(STDOUT);
        assert_eq!(parser.get_commits().len(), 5);
    }

    #[test]
    fn empty_stat_skipped() {
        let stdout = "<<COMMIT>>|82b3ebd9502410c9d74e57a6d677041055decbfd|2024-03-12T12:06:54+09:00
<<COMMIT>>|0bdf1fb931b852af391aa4d1e1341ef38dc7da4b|2024-03-12T11:39:19+09:00
 1 file changed, 72 insertions(+), 3 deletions(-)";
        let parser = LogParser::from(stdout);
        assert_eq!(parser.get_commits().len(), 1);
    }

    #[test]
    fn group_by_month() {
        let parser = LogParser::from(STDOUT);
        let out = parser.group_by(&LogGroupBy::Month);
        assert_eq!(out.len(), 3);

        let test_cases = [
            ((2022, 6, 30), (66, 2)),
            ((2024, 4, 30), (50, 50)),
            ((2024, 5, 31), (1, 1)),
        ];

        for (index, (date, (insertion, deletion))) in test_cases.iter().enumerate() {
            let naive_date = NaiveDate::from_ymd_opt(date.0, date.1, date.2).unwrap();

            assert_eq!(out[index].0, naive_date);
            assert_eq!(out[index].1.get_addition(), *insertion);
            assert_eq!(out[index].1.get_deletion(), *deletion);
        }
    }

    #[test]
    fn group_by_year() {
        let parser = LogParser::from(STDOUT);
        let out = parser.group_by(&LogGroupBy::Year);
        assert_eq!(out.len(), 2);

        let test_cases = [((2022, 12, 31), (66, 2)), ((2024, 12, 31), (51, 51))];

        for (index, (date, (insertion, deletion))) in test_cases.iter().enumerate() {
            let naive_date = NaiveDate::from_ymd_opt(date.0, date.1, date.2).unwrap();

            assert_eq!(out[index].0, naive_date);
            assert_eq!(out[index].1.get_addition(), *insertion);
            assert_eq!(out[index].1.get_deletion(), *deletion);
        }
    }
}
