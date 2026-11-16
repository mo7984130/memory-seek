use std::process::Command;

fn main() {
    // 注入 git commit
    println!("cargo:rerun-if-env-changed=GITHUB_SHA");
    let commit = std::env::var("GITHUB_SHA")
        .ok()
        .filter(|sha| !sha.is_empty())
        .map(|sha| sha.chars().take(7).collect::<String>())
        .or_else(git_short_commit)
        .unwrap_or_else(|| "unknown".to_string());
    println!("cargo:rustc-env=MEMORY_SEEK_GIT_COMMIT={commit}");
}

fn git_short_commit() -> Option<String> {
    Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .filter(|out| out.status.success())
        .map(|out| String::from_utf8_lossy(&out.stdout).trim().to_string())
        .filter(|commit| !commit.is_empty())
}
