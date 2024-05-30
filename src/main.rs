use cliclack::{intro, log, outro, set_theme, spinner, Theme};
use console::style;
use std::process::{Command, Stdio};

struct MyTheme;

impl Theme for MyTheme {
    // fn info_symbol(&self) -> String {
    //     // info symbol
    //     "".into()
    // }

    // fn active_symbol(&self) -> String {
    //     // success symbol
    //     "".into()
    // }

    // fn error_symbol(&self) -> String {
    //     // error symbol
    //     "".into()
    // }

    fn spinner_chars(&self) -> String {
        "".to_string()
    }
}

struct PackageResult {
    manager: String, // apt, yay, go, cargo
    package: String, // name only
    version: String, // version
    desc: String,    // description
    repo: String,    // repo, for yay it's the repo (for go it's the module path?)
    status: String,  // installed, available, not found
}

impl PackageResult {
    fn new(
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

fn check_yay(package_name: &str) -> Result<PackageResult, String> {
    let output = Command::new("yay")
        .arg("-Ss")
        .arg(package_name)
        .output()
        .expect("CUSTOM ERROR: failed to execute yay -Ss <package_name>");

    if !output.stdout.is_empty() {
        let stdout_str =
            std::str::from_utf8(&output.stdout).expect("CUSTOM ERROR: failed to parse stdout");
        let mut lines = stdout_str.split('\n');
        let line = lines.nth_back(2).expect("CUSTOM ERROR: failed to get line");
        let mut chunks = line.split_whitespace();
        let fullname = chunks.next().expect("CUSTOM ERROR: failed to get fullname");
        let (repo, name) = fullname
            .split_once('/')
            .expect("CUSTOM ERROR: failed to split fullname");
        let version = chunks.next().expect("CUSTOM ERROR: failed to get version");

        if name == package_name {
            let status = if line.contains("Installed") {
                "installed"
            } else {
                "available"
            };

            return Result::Ok(PackageResult::new(
                "yay", fullname, status, version, "", repo,
            ));
        } else {
            return Result::Ok(PackageResult::none("yay", package_name));
        }
    }

    Result::Ok(PackageResult::none("yay", package_name))
}

fn check_cargo(package_name: &str) -> Result<PackageResult, String> {
    let mut installed = false;

    // TODO: change this to read the ~/.cargo/.crates.toml file
    let output = Command::new("cargo")
        .arg("install")
        .arg("--list")
        .output()
        .expect("CUSTOM ERROR: failed to execute cargo install --list");

    if !output.stdout.is_empty() {
        let stdout: Vec<u8> = output.stdout;
        let stdout_string = String::from_utf8(stdout).unwrap();
        let lines = stdout_string.split('\n');

        let filtered_lines: Vec<&str> = lines
            .filter(|line| !line.starts_with(' ') && !line.is_empty())
            .collect();
        for line in &filtered_lines {
            let mut chunks = line.split_whitespace();
            let name = chunks.next().unwrap();

            let mut chars = chunks.next().unwrap().chars();
            chars.next_back();
            let version = chars.as_str();

            if package_name == name {
                installed = true;
                return Result::Ok(PackageResult::new(
                    "cargo",
                    name,
                    "installed",
                    version,
                    "",
                    "",
                ));
            }
        }
    }

    // check if availlable if not installed
    if !installed {
        let output = Command::new("cargo")
            .arg("search")
            .arg(package_name)
            .output()
            .expect("CUSTOM ERROR: failed to execute cargo search <package_name>");

        if !output.stdout.is_empty() {
            let stdout: Vec<u8> = output.stdout;
            let stdout_string = String::from_utf8(stdout).unwrap();
            let mut lines = stdout_string.split('\n');
            let line = lines.next().unwrap();
            let mut chunks = line.split_whitespace();
            let name = chunks.next().unwrap();
            chunks.next();
            let mut chars = chunks.next().unwrap().chars();
            chars.next();
            chars.next_back();
            let version = chars.as_str();
            let description = chunks.collect::<Vec<_>>().join(" ");

            if package_name == name {
                return Result::Ok(PackageResult::new(
                    "cargo",
                    name,
                    "available",
                    version,
                    description.as_str(),
                    "",
                ));
            } else {
                return Result::Ok(PackageResult::none("cargo", package_name));
            }
        }
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
                return Result::Ok(PackageResult::new(
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

// fn parse_package(package_name: &str) -> Result<PackageResult, String> {
//     let mut result = check_apt(package_name);
//     if result.is_none() {
//         result = check_pacman(package_name);
//     }
//     if result.is_none() {
//         result = check_yay(package_name);
//     }
//     if result.is_none() {
//         result = check_cargo(package_name);
//     }
//     if result.is_none() {
//         result = check_go(package_name);
//     }

//     result
// }

fn print_result(results: Vec<PackageResult>) -> core::result::Result<(), std::io::Error> {
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

fn main() -> std::io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        println!("Usage: {} <package_name>", args[0]);
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Invalid input, needs a package name as argument",
        ));
    }

    set_theme(MyTheme);
    println!();
    intro(style(" peo ").on_cyan().black())?;

    let package_name = &args[1];
    let installed_managers = get_installed_managers();

    log::remark(format!("Managers: {}", installed_managers.join(", ")))?;

    let spinner = spinner();
    spinner.start("Fetching...");

    let mut results = vec![];
    for manager in &installed_managers {
        match *manager {
            "apt" => {}
            "yay" => match check_yay(package_name) {
                Ok(result) => results.push(result),
                Err(e) => {
                    spinner.error(&e);
                    log::error(e)?;
                }
            },
            "go" => {
                // only installed packages, go doesnt have a search command. (yet)
                match check_go(package_name) {
                    Ok(result) => results.push(result),
                    Err(e) => {
                        spinner.error(&e);
                        log::error(e)?;
                    }
                }
            }
            "cargo" => match check_cargo(package_name) {
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
