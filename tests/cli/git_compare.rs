#![allow(clippy::unwrap_used)]

use std::{fs, path::Path, process::{Command, Output}};
use stellar_xdr::{Limits, ScSpecEntry, ScSpecFunctionInputV0, ScSpecFunctionV0, ScSpecTypeDef, ScSymbol, WriteXdr};
use tempfile::TempDir;

fn git(repo: &Path, args: &[&str]) -> Output {
    let out = Command::new("git").arg("-C").arg(repo).args(args).output().unwrap();
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    out
}

fn spec_wasm(kind: ScSpecTypeDef) -> Vec<u8> {
    let entry = ScSpecEntry::FunctionV0(ScSpecFunctionV0 {
        doc: "".try_into().unwrap(),
        name: ScSymbol("pay".try_into().unwrap()),
        inputs: vec![ScSpecFunctionInputV0 {
            doc: "".try_into().unwrap(),
            name: "amount".try_into().unwrap(),
            type_: kind,
        }].try_into().unwrap(),
        outputs: vec![ScSpecTypeDef::Bool].try_into().unwrap(),
    });
    let bytes = entry.to_xdr(Limits::none()).unwrap();
    let mut section = vec![14];
    section.extend(b"contractspecv0");
    section.extend(bytes);
    let mut wasm = b"\0asm\x01\0\0\0".to_vec();
    wasm.push(0);
    wasm.push(section.len() as u8);
    wasm.extend(section);
    wasm
}

fn setup() -> TempDir {
    let dir = TempDir::new().unwrap();
    let repo = dir.path();
    git(repo, &["init", "-q"]);
    git(repo, &["config", "user.name", "Test"]);
    git(repo, &["config", "user.email", "test@example.invalid"]);
    fs::write(repo.join("token.wasm"), spec_wasm(ScSpecTypeDef::I128)).unwrap();
    git(repo, &["add", "-A"]);
    git(repo, &["commit", "-qm", "old"]);
    git(repo, &["tag", "v1"]);
    fs::write(repo.join("token.wasm"), spec_wasm(ScSpecTypeDef::U128)).unwrap();
    git(repo, &["add", "-A"]);
    git(repo, &["commit", "-qm", "new"]);
    dir
}

fn compare(repo: &Path, from: &str, flags: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_stellaryn"))
        .args(["git", "--repo"]).arg(repo)
        .args(["--from", from, "--to", "HEAD", "--wasm", "token.wasm"])
        .args(flags).output().unwrap()
}

#[test]
fn committed_blobs_generate_breaking_report() {
    let repo = setup();
    let out = compare(repo.path(), "v1", &["--format", "json"]);
    assert_eq!(out.status.code(), Some(2));
    let value: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(value["analysis"]["verdict"], "INCOMPATIBLE");
    assert_eq!(value["before"], "git:v1:token.wasm");
}

#[test]
fn dirty_worktree_is_not_read_or_modified() {
    let repo = setup();
    fs::write(repo.path().join("token.wasm"), b"dirty").unwrap();
    let before = git(repo.path(), &["status", "--porcelain"]).stdout;
    let head = git(repo.path(), &["rev-parse", "HEAD"]).stdout;
    assert_eq!(compare(repo.path(), "v1", &[]).status.code(), Some(2));
    assert_eq!(before, git(repo.path(), &["status", "--porcelain"]).stdout);
    assert_eq!(head, git(repo.path(), &["rev-parse", "HEAD"]).stdout);
    assert_eq!(fs::read(repo.path().join("token.wasm")).unwrap(), b"dirty");
}

#[test]
fn unknown_revision_is_analysis_error_even_with_never_policy() {
    let repo = setup();
    let out = compare(repo.path(), "missing-ref", &["--fail-on", "never"]);
    assert_eq!(out.status.code(), Some(1));
    assert!(out.stdout.is_empty());
}

#[test]
fn never_policy_does_not_change_incompatible_verdict() {
    let repo = setup();
    let out = compare(repo.path(), "v1", &["--fail-on", "never", "--format", "json"]);
    assert_eq!(out.status.code(), Some(0));
    let value: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(value["analysis"]["verdict"], "INCOMPATIBLE");
}
