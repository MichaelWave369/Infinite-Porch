use std::process::Command;
fn git(args: &[&str]) -> String {
    Command::new("git")
        .args(args)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "UNVERIFIED".into())
}
fn main() {
    println!("cargo:rerun-if-changed=src");
    println!("cargo:rerun-if-changed=../../.git/index");
    println!("cargo:rerun-if-changed=../../.git/HEAD");
    println!("cargo:rerun-if-changed=../../.git/refs/heads");
    println!(
        "cargo:rustc-env=PORCH_BUILD_COMMIT={}",
        git(&["rev-parse", "HEAD"])
    );
    println!(
        "cargo:rustc-env=PORCH_BUILD_BRANCH={}",
        git(&["branch", "--show-current"])
    );
    println!(
        "cargo:rustc-env=PORCH_BUILD_TREE_DIRTY={}",
        !git(&["status", "--porcelain", "--untracked-files=no"]).is_empty()
    );
    println!(
        "cargo:rustc-env=PORCH_BUILD_TARGET={}",
        std::env::var("TARGET").unwrap()
    );
}
