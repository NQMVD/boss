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

// apt search = returns a list like yay but with empty lines in between
// apt show = shows only one package with info, DOESNT show if its installed tho
fn check_apt(package_name: &str) -> Result<PackageResult, String> {
    debug!("checking [apt] for [{}]", package_name);

    // 1. check registry if package exists
    let output = match Command::new("apt").arg("show").arg(package_name).output() {
        Ok(output) => output,
        Err(e) => return Err(e.to_string()),
    };
    let lines = match check_output(output) {
        Ok(lines) => lines,
        Err(e) => return Err(e),
    };
    if !lines.iter().any(|line| line.contains("Package:")) {
        debug!("[{}] not found in [apt]", package_name);
        return Result::Ok(PackageResult::none("apt", package_name));
    }
    debug!("[{}] exists", package_name);

    // 2. get info about package: newest version, description
    debug!("extracting version and description");
    let mut version = String::new();
    let mut desc = String::new();

    for line in &lines {
        if line.starts_with("Version:") {
            version = match try_parse!(line => "Version: {}") {
                Ok(version) => version,
                Err(_) => {
                    warn!("could not parse version");
                    return Err("could not parse version".to_string());
                }
            };
        } else if line.starts_with("Description:") {
            desc = match try_parse!(line => "Description: {}") {
                Ok(desc) => desc,
                Err(_) => {
                    warn!("could not parse description");
                    return Err("could not parse description".to_string());
                }
            };
        }
    }
    debug!("version: [{}], description: [{}]", version, desc);

    // 3. check if package is installed
    debug!("running apt list --installed");
    let output = match Command::new("apt").arg("list").arg("--installed").output() {
        Ok(output) => output,
        Err(e) => return Err(e.to_string()),
    };

    debug!("checking output");
    let lines = match check_output(output) {
        Ok(lines) => lines,
        Err(e) => return Err(e),
    };

    debug!("filtering lines");
    let filtered_lines: Vec<String> = lines
        .iter()
        .filter(|line| {
            !line.starts_with("Listing")
                && !line.starts_with(' ')
                && !line.is_empty()
                && line.contains(package_name)
        })
        .map(|line| line.to_string())
        .collect();
    debug!("filtered lines: {}", filtered_lines.len());

    debug!("checking if package is installed");
    for line in &filtered_lines {
        // zlib1g/noble,now 1:1.3.dfsg-3.1ubuntu2 amd64 [installed,automatic]
        debug!("checking line: [{}]", line);
        let scanned: Result<(String, String, String, String, String), _> =
            try_scan!(line => "{}/{} {} {} [{}]");
        let (name, local_version, installed): (String, String, String) = match scanned {
            Ok((name, _, version, _, installed)) => (name, version, installed),
            Err(e) => return Err("parsing error: {e:?}".to_owned()),
        };

        if package_name == name {
            let version_info: String = if local_version != version {
                format!("{} -> {}", local_version, version)
            } else {
                local_version
            };
            debug!("version info: [{}]", version_info);
            debug!("[{}] installed", package_name);
            return Result::Ok(PackageResult::some(
                "apt",
                &name,
                &installed,
                &version_info,
                &desc,
                "",
            ));
        }
    }

    debug!("package not installed but available in [apt] registry");
    Result::Ok(PackageResult::some(
        "apt",
        package_name,
        "available",
        &version,
        &desc,
        "",
    ))
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
    debug!("checking [cargo] for [{}]", package_name);

    // 1. check registry if package exists
    let output = match Command::new("cargo")
        .arg("search")
        .arg(package_name)
        .output()
    {
        Ok(output) => output,
        Err(e) => return Err(e.to_string()),
    };
    let lines = match check_output(output) {
        Ok(lines) => lines,
        Err(e) => {
            debug!("[{}] not found in [cargo] (empty output)", package_name);
            return Result::Ok(PackageResult::none("cargo", package_name)); // cargo search output can be empty
        }
    };
    // check if package exists
    if lines.is_empty() {
        debug!("[{}] not found in [cargo] (lines.is_empty)", package_name);
        return Result::Ok(PackageResult::none("cargo", package_name));
    }
    if lines.iter().all(|line| !line.contains(package_name)) {
        debug!(
            "[{}] not found in [cargo] (no line contains name)",
            package_name
        );
        return Result::Ok(PackageResult::none("cargo", package_name));
    }
    fn check_exact_name(line: &String, package_name: &str) -> bool {
        let mut iter = line.split_whitespace();
        let name: &str = iter.next().unwrap_or_default();
        name == package_name
    }
    if lines
        .iter()
        .all(|line| !check_exact_name(line, package_name))
    {
        debug!("[{}] not found in [cargo] (name not found)", package_name);
        return Result::Ok(PackageResult::none("cargo", package_name));
    }
    debug!("[{}] exists", package_name);

    // 2. get info about package: newest version, description
    debug!("extracting version and description");
    let mut version = String::new();
    let mut desc = String::new();
    let filtered_lines: Vec<String> = lines
        .iter()
        .filter(|line| line.contains(" = "))
        .filter(|line| check_exact_name(line, package_name))
        .map(|line| line.to_string())
        .collect();
    debug!("filtered lines: {}", filtered_lines.len());

    for line in &filtered_lines {
        let reduced_line = reduce_whitespace(line.to_string());
        debug!("reduced line: [{}]", reduced_line);
        let scanned: Result<(String, String, String), _> =
            try_scan!(reduced_line => "{} = \"{}\" # {}");
        (version, desc) = match scanned {
            Ok((_, version, desc)) => (version, desc),
            Err(e) => return Err(format!("[cargo] parsing error: {:?}", e)),
        };
    }
    debug!("version: [{}], description: [{}]", version, desc);

    // 3. check if package is installed
    debug!("running cargo install --list");
    let output = match Command::new("cargo").arg("install").arg("--list").output() {
        Ok(output) => output,
        Err(e) => return Err(e.to_string()),
    };

    debug!("checking output");
    let lines = match check_output(output) {
        Ok(lines) => lines,
        Err(e) => return Err(e),
    };

    debug!("filtering lines");
    let filtered_lines: Vec<String> = lines
        .iter()
        .filter(|line| !line.is_empty() && !line.starts_with(' ') && line.contains(package_name))
        .map(|line| line.to_string())
        .collect();
    debug!("filtered lines: {}", filtered_lines.len());

    debug!("checking if package is installed");
    for line in &filtered_lines {
        debug!("checking line: [{}]", line);
        let scanned: Result<(String, String), _> = try_scan!(line => "{} v{}:");
        let (name, local_version): (String, String) = match scanned {
            Ok((name, version)) => (name, version),
            Err(e) => return Err("parsing error: {e:?}".to_owned()),
        };

        if package_name == name {
            let version_info: String = if local_version != version {
                format!("{} -> {}", local_version, version)
            } else {
                local_version
            };
            debug!("version info: [{}]", version_info);
            return Result::Ok(PackageResult::some(
                "cargo",
                &name,
                "installed",
                &version_info,
                &desc,
                "",
            ));
        }
    }

    debug!("package not installed but available in [cargo] registry");
    Result::Ok(PackageResult::some(
        "cargo",
        package_name,
        "available",
        &version,
        &desc,
        "",
    ))
}

