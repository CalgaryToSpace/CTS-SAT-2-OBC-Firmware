use std::process::Command;

fn main() {
    // rerun build file if HEAD changes (ie. there are new commits)
    println!("cargo:rerun-if-changed=.git/HEAD");
    println!("cargo:rerun-if-changed=.git/refs/heads");

    let output = Command::new("git").args(["rev-parse", "HEAD"]).output();

    let commit_hash = match output {
        Ok(out) if out.status.success() => String::from_utf8(out.stdout)
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|_| "unknown".to_string()),
        _ => "unknown".to_string(),
    };

    // embed the git commit hash as an environment variable to be read in by telecommand
    println!("cargo:rustc-env=COMMIT_HASH={}", commit_hash);
}
