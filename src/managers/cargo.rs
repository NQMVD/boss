use crate::{check_output, reduce_whitespace, PackageResult};
use std::process::Command;
use strp::*;

/// Checks if a package is available or installed using the `cargo` package manager.
pub fn check_cargo(package_name: &str) -> Result<PackageResult, String> {
    // -----------------------------------
    // 1. check registry if package exists
    // -----------------------------------
    let output = match Command::new("cargo")
        .arg("search")
        .arg(package_name)
        .output()
    {
        Ok(output) => output,
        Err(e) => return Err(format!("[cargo] {}", e)),
    };

    let lines = match check_output(output) {
        Ok(lines) => lines,
        Err(e) => {
            return Result::Ok(PackageResult::none("cargo", package_name)); // cargo search output can be empty
        }
    };

    if lines.is_empty() {
        return Result::Ok(PackageResult::none("cargo", package_name));
    }
    if lines.iter().all(|line| !line.contains(package_name)) {
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
        return Result::Ok(PackageResult::none("cargo", package_name));
    }

    // ------------------------------------------------------
    // 2. get info about package: newest version, description
    // ------------------------------------------------------
    let mut version = String::new();
    let mut desc = String::new();
    let filtered_lines: Vec<String> = lines
        .iter()
        .filter(|line| line.contains(" = "))
        .filter(|line| check_exact_name(line, package_name))
        .map(|line| line.to_string())
        .collect();

    for line in &filtered_lines {
        let reduced_line = reduce_whitespace(line.to_string());
        let scanned: Result<(String, String, String), _> =
            try_scan!(reduced_line => "{} = \"{}\" # {}");
        (version, desc) = match scanned {
            Ok((_, version, desc)) => (version, desc),
            Err(e) => return Err(format!("[cargo] parsing error: {:?}", e)),
        };
    }

    // --------------------------------
    // 3. check if package is installed
    // --------------------------------
    let output = match Command::new("cargo").arg("install").arg("--list").output() {
        Ok(output) => output,
        Err(e) => return Err(e.to_string()),
    };

    let lines = match check_output(output) {
        Ok(lines) => lines,
        Err(e) => return Err(e),
    };

    let filtered_lines: Vec<String> = lines
        .iter()
        .filter(|line| !line.is_empty() && !line.starts_with(' ') && line.contains(package_name))
        .map(|line| line.to_string())
        .collect();

    for line in &filtered_lines {
        let scanned: Result<(String, String), _> = try_scan!(line => "{} v{}:");
        let (name, local_version): (String, String) = match scanned {
            Ok((name, version)) => (name, version),
            Err(e) => return Err(format!("[cargo] parsing error: {e:?}")),
        };

        if package_name == name {
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
                &desc,
                "",
            ));
        }
    }

    Result::Ok(PackageResult::some(
        "cargo",
        package_name,
        "available",
        &version,
        &desc,
        "",
    ))
}
