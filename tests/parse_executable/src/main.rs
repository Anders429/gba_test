//! Simple program to extract executable path from `cargo test` output.
//! 
//! Specifically, this requires json output from running
//! `cargo test --no-run --message-format=json`. The executable path is printed to `stdout`.
//! 
//! The primary use of this program is in continuous integration, allowing the test executable to
//! be obtained programmatically.

use cargo_metadata::Message;
use std::{env::args, fs::File, io::BufReader};

fn main() -> anyhow::Result<()> {
    let args: Vec<_> = args().collect();

    // Read the file.
    let file = File::open(&args[1])?;
    let reader = BufReader::new(file);
    for message in Message::parse_stream(reader) {
        match message? {
            Message::CompilerArtifact(artifact) => {
                if let Some(executable) = artifact.executable {
                    print!("{executable}");
                    return Ok(());
                }
            }
            _ => {}
        }
    }

    Ok(())
}
