mod table;

use chrono::{Datelike, NaiveDate};
use clap::{Parser, ValueEnum};
use std::io::{stdout, Write};
use std::{collections::HashMap, process::Command};

#[derive(Debug, Clone, ValueEnum)]
enum GroupBy {
    Year,
    Month,
}

#[derive(Parser, Debug)]
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
}

fn main() {
    let stdout = stdout();
    let mut handle = stdout.lock();

    let args = Args::parse();
    let log = Command::new("git")
        .arg("log")
        .arg("--numstat")
        // Format the output to be easily parsable
        // https://git-scm.com/docs/pretty-formats
        .arg("--pretty=format:%h|%aI")
        .args(["--author", &args.author])
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

    let mut parsed_commits: Vec<(&str, NaiveDate, (i32, i32))> = Vec::new();

    for commit in &commits {
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

        let addition_deletion: (i32, i32) = lines[1..].iter().fold((0, 0), |acc, line| {
            let stats: Vec<&str> = line.split_whitespace().collect();
            let addition = stats[0].parse::<i32>().unwrap_or(0);
            let deletion = stats[1].parse::<i32>().unwrap_or(0);

            (acc.0 + addition, acc.1 + deletion)
        });

        parsed_commits.push((hash, date, addition_deletion));
    }

    if args.verbose {
        writeln!(handle, "{:?}", parsed_commits).unwrap();
    }

    match args.group {
        GroupBy::Year => {
            println!("year {:?}", group_by_year(&parsed_commits));
        }
        GroupBy::Month => {
            println!("month {:?}", group_by_month(&parsed_commits));
        }
    }
}

/**
 * Get the last day of the year
 */
fn last_day_of_year(year: i32) -> NaiveDate {
    let date = NaiveDate::from_ymd_opt(year + 1, 1, 1).expect("Failed to create Date");
    date.pred_opt().unwrap()
}

/**
 * Get the last day of the month
 */
fn last_day_of_month(year: i32, month: u32) -> NaiveDate {
    let date = NaiveDate::from_ymd_opt(year, month + 1, 1).expect("Failed to create Date");
    date.pred_opt().unwrap()
}

// TODO: Needs tests
/**
 * Group the commits by year
 */
fn group_by_year(data: &[(&str, NaiveDate, (i32, i32))]) -> HashMap<NaiveDate, (i32, i32)> {
    let mut grouped_data: HashMap<NaiveDate, (i32, i32)> = HashMap::new();

    for (_, date, (addition, deletion)) in data {
        let year = date.year();
        let entry = grouped_data.entry(last_day_of_year(year)).or_insert((0, 0));
        entry.0 += addition;
        entry.1 += deletion;
    }

    grouped_data
}

/**
 * Group the commits by month
 */
fn group_by_month(data: &[(&str, NaiveDate, (i32, i32))]) -> HashMap<NaiveDate, (i32, i32)> {
    let mut grouped_data: HashMap<NaiveDate, (i32, i32)> = HashMap::new();

    for (_, date, (addition, deletion)) in data {
        let year = date.year();
        let month = date.month();
        let entry = grouped_data
            .entry(last_day_of_month(year, month))
            .or_insert((0, 0));
        entry.0 += addition;
        entry.1 += deletion;
    }

    grouped_data
}
