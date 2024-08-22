#![allow(dead_code)]
#![allow(unused_assignments)]
#![allow(unused_variables)]

#[macro_use]
extern crate log;
extern crate simplelog;

use clap::Parser;
use console::style;

use simplelog::*;

use std::fs::File;
use std::process::{Command, Output, Stdio};

use std::collections::HashMap;

// import the managers module
mod managers;
use managers::{check_apt, check_cargo, check_go, check_snap, check_yay};

/// Represents the result of a package query.
struct PackageResult {
    manager: String, // apt, yay, go, cargo
    package: String, // name only
    version: String, // version
    desc: String,    // description
    repo: String,    // repo, for yay it's the repo (for go it's the module path?)
    status: String,  // installed, available, not found
}

impl PackageResult {
    fn some(
        manager: &str,
        package: &str,
        status: &str,
        version: &str,
        desc: &str,
        repo: &str,
    ) -> Self {
        PackageResult {
            manager: manager.to_string(),
            package: package.to_string(),
            status: status.to_string(),
            version: version.to_string(),
            desc: desc.to_string(),
            repo: repo.to_string(),
        }
    }

    fn none(manager: &str, package: &str) -> Self {
        PackageResult {
            manager: manager.to_string(),
            package: package.to_string(),
            status: "not found".to_string(),
            version: "".to_string(),
            desc: "".to_string(),
            repo: "".to_string(),
        }
    }
}

/// Type alias for the check function signature.
type CheckFn = fn(&str) -> Result<PackageResult, String>;

/// Reduces consecutive whitespace characters in a string to a single space.
fn reduce_whitespace(s: String) -> String {
    // s.split_whitespace().collect::<Vec<&str>>().join(" ")
    let mut result = String::new();
    let mut chars = s.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch.is_whitespace() {
            result.push(' ');
            // Skip all subsequent whitespace characters
            while let Some(&next_ch) = chars.peek() {
                if next_ch.is_whitespace() {
                    chars.next();
                } else {
                    break;
                }
            }
        } else {
            result.push(ch);
        }
    }

    result
}

/// Returns a list of installed package managers.
fn get_installed_managers() -> Vec<&'static str> {
    let managers = vec!["snap", "apt", "yay", "cargo", "go"];
    let mut installed_managers = Vec::new();

    for manager in &managers {
        match Command::new("which")
            .arg(manager)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
        {
            Ok(status) => {
                if status.success() {
                    installed_managers.push(*manager)
                }
            }
            Err(_) => {}
        }
    }

    installed_managers
}

/// Checks the output of a command and returns the lines of output if successful.
fn check_output(output: Output) -> Result<Vec<String>, String> {
    if output.stdout.is_empty() {
        warn!("stdout is empty");
        return Err("stdout is empty".to_string());
    }

    let stdout: Vec<u8> = output.stdout;
    let stdout_string = match String::from_utf8(stdout) {
        Ok(stdout_string) => stdout_string,
        Err(e) => return Err(e.to_string()),
    };

    let lines: Vec<String> = stdout_string
        .split('\n')
        .filter(|line| !line.is_empty())
        .map(|line| line.to_string())
        .collect();

    Result::Ok(lines)
}

/// Returns a map of package manager names to their corresponding check functions.
fn get_check_functions() -> HashMap<&'static str, CheckFn> {
    let mut map: HashMap<&'static str, CheckFn> = HashMap::new();

    map.insert("apt", check_apt as CheckFn);
    map.insert("yay", check_yay as CheckFn);
    map.insert("snap", check_snap as CheckFn);
    map.insert("cargo", check_cargo as CheckFn);

    map
}

/// Returns a vector of sorted package results.
fn sort_results(results: Vec<PackageResult>) -> Vec<PackageResult> {
    let mut installed: Vec<PackageResult> = Vec::new();
    let mut available: Vec<PackageResult> = Vec::new();
    let mut not_found: Vec<PackageResult> = Vec::new();

    for result in results {
        if result.status.contains("installed") {
            installed.push(result);
        } else if result.status == "available" {
            available.push(result);
        } else {
            not_found.push(result);
        }
    }

    installed.append(&mut available);
    installed.append(&mut not_found);
    installed
}

/// Prints the results to the console using cliclack.
fn print_result(results: Vec<PackageResult>) -> core::result::Result<(), std::io::Error> {
    for result in results {
        if result.status.contains("installed") {
            cliclack::log::success(format!(
                "[ {} ] - [{}] - ({})",
                result.manager, result.status, result.version
            ))?;
        } else if result.status == "available" {
            // cliclack::log::info(format!(
            //     "[ {} ] - [available] - ({})",
            //     result.manager, result.version
            // ))?;
            cliclack::note(
                format!(
                    "[ {} ] - [available] - ({})",
                    result.manager, result.version
                ),
                result.desc,
            )?;
        } else if result.status == "not found" {
            cliclack::log::error(format!("[ {} ] - [not found]", result.manager))?;
        }
    }

    Ok(())
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    package: String,
    #[arg(short, long)]
    quiet: bool,
}

fn main() -> std::io::Result<()> {
    CombinedLogger::init(vec![
        TermLogger::new(
            LevelFilter::Error,
            Config::default(),
            TerminalMode::Mixed,
            ColorChoice::Auto,
        ),
        #[cfg(debug_assertions)]
        WriteLogger::new(
            LevelFilter::Debug,
            Config::default(),
            File::create("boss.log").unwrap(),
        ),
    ])
    .unwrap();

    let args = Args::parse();
    debug!("{:?}", args);

    let package_name = args.package;
    let installed_managers = get_installed_managers();

    match args.quiet {
        true => {
            let check_functions = get_check_functions();
            let mut results = vec![];

            for manager in &installed_managers {
                if let Some(check_fn) = check_functions.get(*manager) {
                    match check_fn(&package_name) {
                        Ok(result) => results.push(result),
                        Err(e) => {
                            return Err(std::io::Error::new(
                                std::io::ErrorKind::Other,
                                format!("Error: {}", e),
                            ));
                        }
                    }
                }
            }

            if results.iter().all(|result| result.status == "not found") {
                std::process::exit(1);
            } else {
                Ok(())
            }
        }
        false => {
            println!();
            cliclack::intro(style(" boss ").on_cyan().black())?;

            cliclack::log::remark(format!("Managers: {}", installed_managers.join(", ")))?;
            cliclack::log::remark(format!("Package: {}", package_name))?; // Packages

            let spinner = cliclack::spinner();
            spinner.start("Fetching...");

            let check_functions = get_check_functions();
            let mut results = vec![];

            for manager in &installed_managers {
                if let Some(check_fn) = check_functions.get(*manager) {
                    match check_fn(&package_name) {
                        Ok(result) => results.push(result),
                        Err(e) => {
                            spinner.error(&e);
                            cliclack::log::error(e)?;
                        }
                    }
                }
            }

            spinner.stop("Results:");

            print_result(sort_results(results))?;

            cliclack::outro("Done!")?;

            Ok(())
        }
    }
}
