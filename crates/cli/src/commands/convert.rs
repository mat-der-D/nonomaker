use std::path::PathBuf;

use clap::Args;
use nonomaker_core::{ImageConvertParams, format::grid_to_json, image_to_grid};

use crate::error::CliError;
use crate::io::{read_bytes, write_output};

#[derive(Args, Debug)]
pub struct ConvertArgs {
    /// Input image file path
    #[arg(long, value_name = "PATH")]
    pub input: PathBuf,
    /// Output file path (stdout when omitted)
    #[arg(long, value_name = "PATH")]
    pub output: Option<PathBuf>,
    /// Gaussian blur strength (0-5)
    #[arg(long, default_value = "1.0")]
    pub smooth_strength: f32,
    /// Edge merge strength (0-1)
    #[arg(long, default_value = "0.3")]
    pub edge_strength: f32,
    /// Output grid width (5-50)
    #[arg(long, default_value = "20")]
    pub grid_width: u32,
    /// Output grid height (5-50)
    #[arg(long, default_value = "20")]
    pub grid_height: u32,
    /// Binarization threshold (0-255)
    #[arg(long, default_value = "128")]
    pub threshold: u8,
    /// Minimum connected-component size to keep (0-20)
    #[arg(long, default_value = "0")]
    pub noise_removal: u32,
}

pub fn run(args: ConvertArgs) -> Result<(), CliError> {
    validate_args(&args)?;

    let bytes = read_bytes(&args.input)?;
    let params = ImageConvertParams {
        grid_width: args.grid_width,
        grid_height: args.grid_height,
        smooth_strength: args.smooth_strength,
        threshold: args.threshold,
        edge_strength: args.edge_strength,
        noise_removal: args.noise_removal,
    };

    let grid = image_to_grid(&bytes, &params).map_err(|e| CliError::ImageDecode(e.to_string()))?;
    let json = grid_to_json(&grid)?;

    write_output(args.output.as_deref(), &json)
}

fn validate_args(args: &ConvertArgs) -> Result<(), CliError> {
    if !(5..=50).contains(&args.grid_width) {
        return Err(CliError::Validation(format!(
            "grid_width must be between 5 and 50, got {}",
            args.grid_width
        )));
    }
    if !(5..=50).contains(&args.grid_height) {
        return Err(CliError::Validation(format!(
            "grid_height must be between 5 and 50, got {}",
            args.grid_height
        )));
    }
    if !(0.0..=5.0).contains(&args.smooth_strength) {
        return Err(CliError::Validation(format!(
            "smooth_strength must be between 0 and 5, got {}",
            args.smooth_strength
        )));
    }
    if !(0.0..=1.0).contains(&args.edge_strength) {
        return Err(CliError::Validation(format!(
            "edge_strength must be between 0 and 1, got {}",
            args.edge_strength
        )));
    }
    if args.noise_removal > 20 {
        return Err(CliError::Validation(format!(
            "noise_removal must be between 0 and 20, got {}",
            args.noise_removal
        )));
    }

    Ok(())
}
