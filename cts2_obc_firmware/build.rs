use std::process::Command;
use std::time::SystemTime;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    // rerun build file if HEAD changes (ie. there are new commits)
    println!("cargo:rerun-if-changed=../.git/HEAD");
    println!("cargo:rerun-if-changed=../.git/refs/heads");

    let output = Command::new("git").args(["rev-parse", "HEAD"]).output();

    let commit_hash = match output {
        Ok(out) if out.status.success() => String::from_utf8(out.stdout)
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|_| "unknown".to_string()),
        _ => "unknown".to_string(),
    };

    let build_timestamp: i64 = match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
        Ok(duration) => duration.as_secs() as i64,
        Err(e) => -(e.duration().as_secs() as i64),
    };

    // embed the git commit hash and build timestamp as environment variables
    println!("cargo:rustc-env=COMMIT_HASH={}", commit_hash);
    println!("cargo:rustc-env=BUILD_TIMESTAMP={}", build_timestamp);
}
