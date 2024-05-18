use std::process::{Command, Stdio};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        println!("Usage: {} <package_name>", args[0]);
        return;
    }

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

    println!(
        "Installed package managers: {}",
        installed_managers.join(", ")
    );

    let mut results = vec![];
    for manager in &installed_managers {
        match *manager {
            "apt" => {}
            "yay" => match Command::new("yay").arg("-Ss").arg(package_name).output() {
                Ok(output) => {
                    if !output.stdout.is_empty() {
                        let stdout: Vec<u8> = output.stdout;
                        let stdout_string = String::from_utf8(stdout).unwrap();
                        let mut lines = stdout_string.split('\n');
                        let count = lines.clone().count();
                        let line = lines.nth(count - 3).unwrap();
                        let mut chunks = line.split_whitespace();
                        let fullname = chunks.next().unwrap();
                        let mut fullnamesplit = fullname.split('/');
                        let _repo = fullnamesplit.next().unwrap();
                        let name = fullnamesplit.next().unwrap();
                        let version = chunks.next().unwrap();

                        if package_name == name {
                            if line.contains("Installed") {
                                results.push(format!(
                                    "I -   yay: {} {} [installed]",
                                    fullname, version
                                ));
                            } else {
                                results.push(format!("A -   yay: {} {}", fullname, version));
                            }
                        } else {
                            results.push(format!("X -   yay: {} != {}", package_name, fullname));
                        }
                    }
                }
                Err(_) => results.push(format!("Error running command for {}", manager)),
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
                            let stdout: Vec<u8> = output.stdout;
                            let stdout_string = String::from_utf8(stdout).unwrap();
                            let lines = stdout_string.split('\n');

                            let filtered_lines: Vec<&str> = lines
                                .filter(|line| line.contains("path") && !line.is_empty())
                                .collect();
                            for line in &filtered_lines {
                                let mut chunks = line.split_whitespace();
                                chunks.next();
                                let fullname = chunks.next().unwrap();
                                let mut fullnamesplit = fullname.split('/');
                                fullnamesplit.next();
                                let name = fullnamesplit.clone().last().unwrap();
                                let repo = fullnamesplit.collect::<Vec<_>>().join("/");

                                if package_name == name {
                                    results.push(format!(
                                        "I -    go: {} ({}) [installed]",
                                        name, repo
                                    ));
                                }
                            }
                        } else {
                            println!("stdout is empty!");
                            println!("{:?}", output);
                        }
                    }
                    Err(_) => {
                        results.push(format!("Error running 'install --list' for {}", manager))
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
                                    results.push(format!(
                                        "I - cargo: {} {} [installed]",
                                        name, version
                                    ));
                                    installed = true;
                                }
                            }
                        }
                    }
                    Err(_) => {
                        results.push(format!("Error running 'install --list' for {}", manager))
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
                                    results.push(format!(
                                        "A - cargo: {} {} {}",
                                        name, version, description
                                    ));
                                } else {
                                    results.push(format!(
                                        "X - cargo: {} != {} {}",
                                        package_name, name, description
                                    ));
                                }
                            }
                        }
                        Err(_) => results.push(format!("Error running 'search' for {}", manager)),
                    }
                }
            }
            &_ => todo!(),
        }
    }

    println!("\nResults:");
    for result in results.iter() {
        println!("{}", result);
    }
}
