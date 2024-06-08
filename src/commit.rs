use chrono::NaiveDate;

use crate::stat::{Stat, StateAccess};

#[derive(Debug, Clone)]
pub struct Commit {
    // TODO: SHA1 hash is 40 characters long
    hash: String,
    date: NaiveDate,
    stat: Stat,
}

impl Commit {
    pub fn new(hash: String, date: NaiveDate, insertion: u32, deletion: u32) -> Commit {
        Commit {
            hash,
            date,
            stat: Stat::new(insertion, deletion),
        }
    }

    pub fn get_date(&self) -> NaiveDate {
        self.date
    }
}

impl StateAccess for Commit {
    fn get_stat(&self) -> &Stat {
        &self.stat
    }
}

#[derive(Debug, Default)]
pub struct GroupedCommit {
    commits: Vec<Commit>,
    stat: Stat,
}

impl GroupedCommit {
    pub fn new(commits: Vec<Commit>) -> GroupedCommit {
        let (insertion, deletion) =
            &commits
                .iter()
                .fold((0, 0), |(acc_insertion, acc_deletion), commit| {
                    (
                        acc_insertion + commit.get_insertion(),
                        acc_deletion + commit.get_deletion(),
                    )
                });

        GroupedCommit {
            commits,
            stat: Stat::new(*insertion, *deletion),
        }
    }

    pub fn add_commits(&mut self, commit: Commit) -> Self {
        let mut commits = self.commits.clone();
        commits.push(commit);

        GroupedCommit::new(commits)
    }
}

impl StateAccess for GroupedCommit {
    fn get_stat(&self) -> &Stat {
        &self.stat
    }
}
