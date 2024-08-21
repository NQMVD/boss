use crate::{check_output, reduce_whitespace, PackageResult};
use std::process::Command;
use strp::*;

/// Checks if a package is available or installed using the `snap` package manager.
fn check_snap(package_name: &str) -> Result<PackageResult, String> {
    // found: {name} {version} {_} {_} {summary}
    // No matching snaps for {name}
    // installed: {name} {version} {_}

    // -----------------------------------
    // 1. check registry if package exists
    // -----------------------------------
    let output = match Command::new("snap").arg("find").arg(package_name).output() {
        Ok(output) => output,
        Err(e) => return Err(format!("[snap] {}", e)),
    };
    let lines = match check_output(output) {
        Ok(lines) => lines,
        Err(e) => {
            warn!("snap show output is empty");
            return Result::Ok(PackageResult::none("snap", package_name));
        }
    };
    if lines.is_empty() {
        return Result::Ok(PackageResult::none("snap", package_name));
    }
    if lines
        .iter()
        .any(|line| line.contains("No matching snaps for"))
    {
        return Result::Ok(PackageResult::none("snap", package_name));
    }
    if lines.iter().all(|line| !line.contains(package_name)) {
        return Result::Ok(PackageResult::none("snap", package_name));
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
        return Result::Ok(PackageResult::none("snap", package_name));
    }

    // ------------------------------------------------------
    // 2. get info about package: newest version, description
    // ------------------------------------------------------
    // remove the first line
    match lines.iter().next() {
        Some(line) => (),
        None => return Err("snap show output is empty".to_owned()),
    };

    // filter the lines by exact name
    let filtered_lines: Vec<String> = lines
        .iter()
        .filter(|line| check_exact_name(line, package_name))
        .map(|line| line.to_string())
        .collect();

    // loop over, scan and extract version and description
    let mut version = String::new();
    let mut desc = String::new();
    for line in &filtered_lines {
        let reduced_line = reduce_whitespace(line.to_string());
        let scanned: Result<(String, String, String, String, String), _> =
            try_scan!(reduced_line => "{} {} {} {} {}");
        (version, desc) = match scanned {
            Ok((_, version, _, _, desc)) => (version, desc),
            Err(e) => return Err(format!("[cargo] parsing error: {:?}", e)),
        };
    }

    // --------------------------------
    // 3. check if package is installed
    // --------------------------------
    // run command
    let output = match Command::new("snap").arg("list").output() {
        Ok(output) => output,
        Err(e) => return Err(e.to_string()),
    };

    // check for empty output
    let lines = match check_output(output) {
        Ok(lines) => lines,
        Err(e) => return Err(e),
    };

    // remove the first line (header)
    match lines.iter().next() {
        Some(line) => (),
        None => return Err("snap show output is empty".to_owned()),
    };

    // filter the lines
    let filtered_lines: Vec<String> = lines
        .iter()
        .filter(|line| !line.starts_with(' ') && !line.is_empty() && line.contains(package_name))
        .map(|line| line.to_string())
        .collect();

    // loop over, reduce, scan and extract version
    for line in &filtered_lines {
        let reduced_line = reduce_whitespace(line.to_string());
        let scanned: Result<(String, String, String), _> = try_scan!(reduced_line => "{} {} {}");
        let (name, local_version): (String, String) = match scanned {
            Ok((name, version, _)) => (name, version),
            Err(e) => return Err(format!("parsing error: {e:?}")),
        };

        if package_name == name {
            let version_info: String = if local_version != version {
                format!("{} -> {}", local_version, version)
            } else {
                local_version
            };
            return Result::Ok(PackageResult::some(
                "snap",
                &name,
                "installed",
                &version_info,
                &desc,
                "",
            ));
        }
    }

    Result::Ok(PackageResult::some(
        "snap",
        package_name,
        "available",
        &version,
        &desc,
        "",
    ))
}
