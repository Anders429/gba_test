#![cfg(all(feature = "bincode", feature = "alloc"))]

use bincode::serde::decode_borrowed_from_slice;
use cargo_metadata::Message;
use gba_test_runner::{Conclusion, Outcome, Status, Trial};
use std::{
    fs,
    path::PathBuf,
    process::{Command, Stdio},
};

#[test]
fn single() {
    // Build and run the test.
    let mut command = Command::new("cargo")
        .args([
            "test",
            #[cfg(not(debug_assertions))]
            "--release",
            "--message-format=json-render-diagnostics",
        ])
        .stdout(Stdio::piped())
        .current_dir("tests/single")
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
    let mut save_file = PathBuf::from(executable_name.expect("unable to find executable name"));
    save_file.set_extension("sav");

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
                        break decode_borrowed_from_slice(&output, gba_test_runner::BINCODE_CONFIG)
                            .expect("unable to decode save data");
                    }
                }
            }
        }
        std::thread::sleep(std::time::Duration::from_secs(1));
    };

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

    // It's fine for this not to succeed, as the test has already been completed. This cleanup is
    // best-effort only.
    #[allow(unused_must_use)]
    {
        command.kill();
    }
}
