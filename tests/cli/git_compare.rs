#![allow(clippy::unwrap_used)]

use std::{fs, path::Path, process::{Command, Output}};
use stellar_xdr::{Limits, ScSpecEntry, ScSpecFunctionInputV0, ScSpecFunctionV0, ScSpecTypeDef, ScSymbol, WriteXdr};
use tempfile::TempDir;

fn git(repo: &Path, args: &[&str]) -> Output {
    let out = Command::new("git").arg("-C").arg(repo).args(args).output().unwrap();
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    out
}

fn leb(bytes: &mut Vec<u8>, mut value: u32) {
    loop {
        let mut byte = (value & 0x7f) as u8;
        value >>= 7;
        if value != 0 {
            byte |= 0x80;
        }
        bytes.push(byte);
        if value == 0 {
            break;
        }
    }
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
    let mut section = Vec::new();
    leb(&mut section, 14);
    section.extend(b"contractspecv0");
    section.extend(bytes);
    let mut wasm = b"\0asm\x01\0\0\0".to_vec();
    wasm.push(0);
    leb(&mut wasm, section.len() as u32);
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

#[test]
fn ancestry_revisions_are_accepted() {
    let repo = setup();
    let output = compare(repo.path(), "HEAD~1", &[]);
    assert_eq!(output.status.code(), Some(2));
}

#[test]
fn same_git_revision_is_compatible() {
    let repo = setup();
    let output = Command::new(env!("CARGO_BIN_EXE_stellaryn"))
        .args(["git", "--repo"]).arg(repo.path())
        .args(["--from", "v1", "--to", "v1", "--wasm", "token.wasm"])
        .output().unwrap();
    assert_eq!(output.status.code(), Some(0));
    assert!(String::from_utf8(output.stdout).unwrap().contains("COMPATIBLE"));
}

#[test]
fn missing_committed_artifact_and_path_traversal_fail() {
    let repo = setup();
    for path in ["missing.wasm", "../token.wasm", "./token.wasm"] {
        let output = Command::new(env!("CARGO_BIN_EXE_stellaryn"))
            .args(["git", "--repo"]).arg(repo.path())
            .args(["--from", "v1", "--to", "HEAD", "--wasm", path])
            .output().unwrap();
        assert_eq!(output.status.code(), Some(1));
        assert!(output.stdout.is_empty());
    }
}

#[test]
fn renamed_wasm_path_can_be_supplied_per_revision() {
    let repo = setup();
    fs::create_dir_all(repo.path().join("releases")).unwrap();
    fs::rename(repo.path().join("token.wasm"), repo.path().join("releases/new token.wasm")).unwrap();
    git(repo.path(), &["add", "-A"]);
    git(repo.path(), &["commit", "-qm", "move artifact"]);
    let output = Command::new(env!("CARGO_BIN_EXE_stellaryn"))
        .args(["git", "--repo"]).arg(repo.path())
        .args(["--from", "v1", "--to", "HEAD", "--wasm", "token.wasm"])
        .args(["--after-wasm", "releases/new token.wasm", "--format", "json"])
        .output().unwrap();
    assert_eq!(output.status.code(), Some(2));
    let data: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(data["after"], "git:HEAD:releases/new token.wasm");
}

#[test]
fn corrupt_committed_target_is_an_error_even_with_never() {
    let repo = setup();
    fs::write(repo.path().join("token.wasm"), b"invalid").unwrap();
    git(repo.path(), &["add", "-A"]);
    git(repo.path(), &["commit", "-qm", "malformed artifact"]);
    let output = compare(repo.path(), "v1", &["--fail-on", "never", "--format", "json"]);
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8(output.stderr).unwrap().contains("after Git artifact specification"));
}

#[test]
fn repeated_git_json_is_deterministic() {
    let repo = setup();
    let a = compare(repo.path(), "v1", &["--format", "json"]);
    let b = compare(repo.path(), "v1", &["--format", "json"]);
    assert_eq!(a.status.code(), Some(2));
    assert_eq!(a.stdout, b.stdout);
}
