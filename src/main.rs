use std::process::Command;

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
    let output = Command::new("git")
        .arg("log")
        // .arg("--numstat")
        // Format the output to be easily parsable
        // https://git-scm.com/docs/pretty-formats
        .arg("--pretty=format:%h|%aI")
        .args(["--author", &args.author])
        .output()
        .expect("Failed to execute git log command");

    println!(
        "{:?}",
        String::from_utf8(output.stdout).expect("Invalid UTF-8")
    );
}
