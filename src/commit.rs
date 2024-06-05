use chrono::NaiveDate;

#[derive(Debug, Clone)]
pub struct Commit {
    // TODO: SHA1 hash is 40 characters long
    hash: String,
    date: NaiveDate,
    // TODO: Try readonly crate
    addition: u32,
    deletion: u32,
    change_delta: i32,
}

impl Commit {
    pub fn new(hash: String, date: NaiveDate, addition: u32, deletion: u32) -> Commit {
        Commit {
            hash,
            date,
            addition,
            deletion,
            change_delta: addition as i32 - deletion as i32,
        }
    }

    pub fn get_date(&self) -> NaiveDate {
        self.date
    }

    pub fn get_addition(&self) -> u32 {
        self.addition
    }

    pub fn get_deletion(&self) -> u32 {
        self.deletion
    }

    pub fn get_change_delta(&self) -> i32 {
        self.change_delta
    }
}

#[derive(Debug, Default)]
pub struct GroupedCommit {
    commits: Vec<Commit>,
    addition: u32,
    deletion: u32,
    change_delta: i32,
}

impl GroupedCommit {
    pub fn new(commits: Vec<Commit>) -> GroupedCommit {
        let (addition, deletion, change_delta) = &commits.iter().fold(
            (0, 0, 0),
            |(acc_addition, acc_deletion, acc_change_delta), commit| {
                (
                    acc_addition + commit.get_addition(),
                    acc_deletion + commit.get_deletion(),
                    acc_change_delta + commit.get_change_delta(),
                )
            },
        );

        GroupedCommit {
            commits,
            addition: *addition,
            deletion: *deletion,
            change_delta: *change_delta,
        }
    }

    pub fn get_addition(&self) -> u32 {
        self.addition
    }

    pub fn get_deletion(&self) -> u32 {
        self.deletion
    }

    pub fn get_change_delta(&self) -> i32 {
        self.change_delta
    }

    pub fn add_commits(&mut self, commit: Commit) -> Self {
        let mut commits = self.commits.clone();
        commits.push(commit);

        GroupedCommit::new(commits)
    }
}
