#![allow(dead_code)]
// #![allow(unused_assignments)]
// #![allow(unused_variables)]
#![allow(unused_imports)]

#[macro_use]
extern crate log;
extern crate simplelog;

use clap::{
    builder::{styling::AnsiColor, Styles},
    crate_description, crate_version, Arg, ArgAction, ArgGroup, ArgMatches, Command as CliCommand,
};
use cliclack::{progress_bar, Theme};
use console::style;

use simplelog::*;

use std::fs::File;
use std::process::{Command, Output, Stdio};

use std::collections::HashMap;

// import the managers module
mod managers;
use managers::{check_apt, check_cargo, check_snap, check_yay};

// TODO enum of managers, maybe create a type for each manager for better handeling

/// Represents the result of a package query.
struct PackageResult {
    manager: String, // apt, yay, go, cargo
    package: String, // name only
    version: String, // version
    desc: String,    // description
    repo: String,    // repo, for yay it's the repo (for go it's the module path?)
    status: String,  // installed, available, not found; TODO: create an enum
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

struct MyTheme;
impl Theme for MyTheme {
    fn spinner_chars(&self) -> String {
        "⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏".into()
    }
}

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

fn cli() -> CliCommand {
    // arg examples
    // .arg(
    //     Arg::new(arg::DRY_RUN)
    //       .short('n')
    //       .long("dry-run")
    //       .env("JUST_DRY_RUN")
    //       .action(ArgAction::SetTrue)
    //       .help("Print what just would do without doing it")
    //       .conflicts_with(arg::QUIET),
    //   )
    //   .arg(
    //     Arg::new(arg::DUMP_FORMAT)
    //       .long("dump-format")
    //       .env("JUST_DUMP_FORMAT")
    //       .action(ArgAction::Set)
    //       .value_parser(clap::value_parser!(DumpFormat))
    //       .default_value("just")
    //       .value_name("FORMAT")
    //       .help("Dump justfile as <FORMAT>"),
    //   )

    CliCommand::new("boss")
        .about(crate_description!())
        .version(crate_version!())
        .styles(
            Styles::styled()
                .header(AnsiColor::Green.on_default().bold())
                .usage(AnsiColor::Green.on_default().bold())
                .literal(AnsiColor::Cyan.on_default().bold())
                .placeholder(AnsiColor::Cyan.on_default()),
        )
        .arg_required_else_help(true)
        .arg(
            Arg::new("package")
                .num_args(1..)
                .help("The package(s) to check")
                .action(ArgAction::Append)
                .conflicts_with("interactive"),
        )
        .arg(
            Arg::new("quiet")
                .short('q')
                .long("quiet")
                .help("Stay quiet, only return 0 or 1")
                .action(ArgAction::SetTrue)
                .conflicts_with("interactive"),
        )
        .arg(
            Arg::new("interactive")
                .short('i')
                .long("interactive")
                .help("Run in interactive mode and prompt the user for packges")
                .action(ArgAction::SetTrue)
                .conflicts_with("quiet")
                .conflicts_with("package"),
        )
}

fn main() -> std::io::Result<()> {
    let config = ConfigBuilder::new()
        .set_thread_level(LevelFilter::Error)
        .set_target_level(LevelFilter::Error)
        .build();
    CombinedLogger::init(vec![
        TermLogger::new(
            LevelFilter::Error,
            config.clone(),
            TerminalMode::Mixed,
            ColorChoice::Auto,
        ),
        #[cfg(debug_assertions)]
        WriteLogger::new(
            LevelFilter::Debug,
            config,
            File::create("boss.log").unwrap(),
        ),
    ])
    .unwrap();

    let matches = cli().try_get_matches().unwrap_or_else(|e| e.exit());
    debug!("Matches: {:?}", matches);

    let packages: Vec<&str> = match matches.get_many::<String>("package") {
        Some(vals) => vals.map(|v| v.as_str()).collect(),
        None => {
            eprintln!("No packages provided.");
            std::process::exit(1);
        }
    };
    debug!("Packages: {:?}", packages);

    let is_interactive = matches.get_flag("interactive");
    let stay_quiet = matches.get_flag("quiet");

    // check for missing args
    // if packages.is_empty() && !is_interactive {
    //     return Err(std::io::Error::new(
    //         std::io::ErrorKind::Other,
    //         "Missing package(s) or --interactive flag",
    //     ));
    // }

    // check if package and interactive flags are set
    // if args.package.is_some() && args.interactive {
    //     return Err(std::io::Error::new(
    //         std::io::ErrorKind::Other,
    //         "Cannot use both package name and --interactive flag",
    //     ));
    // }

    // check for colliding args
    // if args.quiet && args.interactive {
    //     return Err(std::io::Error::new(
    //         std::io::ErrorKind::Other,
    //         "Cannot use both --quiet and --interactive flags",
    //     ));
    // }

    if stay_quiet {
        let package_name = packages.get(0).unwrap().to_string();
        let check_functions = get_check_functions();
        let mut results = vec![];

        for manager in get_installed_managers() {
            if let Some(check_fn) = check_functions.get(manager) {
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
            return Ok(());
        }
    }

    println!();

    cliclack::set_theme(MyTheme);
    cliclack::intro(style(" boss ").on_cyan().black())?;

    // get managers
    let installed_managers = get_installed_managers();

    cliclack::log::remark(format!(
        "Managers: {} ({})",
        installed_managers.join(", "),
        installed_managers.len()
    ))?;

    // only get the first package in the list for now
    let package_name: String = if is_interactive {
        match cliclack::input("Enter package name: ").interact() {
            Ok(name) => name,
            Err(e) => {
                cliclack::log::error(e)?;
                return Ok(());
            }
        }
    } else {
        packages.get(0).unwrap().to_string()
    };

    cliclack::log::remark(format!(
        "Package: {}",
        style(&package_name).on_black().cyan()
    ))?;

    let progress = progress_bar(installed_managers.len() as u64)
        .with_template("{msg:20} {bar:15.cyan/blue} {pos}/{len} [{elapsed}]");
    progress.start("Fetching...");

    let check_functions = get_check_functions();
    let mut results = vec![];

    for manager in &installed_managers {
        if let Some(check_fn) = check_functions.get(*manager) {
            progress.set_message(format!("Checking {}...", manager));
            match check_fn(&package_name) {
                Ok(result) => {
                    results.push(result);
                    progress.inc(1);
                }
                Err(e) => {
                    progress.error(&e);
                    cliclack::log::error(e)?;
                }
            }
        }
    }

    progress.stop("Results:");
    print_result(sort_results(results))?;
    cliclack::outro("Done!")?;

    Ok(())
}
