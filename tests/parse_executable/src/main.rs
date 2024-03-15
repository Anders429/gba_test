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
