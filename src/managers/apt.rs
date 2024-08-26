use crate::{check_output, PackageResult};
use std::process::Command;
use strp::*;

/// Checks if a package is available or installed using the `apt` package manager.
pub fn check_apt(package_name: &str) -> Result<PackageResult, String> {
    // -----------------------------------
    // 1. check registry if package exists
    // -----------------------------------
    debug!("checking registry for package: {}", package_name);
    let output = match Command::new("apt").arg("show").arg(package_name).output() {
        Ok(output) => output,
        Err(e) => {
            error!("could not check registry: {}", e);
            return Err(format!("[apt] {}", e));
        }
    };
    let lines = match check_output(output) {
        Ok(lines) => lines,
        Err(e) => {
            warn!("apt show output is empty");
            return Result::Ok(PackageResult::none("apt", package_name));
        }
    };
    if !lines.iter().any(|line| line.contains("Package:")) {
        debug!("package not found in registry");
        return Result::Ok(PackageResult::none("apt", package_name));
    }
    // -----------------------------------
    // 1.1. check if package is virtual
    // -----------------------------------
    if lines.iter().any(|line| line.contains("not a real package")) {
        debug!("package is virtual and not a real package");
        return Result::Ok(PackageResult::none("apt", package_name));
    }

    // ------------------------------------------------------
    // 2. get info about package: newest version, description
    // ------------------------------------------------------
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

    // --------------------------------
    // 3. check if package is installed
    // --------------------------------
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
        .filter(|line| {
            !line.starts_with("Listing")
                && !line.starts_with(' ')
                && !line.is_empty()
                && line.contains(package_name)
        })
        .map(|line| line.to_string())
        .collect();

    for line in &filtered_lines {
        // zlib1g/noble,now 1:1.3.dfsg-3.1ubuntu2 amd64 [installed,automatic]
        let scanned: Result<(String, String, String, String, String), _> =
            try_scan!(line => "{}/{} {} {} [{}]");
        let (name, local_version, installed): (String, String, String) = match scanned {
            Ok((name, _, version, _, installed)) => (name, version, installed),
            Err(e) => return Err(format!("parsing error: {e:?}")),
        };

        if package_name == name {
            let version_info: String = if local_version != version {
                format!("{} -> {}", local_version, version)
            } else {
                local_version
            };
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

    Result::Ok(PackageResult::some(
        "apt",
        package_name,
        "available",
        &version,
        &desc,
        "",
    ))
}
