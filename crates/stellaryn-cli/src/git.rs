//! Read committed WASM artifacts from Git without checking out revisions.
//!
//! All arguments are passed directly to `git` (never through a shell), and
//! only blobs referenced by a commit are accepted. The active worktree, index,
//! and HEAD are never modified.

use std::io;
use std::path::Path;
use std::process::{Command, Output};

use thiserror::Error;

/// Limit a stored artifact before Git materializes it in process memory.
pub const MAX_GIT_BLOB_BYTES: u64 = 32 * 1024 * 1024;

#[derive(Debug, Error)]
pub enum GitArtifactError {
    #[error("invalid Git revision '{revision}': expected a nonempty revision without leading '-', ':', or control/whitespace characters")]
    InvalidRevision { revision: String },

    #[error("invalid repository-relative artifact path '{path}': use a nonempty forward-slash path without '.' or '..' components, ':' or backslashes")]
    InvalidPath { path: String },

    #[error("could not launch Git: {0}")]
    GitIo(#[from] io::Error),

    #[error("Git {operation} failed: {reason}")]
    GitFailure {
        operation: &'static str,
        reason: String,
    },

    #[error("Git returned an invalid blob size for '{revision}:{path}'")]
    InvalidBlobSize { revision: String, path: String },

    #[error("Git blob '{revision}:{path}' exceeds the 32 MiB limit (actual size: {size})")]
    TooLarge {
        revision: String,
        path: String,
        size: u64,
    },
}

fn validate_revision(revision: &str) -> Result<(), GitArtifactError> {
    if revision.is_empty()
        || revision.starts_with('-')
        || revision.contains(':')
        || revision.chars().any(char::is_whitespace)
        || revision.chars().any(char::is_control)
    {
        return Err(GitArtifactError::InvalidRevision {
            revision: revision.to_owned(),
        });
    }
    Ok(())
}

fn validate_artifact_path(path: &str) -> Result<(), GitArtifactError> {
    if path.is_empty()
        || path.starts_with('/')
        || path.contains(':')
        || path.contains('\\')
        || path.chars().any(char::is_control)
        || path
            .split('/')
            .any(|component| component.is_empty() || component == "." || component == "..")
    {
        return Err(GitArtifactError::InvalidPath {
            path: path.to_owned(),
        });
    }
    Ok(())
}

fn git(repo: &Path, args: &[&str], operation: &'static str) -> Result<Output, GitArtifactError> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(args)
        .output()?;
    if output.status.success() {
        Ok(output)
    } else {
        // Treat Git stderr as untrusted text and cap how much reaches diagnostics.
        let reason: String = String::from_utf8_lossy(&output.stderr)
            .chars()
            .filter(|ch| !ch.is_control() || *ch == '\n')
            .take(400)
            .collect();
        Err(GitArtifactError::GitFailure {
            operation,
            reason: if reason.trim().is_empty() {
                format!("exit status {}", output.status)
            } else {
                reason.trim().to_owned()
            },
        })
    }
}

/// Extract a raw blob from a particular Git revision.
///
/// `revision` must peel to a commit, and `artifact_path` is relative to the
/// repository root (not to the current working directory). This function
/// never runs `git checkout`, `git reset`, `git clean`, or shell commands.
pub fn read_git_wasm(
    repo: &Path,
    revision: &str,
    artifact_path: &str,
) -> Result<Vec<u8>, GitArtifactError> {
    validate_revision(revision)?;
    validate_artifact_path(artifact_path)?;

    // Force a commit-ish. This rejects a bare blob/tree revision but supports
    // annotated tags, branches, HEAD, and normal Git ancestry expressions.
    let commit = format!("{revision}^{{commit}}");
    let kind = git(repo, &["cat-file", "-t", &commit], "commit lookup")?;
    if kind.stdout != b"commit\n" {
        return Err(GitArtifactError::GitFailure {
            operation: "commit lookup",
            reason: "revision does not resolve to a commit".to_owned(),
        });
    }

    let locator = format!("{revision}:{artifact_path}");
    let size_output = git(repo, &["cat-file", "-s", &locator], "artifact lookup")?;
    let size = String::from_utf8_lossy(&size_output.stdout)
        .trim()
        .parse::<u64>()
        .map_err(|_| GitArtifactError::InvalidBlobSize {
            revision: revision.to_owned(),
            path: artifact_path.to_owned(),
        })?;
    if size > MAX_GIT_BLOB_BYTES {
        return Err(GitArtifactError::TooLarge {
            revision: revision.to_owned(),
            path: artifact_path.to_owned(),
            size,
        });
    }

    let blob = git(repo, &["cat-file", "blob", &locator], "artifact read")?;
    if blob.stdout.len() as u64 > MAX_GIT_BLOB_BYTES {
        return Err(GitArtifactError::TooLarge {
            revision: revision.to_owned(),
            path: artifact_path.to_owned(),
            size: blob.stdout.len() as u64,
        });
    }
    Ok(blob.stdout)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_option_like_and_ambiguous_revision_text() {
        for revision in ["", "--help", "-v", "a:b", "HEAD\n", "v 1"] {
            assert!(validate_revision(revision).is_err());
        }
        for revision in ["HEAD", "HEAD~1", "v1.2.0", "release/v1"] {
            assert!(validate_revision(revision).is_ok());
        }
    }

    #[test]
    fn rejects_traversal_absolute_and_ambiguous_git_paths() {
        for path in [
            "",
            "/tmp/contract.wasm",
            "../escape.wasm",
            "a/../token.wasm",
            "a//token.wasm",
            "./token.wasm",
            "a\\token.wasm",
            "C:token.wasm",
            "token\n.wasm",
        ] {
            assert!(validate_artifact_path(path).is_err());
        }
        assert!(validate_artifact_path("contracts/my token.wasm").is_ok());
        assert!(validate_artifact_path("token.wasm").is_ok());
    }
}
