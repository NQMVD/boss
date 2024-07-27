#![allow(dead_code)]
#![allow(unused_assignments)]

use clap::Parser;
use cliclack::{intro, log, outro, spinner};
use console::style;
use std::process::{Command, Output, Stdio};
use strp::*;

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

fn reduce_whitespace(s: String) -> String {
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

fn get_installed_managers() -> Vec<&'static str> {
    let managers = vec!["apt", "yay", "go", "cargo"];
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

// function to check if output is empty, if it is, return Err, else return Ok with a vector of lines
fn check_output(output: Output) -> Result<Vec<String>, String> {
    if output.stdout.is_empty() {
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

// apt search = returns a list like yay but with empty lines in between
// apt show = shows only one package with info, DOESNT show if its installed tho
fn check_apt(package_name: &str) -> Result<PackageResult, String> {
    let output = match Command::new("apt").arg("list").arg("--installed").output() {
        Ok(output) => output,
        Err(e) => return Err(e.to_string()),
    };

    let lines = match check_output(output) {
        Ok(lines) => lines,
        Err(e) => return Err(e),
    };

    let filtered_lines: Vec<String> = lines
        .iter()
        .filter(|line| !line.starts_with("Listing") && !line.starts_with(' ') && !line.is_empty())
        .map(|line| line.to_string())
        .collect();

    for line in &filtered_lines {
        let mut chunks = line.split_whitespace();
        let fullname = chunks.next().expect("fullname should exist");
        let name = fullname
            .split('/')
            .next()
            .expect("fullname should contain a /");
        let version = chunks.next().expect("version should exist");

        if package_name == name {
            return Result::Ok(PackageResult::some(
                "apt",
                name,
                "installed",
                version,
                // TODO: get description
                "",
                "",
            ));
        }
    }

    let output = match Command::new("apt").arg("show").arg(package_name).output() {
        Ok(output) => output,
        Err(e) => return Err(e.to_string()),
    };

    let lines = match check_output(output) {
        Ok(lines) => lines,
        Err(e) => return Err(e),
    };

    let version = lines
        .iter()
        .nth(1)
        .expect("line should exist")
        .split_whitespace()
        .last()
        .expect("line should be splitable");

    // TODO: get description
    let desc = String::from("");

    return Result::Ok(PackageResult::some(
        "apt",
        package_name,
        "available",
        version,
        desc.as_str(),
        "",
    ));

    Result::Ok(PackageResult::none("apt", package_name))
}

fn check_yay(package_name: &str) -> Result<PackageResult, String> {
    let output = Command::new("yay")
        .arg("-Ss")
        .arg(package_name)
        .output()
        .expect("yay should succed at this point");

    let lines = check_output(output).expect("yay should return a list of installed packages");

    let line = lines.iter().nth_back(2).expect("line should exist");
    let mut chunks = line.split_whitespace();
    let fullname = chunks.next().expect("fullname should exist");
    let (repo, name) = fullname
        .split_once('/')
        .expect("fullname should contain a /");
    let version = chunks.next().expect("version should exist");

    if name == package_name {
        let status = if line.contains("Installed") {
            "installed"
        } else {
            "available"
        };

        return Result::Ok(PackageResult::some(
            "yay", fullname, status, version, "", repo,
        ));
    }

    Result::Ok(PackageResult::none("yay", package_name))
}

fn check_cargo(package_name: &str) -> Result<PackageResult, String> {
    // let mut installed = false;
    // 1. search for package in registry to get all informations
    // 2. search for package in installed packages to get installed version
    // 3. if installed, return installed version (and compare to newest version), else return available version

    // query the registry
    let output = Command::new("cargo")
        .arg("search")
        .arg(package_name)
        .output()
        .expect("cargo should succed at this point");

    let lines = check_output(output).expect("cargo should return a list of installed packages");

    // check the first line (maybe check all lines if there are multiple results)
    let line = lines.iter().next().expect("line should exist");
    let reduced_line = reduce_whitespace(line.to_string());
    let (name, version, description): (String, String, String) =
        try_scan!(reduced_line => "{} = \"{}\" # {}").expect("line should have this format");

    // check if name doesnt match, return none
    if package_name != name {
        return Result::Ok(PackageResult::none("cargo", package_name));
    }

    // query installed packages
    let output = Command::new("cargo")
        .arg("install")
        .arg("--list")
        .output()
        .expect("cargo should succed at this point");

    let lines = check_output(output).expect("cargo should return a list of installed packages");

    let filtered_lines: Vec<&str> = lines
        .iter()
        .filter(|line| !line.starts_with(' ') && !line.is_empty())
        .map(|line| line.as_str())
        .collect();

    // find the package in the list
    for line in &filtered_lines {
        let reduced_line = reduce_whitespace(line.to_string());
        let (_, local_version): (String, String) =
            try_scan!(reduced_line => "{} v{}:").expect("line should have this format");

        if package_name == name {
            // check if installed version is differs from the newest version
            let version_info: String = if local_version != version {
                format!("{} -> {}", local_version, version)
            } else {
                local_version
            };

            return Result::Ok(PackageResult::some(
                "cargo",
                &name,
                "installed",
                &version_info,
                &description,
                "",
            ));
        }
    }

    if package_name == name {
        return Result::Ok(PackageResult::some(
            "cargo",
            &name,
            "available",
            &version,
            &description,
            "",
        ));
    }

    Result::Ok(PackageResult::none("cargo", package_name))
}

fn check_go(package_name: &str) -> Result<PackageResult, String> {
    let output = Command::new("go")
        .arg("version")
        .arg("-m")
        .arg("/home/noah/go/bin")
        .output()
        .expect("CUSTOM ERROR: failed to execute go list -m -u <package_name>");

    if !output.stdout.is_empty() {
        let stdout: Vec<u8> = output.stdout;
        let stdout_string = String::from_utf8(stdout).unwrap();

        let filtered_lines = stdout_string
            .split('\n')
            .filter(|line| line.contains("path") && !line.is_empty())
            .collect::<Vec<_>>();

        for line in &filtered_lines {
            let mut chunks = line.split_whitespace();
            chunks.next();
            let fullname = chunks.next().expect("CUSTOM ERROR: failed to get fullname");
            let mut fullnamesplit = fullname.split('/');
            fullnamesplit.next();
            let name = fullnamesplit
                .clone()
                .last()
                .expect("CUSTOM ERROR: failed to get name");
            let repo = fullnamesplit.collect::<Vec<_>>().join("/");

            if package_name == name {
                return Result::Ok(PackageResult::some(
                    "go",
                    fullname,
                    "installed",
                    "",
                    "",
                    repo.as_str(),
                ));
            }
        }
    }
    Result::Ok(PackageResult::none("go", package_name))
}

fn check_snap(package_name: &str) -> Result<PackageResult, String> {
    let output = Command::new("snap")
        .arg("list")
        .output()
        .expect("CUSTOM ERROR: failed to execute snap list");

    if !output.stdout.is_empty() {
        let stdout: Vec<u8> = output.stdout;
        let stdout_string = String::from_utf8(stdout).unwrap();
        let lines = stdout_string.split('\n');

        for line in lines {
            let mut chunks = line.split_whitespace();
            let name = chunks.next().unwrap();
            let version = chunks.next().unwrap();

            if package_name == name {
                return Result::Ok(PackageResult::some(
                    "snap",
                    name,
                    "installed",
                    version,
                    "",
                    "",
                ));
            }
        }
    }

    Result::Ok(PackageResult::none("snap", package_name))
}

fn print_result(results: Vec<PackageResult>) -> core::result::Result<(), std::io::Error> {
    // TODO: fix this mess
    for result in results {
        if result.manager == "yay" {
            if result.status == "installed" {
                log::success(format!(
                    "[installed] -   yay: {} ({})",
                    result.package, result.version
                ))?;
            } else if result.status == "available" {
                log::info(format!(
                    "[available] -   yay: {} ({})",
                    result.package, result.version
                ))?;
            } else if result.status == "not found" {
                log::error(format!("[not found] -   yay: {} ", result.package))?;
            }
        } else if result.manager == "apt" {
            if result.status == "installed" {
                log::success(format!(
                    "[installed] -   apt: {} ({})",
                    result.package, result.version
                ))?;
            } else if result.status == "available" {
                log::info(format!(
                    "[available] -   apt: {} ({})",
                    result.package, result.version
                ))?;
            } else if result.status == "not found" {
                log::error(format!("[not found] -   apt: {} ", result.package))?;
            }
        } else if result.manager == "go" {
            if result.status == "installed" {
                log::success(format!(
                    "[installed] -    go: {} ({})",
                    result.package, result.version
                ))?;
            } else if result.status == "available" {
                log::info(format!(
                    "[available] -    go: {} ({})",
                    result.package, result.version
                ))?;
            } else if result.status == "not found" {
                log::error(format!("[not found] -    go: {} ", result.package))?;
            }
        } else if result.manager == "cargo" {
            if result.status == "installed" {
                log::success(format!(
                    "[installed] - cargo: {} ({})",
                    result.package, result.version
                ))?;
            } else if result.status == "available" {
                // log::info(format!(
                //     "[available] - cargo: {} ({})",
                //     result.package, result.version
                // ))?;
                cliclack::note(
                    format!(
                        "[available] - cargo: {} ({})",
                        result.package, result.version
                    ),
                    result.desc,
                )?;
            } else if result.status == "not found" {
                log::error(format!("[not found] - cargo: {} ", result.package))?;
            }
        }
    }

    Ok(())
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    package: String,
}

fn main() -> std::io::Result<()> {
    let args = Args::parse();

    // set_theme(MyTheme);
    println!();
    intro(style(" boss ").on_cyan().black())?;

    let package_name = args.package;
    let installed_managers = get_installed_managers();

    log::remark(format!("Managers: {}", installed_managers.join(", ")))?;
    log::remark(format!("Checking: {}", package_name))?;

    let spinner = spinner();
    spinner.start("Fetching...");

    let mut results = vec![];
    for manager in &installed_managers {
        match *manager {
            "apt" => match check_apt(&package_name) {
                Ok(result) => results.push(result),
                Err(e) => {
                    spinner.error(&e);
                    log::error(e)?;
                }
            },
            "yay" => match check_yay(&package_name) {
                Ok(result) => results.push(result),
                Err(e) => {
                    spinner.error(&e);
                    log::error(e)?;
                }
            },
            "go" => {
                // only installed packages, go doesnt have a search command. (yet)
                match check_go(&package_name) {
                    Ok(result) => results.push(result),
                    Err(e) => {
                        spinner.error(&e);
                        log::error(e)?;
                    }
                }
            }
            "cargo" => match check_cargo(&package_name) {
                Ok(result) => results.push(result),
                Err(e) => {
                    spinner.error(&e);
                    log::error(e)?;
                }
            },
            &_ => todo!(),
        }
    }

    spinner.stop("Results:");

    print_result(results)?;

    outro("Done!")?;

    Ok(())
}
