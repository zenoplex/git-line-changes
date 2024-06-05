use chrono::NaiveDate;

#[derive(Debug, Clone)]
pub struct Commit {
    // TODO: SHA1 hash is 40 characters long
    hash: String,
    date: NaiveDate,
    // TODO: Try readonly crate
    addition: u32,
    deletion: u32,
}

impl Commit {
    pub fn new(hash: String, date: NaiveDate, addition: u32, deletion: u32) -> Commit {
        Commit {
            hash,
            date,
            addition,
            deletion,
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

    /**
     * Get the calculated line changes of the commit
     */
    fn calc_line_changes(&self) -> i32 {
        self.addition as i32 - self.deletion as i32
    }
}
