#![cfg(all(feature = "postcard", feature = "alloc"))]

use cargo_metadata::Message;
use gba_test_runner::{Conclusion, Outcome, Status, Trial};
use std::{
    env, fs,
    path::PathBuf,
    process::{Command, Stdio},
};

#[test]
fn pass() {
    // Build and run the test.
    let mut command = Command::new("cargo")
        .args([
            "test",
            #[cfg(not(debug_assertions))]
            "--release",
            "--message-format=json-render-diagnostics",
        ])
        .stdout(Stdio::piped())
        .current_dir("tests/pass")
        .spawn()
        .expect("failed to build test");

    // Find the executable name.
    let reader = std::io::BufReader::new(command.stdout.as_mut().unwrap());
    let mut executable_name = None;
    for message in Message::parse_stream(reader) {
        match message.unwrap() {
            Message::CompilerArtifact(artifact) => {
                if let Some(executable) = artifact.executable {
                    executable_name = Some(executable);
                }
            }
            Message::BuildFinished(_) => {
                break;
            }
            _ => (), // Unknown message
        }
    }

    // Produce the save file name.
    let save_file = {
        let mut save_file = PathBuf::from(executable_name.expect("unable to find executable name"));
        save_file.set_extension("sav");
        env::current_dir()
            .expect("unable to find current directory")
            .join(format!(
                "tests/pass/{}",
                save_file
                    .file_name()
                    .expect("unable to obtain save file name")
                    .to_str()
                    .expect("unable to convert save file name to string")
            ))
    };

    let output = loop {
        if let Ok(output) = fs::read(&save_file) {
            break output;
        }
        std::thread::sleep(std::time::Duration::from_secs(1));
    };

    let conclusion: Conclusion = loop {
        if let Some(&first_byte) = output.get(0) {
            if let Ok(status) = first_byte.try_into() {
                match status {
                    Status::Running => continue,
                    _ => {
                        break postcard::from_bytes(&output).expect("unable to decode save data");
                    }
                }
            }
        }
        std::thread::sleep(std::time::Duration::from_secs(1));
    };

    // Clean up.
    fs::remove_file(save_file).expect("could not delete save file");
    // It's fine for this not to succeed, as the test has already been completed. This cleanup is
    // best-effort only.
    #[allow(unused_must_use)]
    {
        command.kill();
    }

    // Compare the output with the expected output.
    assert_eq!(
        conclusion,
        Conclusion {
            status: Status::Success,
            trials: vec![Trial {
                name: "it_works",
                outcome: Outcome::Passed,
            }],
        }
    );
}

#[test]
fn ignore() {
    // Build and run the test.
    let mut command = Command::new("cargo")
        .args([
            "test",
            #[cfg(not(debug_assertions))]
            "--release",
            "--message-format=json-render-diagnostics",
        ])
        .stdout(Stdio::piped())
        .current_dir("tests/ignore")
        .spawn()
        .expect("failed to build test");

    // Find the executable name.
    let reader = std::io::BufReader::new(command.stdout.as_mut().unwrap());
    let mut executable_name = None;
    for message in Message::parse_stream(reader) {
        match message.unwrap() {
            Message::CompilerArtifact(artifact) => {
                if let Some(executable) = artifact.executable {
                    executable_name = Some(executable);
                }
            }
            Message::BuildFinished(_) => {
                break;
            }
            _ => (), // Unknown message
        }
    }

    // Produce the save file name.
    let save_file = {
        let mut save_file = PathBuf::from(executable_name.expect("unable to find executable name"));
        save_file.set_extension("sav");
        env::current_dir()
            .expect("unable to find current directory")
            .join(format!(
                "tests/ignore/{}",
                save_file
                    .file_name()
                    .expect("unable to obtain save file name")
                    .to_str()
                    .expect("unable to convert save file name to string")
            ))
    };

    let output = loop {
        if let Ok(output) = fs::read(&save_file) {
            break output;
        }
        std::thread::sleep(std::time::Duration::from_secs(1));
    };

    let conclusion: Conclusion = loop {
        if let Some(&first_byte) = output.get(0) {
            if let Ok(status) = first_byte.try_into() {
                match status {
                    Status::Running => continue,
                    _ => {
                        break postcard::from_bytes(&output).expect("unable to decode save data");
                    }
                }
            }
        }
        std::thread::sleep(std::time::Duration::from_secs(1));
    };

    // Clean up.
    fs::remove_file(save_file).expect("could not delete save file");
    // It's fine for this not to succeed, as the test has already been completed. This cleanup is
    // best-effort only.
    #[allow(unused_must_use)]
    {
        command.kill();
    }

    // Compare the output with the expected output.
    assert_eq!(
        conclusion,
        Conclusion {
            status: Status::Success,
            trials: vec![Trial {
                name: "it_works",
                outcome: Outcome::Ignored,
            }],
        }
    );
}

#[test]
fn fail() {
    // Build and run the test.
    let mut command = Command::new("cargo")
        .args([
            "test",
            #[cfg(not(debug_assertions))]
            "--release",
            "--message-format=json-render-diagnostics",
        ])
        .stdout(Stdio::piped())
        .current_dir("tests/fail")
        .spawn()
        .expect("failed to build test");

    // Find the executable name.
    let reader = std::io::BufReader::new(command.stdout.as_mut().unwrap());
    let mut executable_name = None;
    for message in Message::parse_stream(reader) {
        match message.unwrap() {
            Message::CompilerArtifact(artifact) => {
                if let Some(executable) = artifact.executable {
                    executable_name = Some(executable);
                }
            }
            Message::BuildFinished(_) => {
                break;
            }
            _ => (), // Unknown message
        }
    }

    // Produce the save file name.
    let save_file = {
        let mut save_file = PathBuf::from(executable_name.expect("unable to find executable name"));
        save_file.set_extension("sav");
        env::current_dir()
            .expect("unable to find current directory")
            .join(format!(
                "tests/fail/{}",
                save_file
                    .file_name()
                    .expect("unable to obtain save file name")
                    .to_str()
                    .expect("unable to convert save file name to string")
            ))
    };

    let output = loop {
        if let Ok(output) = fs::read(&save_file) {
            break output;
        }
        std::thread::sleep(std::time::Duration::from_secs(1));
    };

    let conclusion: Conclusion = loop {
        if let Some(&first_byte) = output.get(0) {
            if let Ok(status) = first_byte.try_into() {
                match status {
                    Status::Running => continue,
                    _ => {
                        break postcard::from_bytes(&output).expect("unable to decode save data");
                    }
                }
            }
        }
        std::thread::sleep(std::time::Duration::from_secs(1));
    };

    // Clean up.
    fs::remove_file(save_file).expect("could not delete save file");
    // It's fine for this not to succeed, as the test has already been completed. This cleanup is
    // best-effort only.
    #[allow(unused_must_use)]
    {
        command.kill();
    }

    // Compare the output with the expected output.
    assert_eq!(
        conclusion,
        Conclusion {
            status: Status::Failure,
            trials: vec![Trial {
                name: "it_works",
                outcome: Outcome::Failed {
                    message: "panicked at 'assertion failed: `(left == right)`\n  left: `4`,\n right: `5`', src/lib.rs:28:9",
                },
            }],
        }
    );
}
