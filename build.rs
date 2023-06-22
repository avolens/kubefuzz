use std::process::Command;

fn main() {
    let output = Command::new("git")
        .args(&["describe", "--tags"])
        .output()
        .expect("Failed to execute `git describe --tags`");

    let git_describe = if output.status.success() {
        std::str::from_utf8(&output.stdout)
            .unwrap()
            .trim()
            .to_string()
    } else {
        "dev".to_string() // default version when no tags are available
    };

    println!("cargo:rustc-env=VERSION={}", git_describe);
}
