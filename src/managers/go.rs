use crate::{check_output, reduce_whitespace, PackageResult};
use std::process::Command;
use strp::*;

/// Checks if a package is available or installed using the `go` package manager.
/// disabled until go package check is implemented
pub fn check_go(package_name: &str) -> Result<PackageResult, String> {
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
