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

// struct Result<'a> {
//     manager: &'a str,
//     package: &'a str,
//     version: &'a str,
//     info: &'a str,
//     status: &'a str,
// }

// impl<'a> Result<'a> {
//     fn new(
//         manager: &'a str,
//         package: &'a str,
//         version: &'a str,
//         info: &'a str,
//         status: &'a str,
//     ) -> Self {
//         Self {
//             manager,
//             package,
//             version,
//             info,
//             status,
//         }
//     }
// }

struct Result {
    manager: &'static str,
    package: String,
    version: String,
    info: String,
    repo: String,
    status: String,
}

impl Result {
    fn new(
        manager: &'static str,
        package: &str,
        version: &str,
        info: &str,
        repo: &str,
        status: &str,
    ) -> Self {
        Result {
            manager,
            package: package.to_string(),
            version: version.to_string(),
            info: info.to_string(),
            repo: repo.to_string(),
            status: status.to_string(),
        }
    }
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
            Err(_) => todo!(),
        }
    }

    log::remark(format!("Managers: {}", installed_managers.join(", ")))?;

    let spinner = spinner();
    spinner.start("Fetching...");

    let mut results = vec![];
    for manager in &installed_managers {
        match *manager {
            "apt" => {}
            "yay" => match Command::new("yay").arg("-Ss").arg(package_name).output() {
                Ok(output) => {
                    if !output.stdout.is_empty() {
                        if let Ok(stdout_str) = std::str::from_utf8(&output.stdout) {
                            let mut lines = stdout_str.split('\n');
                            if let Some(line) = lines.nth_back(2) {
                                let mut chunks = line.split_whitespace();
                                if let Some(fullname) = chunks.next() {
                                    if let Some((repo, name)) = fullname.split_once('/') {
                                        if let Some(version) = chunks.next() {
                                            if name == package_name {
                                                let status = if line.contains("Installed") {
                                                    "installed"
                                                } else {
                                                    "available"
                                                };

                                                results.push(Result::new(
                                                    "yay", fullname, version, "", repo, status,
                                                ));
                                            } else {
                                                results.push(Result::new(
                                                    "yay",
                                                    package_name,
                                                    "",
                                                    "",
                                                    "",
                                                    "not found",
                                                ));
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    } else {
                        results.push(Result::new("yay", package_name, "", "", "", "not found"));
                    }
                }
                Err(_) => {
                    log::error("yay failed!")?;
                    spinner.error("yay failed!");
                }
            },
            "go" => {
                // only installed packages, go doesnt have a search command
                // TODO: create one?

                match Command::new("go")
                    .arg("version")
                    .arg("-m")
                    .arg("/home/noah/go/bin")
                    .output()
                {
                    Ok(output) => {
                        if !output.stdout.is_empty() {
                            if let Ok(stdout_str) = std::str::from_utf8(&output.stdout) {
                                stdout_str
                                    .split('\n')
                                    .filter(|line| line.contains("path") && !line.is_empty())
                                    .for_each(|line| {
                                        let mut chunks = line.split_whitespace();
                                        chunks.next();
                                        if let Some(fullname) = chunks.next() {
                                            let mut fullnamesplit = fullname.split('/');
                                            fullnamesplit.next();
                                            if let Some(name) = fullnamesplit.clone().last() {
                                                let repo =
                                                    fullnamesplit.collect::<Vec<_>>().join("/");

                                                if package_name == name {
                                                    results.push(Result::new(
                                                        "go",
                                                        fullname,
                                                        "",
                                                        "",
                                                        repo.as_str(),
                                                        "installed",
                                                    ));
                                                }
                                            }
                                        }
                                    });

                                // let mut lines = stdout_str.split('\n');
                                // let filtered_lines: Vec<&str> = lines
                                //     .filter(|line| line.contains("path") && !line.is_empty())
                                //     .collect();
                                // for line in &filtered_lines {
                                //     let mut chunks = line.split_whitespace();
                                //     chunks.next();
                                //     let fullname = chunks.next().unwrap();
                                //     let mut fullnamesplit = fullname.split('/');
                                //     fullnamesplit.next();
                                //     let name = fullnamesplit.clone().last().unwrap();
                                //     let repo = fullnamesplit.collect::<Vec<_>>().join("/");

                                //     if package_name == name {
                                //         results.push(Result::new(
                                //             "go",
                                //             fullname,
                                //             "",
                                //             "",
                                //             repo.as_str(),
                                //             "installed",
                                //         ));
                                //     }
                            }
                        } else {
                            log::error(format!("go failed! stdout is empty..."))?;
                            spinner.error("go failed! stdout is empty...");
                            println!("{:?}", output);
                        }
                    }
                    Err(_) => {
                        log::error(format!("go failed!"))?;
                        spinner.error("go failed!");
                    }
                }
            }
            "cargo" => {
                let mut installed = false;
                // first check if installed
                match Command::new("cargo").arg("install").arg("--list").output() {
                    Ok(output) => {
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
                                let version = chunks.next().unwrap();

                                if package_name == name {
                                    results.push(Result::new(
                                        "cargo",
                                        name,
                                        version,
                                        "",
                                        "",
                                        "installed",
                                    ));
                                    installed = true;
                                }
                            }
                        }
                    }
                    Err(_) => {
                        log::error(format!("cargo list failed!"))?;
                        spinner.error("cargo list failed!");
                    }
                }

                // check if availlable if not installed
                if !installed {
                    match Command::new("cargo")
                        .arg("search")
                        .arg(package_name)
                        .output()
                    {
                        Ok(output) => {
                            if !output.stdout.is_empty() {
                                let stdout: Vec<u8> = output.stdout;
                                let stdout_string = String::from_utf8(stdout).unwrap();
                                let mut lines = stdout_string.split('\n');
                                let line = lines.next().unwrap();
                                let mut chunks = line.split_whitespace();
                                let name = chunks.next().unwrap();
                                chunks.next();
                                let version = chunks.next().unwrap();
                                let description = chunks.collect::<Vec<_>>().join(" ");

                                if package_name == name {
                                    results.push(Result::new(
                                        "cargo",
                                        name,
                                        version,
                                        description.as_str(),
                                        "",
                                        "available",
                                    ));
                                } else {
                                    results.push(Result::new(
                                        "cargo",
                                        package_name,
                                        "",
                                        "",
                                        "",
                                        "not found",
                                    ));
                                }
                            }
                        }
                        Err(_) => {
                            log::error(format!("cargo search failed!"))?;
                            spinner.error("cargo search failed!");
                        }
                    }
                }
            }
            &_ => todo!(),
        }
    }

    spinner.stop("Results:");

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
                    result.info,
                )?;
            } else if result.status == "not found" {
                log::error(format!("[not found] - cargo: {} ", result.package))?;
            }
        }
    }

    outro("Done!")?;

    Ok(())
}
