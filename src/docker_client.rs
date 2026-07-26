use anyhow::{Context, Result, bail};
use bollard::{API_DEFAULT_VERSION, Docker};
use std::env;
use std::path::PathBuf;
const TIMEOUT_SECS: u64 = 120;

/// Connect to Docker.
///
/// Priority:
/// 1. `docker_host` from config (if provided)
/// 2. `DOCKER_HOST` env (default scenario for local dev and vps for example)
/// 3. First existing known unix socket
pub fn connect(docker_host_override: Option<&str>) -> Result<Docker> {
    if let Some(host) = docker_host_override {
        return connect_host(host)
            .with_context(|| format!("failed to connect using config docker_host ({host})"));
    }

    if let Ok(host) = env::var("DOCKER_HOST") {
        let host = host.trim();

        if !host.is_empty() {
            return connect_host(host)
                .with_context(|| format!("failed to connect using DOCKER_HOST ({host})"));
        }
    }

    let candidates = candidate_sockets();
    let mut tried = Vec::with_capacity(candidates.len());
    for path in &candidates {
        tried.push(path.display().to_string());
        if !path.exists() {
            continue;
        }
        let Some(path_str) = path.to_str() else {
            continue;
        };
        match Docker::connect_with_unix(path_str, TIMEOUT_SECS, API_DEFAULT_VERSION) {
            Ok(docker) => return Ok(docker),
            Err(_) => continue,
        }
    }
    bail!(
        "failed to connect to Docker socket (tried: {})",
        tried.join(", ")
    )
}

fn connect_host(host: &str) -> Result<Docker> {
    let host = host.trim();

    if looks_like_uri(host) {
        return Docker::connect_with_host(host).map_err(Into::into);
    }

    // default scenario
    Docker::connect_with_unix(host, TIMEOUT_SECS, API_DEFAULT_VERSION).map_err(Into::into)
}

fn looks_like_uri(host: &str) -> bool {
    host.starts_with("unix://")
        || host.starts_with("tcp://")
        || host.starts_with("http://")
        || host.starts_with("https://")
        || host.starts_with("npipe://")
        || host.starts_with("ssh://")
}

fn candidate_sockets() -> Vec<PathBuf> {
    let home = env::var("HOME").map(PathBuf::from);
    let mut paths = Vec::new();

    #[cfg(target_os = "macos")]
    {
        if let Ok(home) = &home {
            paths.push(home.join(".docker/run/docker.sock"));
            paths.push(home.join(".colima/default/docker.sock"));
            paths.push(home.join(".orbstack/run/docker.sock"));
        }

        paths.push(PathBuf::from("/var/run/docker.sock"));
    }

    // Linux/WSL code block
    #[cfg(not(target_os = "macos"))]
    {
        paths.push(PathBuf::from("/var/run/docker.sock"));
        if let Ok(home) = &home {
            paths.push(home.join(".docker/run/docker.sock"));
            paths.push(home.join(".colima/default/docker.sock"));
            paths.push(home.join(".orbstack/run/docker.sock"));
        }
        // rootless Docker on Linux
        if let Ok(uid) = env::var("UID") {
            if !uid.is_empty() {
                paths.push(PathBuf::from(format!("/run/user/{uid}/docker.sock")));
            }
        }
    }
    paths
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn looks_like_uri_detects_schemes() {
        assert!(looks_like_uri("unix:///var/run/docker.sock"));
        assert!(looks_like_uri("tcp://127.0.0.1:2375"));
        assert!(!looks_like_uri("/var/run/docker.sock"));
        assert!(!looks_like_uri("/Users/me/.docker/run/docker.sock"));
    }
    #[test]
    fn candidates_are_non_empty() {
        assert!(!candidate_sockets().is_empty());
    }
}
