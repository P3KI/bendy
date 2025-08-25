use assert_cmd::Command;
use assert_fs::{prelude::*, TempDir};

fn cmd() -> Command {
    Command::cargo_bin(env!("CARGO_PKG_NAME"))
        .expect("Must be able to find bencode-pretty binary")
}

#[test]
fn basic_stdin_test() {
    cmd()
        .write_stdin(b"li1ei2ee")
        .assert()
        .success()
        .stdout("l\n\ti1e\n\ti2e\ne\n");
}

#[test]
fn basic_file_test() {
    let tmpdir = TempDir::new()
        .expect("Must be able to create temp dir");
    let f1 = tmpdir.child("f1.bencode");
    f1.write_binary(
        b"li1ei2ee"
    ).expect("Must be able to create temp file");
    let f1p = f1.to_str().expect("Temp file path was not valid unicode");

    cmd()
        .arg(format!("{}", f1p))
        .assert()
        .success()
        .stdout("l\n\ti1e\n\ti2e\ne\n");

    let f2 = tmpdir.child("f2.bencode");
    f2.write_binary(
        b"d3:aaali123eee"
    ).expect("Must be able to create temp file");
    let f2p = f2.to_str().expect("Temp file path was not valid unicode");

    cmd()
        .arg(format!("{}", f2p))
        .assert()
        .success()
        .stdout("d\n\t3:aaa\n\tl\n\t\ti123e\n\te\ne\n");

    cmd()
        .arg(format!("{}", f1p))
        .arg(format!("{}", f2p))
        .assert()
        .success()
        .stdout("l\n\ti1e\n\ti2e\ne\nd\n\t3:aaa\n\tl\n\t\ti123e\n\te\ne\n");
}
