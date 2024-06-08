mod commit;
mod parser;
mod table;
mod utils;

use crate::parser::{LogGroupBy, LogParser};
use crate::table::Table;
use clap::Parser;
use std::io::{stdout, Write};
use std::process::Command;

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
    let mut log = Command::new("git");
    log.arg("log")
        .args(LogParser::get_git_log_args())
        .args(["--author", &args.author]);

    if let Some(after) = &args.after {
        log.args(["--after", after]);
    }
    if let Some(before) = &args.before {
        log.args(["--before", before]);
    }

    let output = log.output().expect("Failed to execute git log command");
    let stdout = String::from_utf8(output.stdout).expect("Invalid UTF-8");
    let parser = LogParser::from(stdout.as_str());

    let mut sum_insertion = 0;
    let mut sum_deletion = 0;
    let mut sum_change_delta = 0;

    let log_group = match args.group {
        GroupBy::Year => LogGroupBy::Year,
        GroupBy::Month => LogGroupBy::Month,
    };

    let rows = parser
        .group_by(&log_group)
        .iter()
        .map(|(date, grouped_commit)| {
            sum_insertion += grouped_commit.get_insertion();
            sum_deletion += grouped_commit.get_deletion();
            sum_change_delta += grouped_commit.get_change_delta();

            vec![
                date.format("%Y").to_string(),
                grouped_commit.get_insertion().to_string(),
                grouped_commit.get_deletion().to_string(),
                grouped_commit.get_change_delta().to_string(),
            ]
        })
        .collect();

    writeln!(
        handle,
        "Found total of {} commits by {}\n",
        &parser.get_commits().len(),
        &args.author
    )
    .unwrap();

    let table = Table::new(
        vec![
            "Date".to_string(),
            "Insertion".to_string(),
            "Deletion".to_string(),
            "Change Delta".to_string(),
        ],
        rows,
    );
    table.render();

    writeln!(
        handle,
        "sum {:?} {:?} {:?}",
        sum_insertion, sum_deletion, sum_change_delta,
    )
    .unwrap();
}
