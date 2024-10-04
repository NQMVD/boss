use crate::{check_output, PackageResult};
use std::process::Command;

/// Checks if a package is available or installed using the `yay` package manager.
pub fn check_yay(package_name: &str) -> Result<PackageResult, String> {
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
