use gba_test_runner::{Conclusion, Outcome, Status, Trial};
use bincode::serde::decode_borrowed_from_slice;
use cargo_metadata::Message;
use std::{fs, path::PathBuf, process::{Command, Stdio}};

#[test]
fn single() {
    // Build and run the test.
    let mut command = Command::new("cargo")
        .args([
            "test",
            #[cfg(not(debug_assertions))]
            "--release",
            "--message-format=json-render-diagnostics",
            ]).stdout(Stdio::piped())
        .current_dir("tests/single").spawn().expect("failed to build test");

    // command.kill();

    // Find the executable name.
    let reader = std::io::BufReader::new(command.stdout.take().unwrap());
    let mut executable_name = None;
    for message in Message::parse_stream(reader) {
        match message.unwrap() {
            Message::CompilerArtifact(artifact) => {
                if let Some(executable) = artifact.executable {
                    executable_name = Some(executable);
                }
            },
            _ => () // Unknown message
        }
    }

    // Open the save file.
    let mut save_file = PathBuf::from(executable_name.expect("unable to find executable name"));
    save_file.set_extension("sav");
    let output = fs::read(save_file).expect("unable to open save file");
    let conclusion: Conclusion = decode_borrowed_from_slice(&output, gba_test_runner::BINCODE_CONFIG).expect("unable to decode save data");

    // Compare the output with the expected output.
    assert_eq!(conclusion, Conclusion {
        status: Status::Success,
        trials: vec![
            Trial {
                name: "it_works",
                outcome: Outcome::Passed,
            }
        ],
    });
}
