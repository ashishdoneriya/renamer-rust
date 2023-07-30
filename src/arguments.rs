use std::fmt;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
	#[arg(long, default_value = ".")]
	pub source_dir: String,
	#[arg(long)]
	pub rename_images: bool,
	#[arg(long)]
	pub rename_videos: bool,
	#[arg(long)]
	pub use_file_name: bool,
	#[arg(long)]
	pub use_last_modified: bool,
	#[arg(long)]
	pub use_image_properties: bool,
	#[arg(long)]
	pub move_files: bool,
	#[arg(long)]
	pub update_last_modified: bool
}

#[derive(Debug)]
pub struct InvalidArgumentsError {
	message: String,
}

impl fmt::Display for InvalidArgumentsError {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{}", self.message)
	}
}

pub fn parse() -> Result<Args, InvalidArgumentsError> {
	let args = Args::parse();
	if !args.rename_videos && !args.rename_images && !args.move_files && !args.update_last_modified {
		return Err(InvalidArgumentsError { message: "Kindly use at least one argument from below\
			\n\t--rename-videos\
			\n\t--rename-images\
			\n\t--move-files\
			\n\t--update-last-modified\
			\nUse --help for more options".to_string() });
	}
	let mut count = 0;
	if args.use_last_modified {
		count += 1;
	}
	if args.use_file_name {
		count += 1;
	}
	if args.rename_videos && count == 0 {
		if count == 0 {
			return Err(InvalidArgumentsError { message: "Kindly use --use-last-modified or --use-file-name along with --rename-videos".to_string() })
		} else if count == 2 {
			return Err(InvalidArgumentsError { message: "Kindly use exactly one from --use-last-modified or --use-file-name along with --rename-videos".to_string() })
		}
	}

	if args.use_image_properties {
		count += 1;
	}

	if args.rename_images && count == 0 {
		if count == 0 {
			return Err(InvalidArgumentsError { message: "Kindly use --use-image-properties or --use-last-modified or --use-file-name along with --rename-images".to_string() })
		} else if count > 1 {
			return Err(InvalidArgumentsError { message: "Kindly use exactly one from --use-image-properties or --use-last-modified or --use-file-name along with --rename-images".to_string() })
		}
	}

	return Ok(args);
}