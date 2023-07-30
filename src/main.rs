mod arguments;
mod fileutils;
mod img_renamer;
mod videos_renamer;

fn main() {
	let args = arguments::parse();
	match args {
		Ok(args) => {
			if args.rename_images {
				img_renamer::rename_images(&args).unwrap_or_else(|err| eprintln!("{}", err.to_string()));
			}
			if args.rename_videos {
				videos_renamer::rename_videos(&args).unwrap_or_else(|err| eprintln!("{}", err.to_string()));
			}
			if args.move_files {
				fileutils::move_files(&args.source_dir).unwrap_or_else(|err| eprintln!("{}", err.to_string()));
			}
		}
		Err(err) => {
			eprintln!("{:?}", err)
		}
	}
}




