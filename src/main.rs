mod arguments;
mod fileutils;
mod img_renamer;
mod videos_renamer;

fn main() {
	let args = arguments::parse();
	match args {
		Ok(args) => {
			if args.rename_images {
				img_renamer::rename_images(&args).unwrap_or_else(|err| eprintln!("{}", err));
			}
			if args.rename_videos {
				videos_renamer::rename_videos(&args).unwrap_or_else(|err| eprintln!("{}", err));
			}
			if args.update_last_modified {
				fileutils::update_last_modified(&args.source_dir).unwrap_or_else(|err| eprintln!("{}", err));
			}
			if args.move_files {
				fileutils::move_files(&args.source_dir).unwrap_or_else(|err| eprintln!("{}", err));
			}
		}
		Err(err) => {
			eprintln!("{}", err)
		}
	}
}




