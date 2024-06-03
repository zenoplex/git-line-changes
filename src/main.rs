use std::process::Command;

use chrono::NaiveDate;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Name of the person to log
    #[arg(short, long)]
    author: String,
}

fn main() {
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
    println!(
        "Found total of {} commits by {}",
        commits.len(),
        args.author
    );

    let mut parsed_commits: Vec<(&str, NaiveDate, (i32, i32))> = Vec::new();

    // TODO: Be more functional
    for commit in commits {
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

        // println!("{:?} commit_date: {}", hash_date, date);

        let addition_deletion: (i32, i32) = lines[1..].iter().fold((0, 0), |acc, line| {
            let stats: Vec<&str> = line.split_whitespace().collect();
            let addition = stats[0].parse::<i32>().unwrap_or(0);
            let deletion = stats[1].parse::<i32>().unwrap_or(0);

            (acc.0 + addition, acc.1 + deletion)
        });
        // println!("addition_deletion {:?}", addition_deletion);
        parsed_commits.push((hash, date, addition_deletion));
    }

    println!("{:?}", parsed_commits);
}
