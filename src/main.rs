mod commit;
mod commits;
mod table;
mod utils;

use crate::commits::Commits;
use clap::Parser;
use std::io::{stdout, Write};
use std::process::Command;
use table::Table;

#[derive(Debug, Clone, clap::ValueEnum)]
enum GroupBy {
    Year,
    Month,
}

#[derive(clap::Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Name of the person to log
    #[arg(short, long)]
    author: String,

    /// Show verbose output
    #[arg(short, long)]
    verbose: bool,

    /// Group commits by year and month
    #[arg(short, long, default_value = "month")]
    group: GroupBy,

    /// Filter commits after a specific date
    #[arg(long)]
    after: Option<String>,

    /// Filter commits before a specific date
    #[arg(long)]
    before: Option<String>,
}

fn main() {
    let stdout = stdout();
    let mut handle = stdout.lock();

    let args = Args::parse();
    let log = Command::new("git")
        .arg("log")
        .arg("--numstat")
        .arg("--no-merges")
        // Format the output to be easily parsable
        // https://git-scm.com/docs/pretty-formats
        .arg("--pretty=format:%H|%aI")
        .args(["--author", &args.author])
        // .args(["--after", &args.after])
        // .args(["--before", &args.before])
        .output()
        .expect("Failed to execute git log command");

    let output = String::from_utf8(log.stdout).expect("Invalid UTF-8");
    let commits = output.split("\n\n").collect::<Vec<&str>>();

    writeln!(
        handle,
        "Found total of {} commits by {}",
        &commits.len(),
        &args.author
    )
    .unwrap();

    let parser = Commits::from(commits);

    let rows = match args.group {
        GroupBy::Year => parser
            .group_by_year()
            .iter()
            .map(|(date, grouped_commit)| {
                vec![
                    date.format("%Y").to_string(),
                    grouped_commit.get_addition().to_string(),
                    grouped_commit.get_deletion().to_string(),
                ]
            })
            .collect(),
        GroupBy::Month => parser
            .group_by_month()
            .iter()
            .map(|(date, grouped_commit)| {
                vec![
                    date.format("%Y-%m").to_string(),
                    grouped_commit.get_addition().to_string(),
                    grouped_commit.get_deletion().to_string(),
                ]
            })
            .collect(),
    };

    let table = Table::new(
        vec![
            "Date".to_string(),
            "Addition".to_string(),
            "Deletion".to_string(),
        ],
        rows,
    );
    table.render();
}