fn check_go(package_name: &str) -> Result<PackageResult, String> {
    // TODO: implement go package check
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

// snap find = query
// snap list = installed
fn check_snap(package_name: &str) -> Result<PackageResult, String> {}

fn print_result(results: Vec<PackageResult>) -> core::result::Result<(), std::io::Error> {
    // TODO: fix this mess
    for result in results {
        // O  [ apt ] - [installed] - (0.5.0-1)
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
}

fn main() -> std::io::Result<()> {
    CombinedLogger::init(vec![WriteLogger::new(
        LevelFilter::Debug,
        Config::default(),
        File::create("boss.log").unwrap(),
    )])
    .unwrap();

    let args = Args::parse();
    debug!("{:?}", args);

    // set_theme(MyTheme);
    println!();
    cliclack::intro(style(" boss ").on_cyan().black())?;

    let package_name = args.package;
    let installed_managers = get_installed_managers();

    cliclack::log::remark(format!("Managers: {}", installed_managers.join(", ")))?;
    cliclack::log::remark(format!("Package: {}", package_name))?; // Packages

    let spinner = cliclack::spinner();
    spinner.start("Fetching...");

    let mut results = vec![];
    for manager in &installed_managers {
        match *manager {
            "apt" => match check_apt(&package_name) {
                Ok(result) => results.push(result),
                Err(e) => {
                    spinner.error(&e);
                    cliclack::log::error(e)?;
                }
            },
            "yay" => match check_yay(&package_name) {
                Ok(result) => results.push(result),
                Err(e) => {
                    spinner.error(&e);
                    cliclack::log::error(e)?;
                }
            },
            "go" => {
                // only installed packages, go doesnt have a search command. (yet)
                // match check_go(&package_name) {
                //     Ok(result) => results.push(result),
                //     Err(e) => {
                //         spinner.error(&e);
                //         cliclack::log::error(e)?;
                //     }
                // }
            }
            "cargo" => match check_cargo(&package_name) {
                Ok(result) => results.push(result),
                Err(e) => {
                    spinner.error(&e);
                    cliclack::log::error(e)?;
                }
            },
            &_ => todo!(),
        }
    }

    spinner.stop("Results:");

    print_result(results)?;

    cliclack::outro("Done!")?;

    Ok(())
}
