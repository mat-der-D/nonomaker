use std::{
    fs,
    io::{self, Read},
    path::Path,
};

use crate::error::CliError;

pub fn read_input(path: Option<&Path>) -> Result<String, CliError> {
    match path {
        None => {
            let mut buf = String::new();
            io::stdin().read_to_string(&mut buf)?;
            Ok(buf)
        }
        Some(p) => Ok(fs::read_to_string(p)?),
    }
}

pub fn read_bytes(path: &Path) -> Result<Vec<u8>, CliError> {
    Ok(fs::read(path)?)
}

pub fn write_output(path: Option<&Path>, content: &str) -> Result<(), CliError> {
    match path {
        None => {
            println!("{content}");
            Ok(())
        }
        Some(p) => Ok(fs::write(p, content)?),
    }
}
