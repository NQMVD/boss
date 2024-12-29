use crate::{check_output, reduce_whitespace, PackageResult};
use std::process::Command;
use strp::*;

/// Checks if a package is available or installed using the `cargo` package manager.
pub fn check_nix(package_name: &str) -> Result<PackageResult, String> {
    let full_name: String = "nixpkgs#".to_owned().clone() + package_name;
    // -----------------------------------
    // 1. check registry if package exists
    // -----------------------------------
    let output = match Command::new("nix")
        .arg("search")
        .arg(full_name)
        .arg("^")
        .output()
    {
        Ok(output) => output,
        Err(e) => return Err(format!("[nix] {}", e)),
    };

    let lines = match check_output(output) {
        Ok(lines) => lines,
        Err(_) => {
            debug!("nix search output is empty");
            return Result::Ok(PackageResult::none("nix", package_name)); // nix search output can be empty
        }
    };

    if lines.is_empty() {
        debug!("nix search output is empty");
        return Result::Ok(PackageResult::none("nix", package_name));
    }
    if lines.iter().all(|line| !line.contains(package_name)) {
        debug!("nix search output didnt contain package name");
        return Result::Ok(PackageResult::none("nix", package_name));
    }

    // ------------------------------------------------------
    // 2. get info about package: newest version, description
    // ------------------------------------------------------
    let mut version = String::new();
    let desc = String::new();
    let filtered_lines: Vec<String> = lines
        .iter()
        .filter(|line| line.contains("*"))
        .map(|line| line.to_string())
        .collect();

    for line in &filtered_lines {
        let reduced_line = reduce_whitespace(line.to_string());
        let scanned: Result<(String, String, String, String), _> =
            // try_scan!(reduced_line => "{} = \"{}\" # {}");
            try_scan!(reduced_line => "* {}.{}.{} ({})");
        version = match scanned {
            Ok((_, _, _, version)) => version,
            Err(e) => return Err(format!("[nix] parsing error: {:?}", e)),
        };
    }

    // --------------------------------
    // 3. check if package is installed
    // --------------------------------
    let output = match Command::new("nix").arg("profile").arg("list").output() {
        Ok(output) => output,
        Err(e) => return Err(e.to_string()),
    };

    let lines = match check_output(output) {
        Ok(lines) => lines,
        Err(e) => return Err(e),
    };

    let filtered_lines: Vec<String> = lines
        .iter()
        .filter(|line| !line.is_empty() && line.contains(package_name) && line.contains("Name:"))
        .map(|line| line.to_string())
        .collect();

    for line in &filtered_lines {
        let reduced_line = reduce_whitespace(line.to_string());
        let scanned: Result<String, _> = try_parse!(reduced_line => "Name: {}");
        let name: String = match scanned {
            Ok(name) => name,
            Err(e) => return Err(format!("[nix] parsing error: {e:?}")),
        };

        if package_name == name {
            return Result::Ok(PackageResult::some(
                "nix",
                &name,
                "installed",
                &version,
                &desc,
                "",
            ));
        }
    }

    Result::Ok(PackageResult::some(
        "nix",
        package_name,
        "available",
        &version,
        &desc,
        "",
    ))
}
