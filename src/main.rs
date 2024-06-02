use std::{collections::HashMap, process::Command};

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

    let mut _changes_per_month: HashMap<String, u32> = HashMap::new();

    // TODO: Be more functional
    for commit in commits {
        let lines: Vec<&str> = commit.lines().collect();
        // println!("lines: {:?}", lines);

        // TODO: Be more functional
        for line in &lines[1..] {
            let stats: Vec<&str> = line.split_whitespace().collect();
            let addition = stats[0].parse::<u8>().unwrap_or(0);
            let deletion = stats[1].parse::<u8>().unwrap_or(0);

            println!("{:?} {}: {}", addition, deletion, addition - deletion);
        }
    }
}
