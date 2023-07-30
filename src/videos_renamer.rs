use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use regex::Regex;

use crate::arguments::Args;
use crate::fileutils;
use crate::fileutils::file_exists;

pub fn rename_videos(args: &Args) -> Result<(), Box<dyn Error>> {
	let files = get_video_files_in_directory(&args.source_dir)?;
	let mut regex_str_list = Vec::new();
	regex_str_list.push(r"VID_(\d{4})(\d{2})(\d{2})_(\d{2})(\d{2})(\d{2}).(mp4)");
	regex_str_list.push(r"VID_(\d{4})(\d{2})(\d{2})_(\d{2})(\d{2})(\d{2})_HSR_\d{3}.(mp4)");
	let regex_list = fileutils::compile_regex(regex_str_list)?;
	for file_path in files {
		match generate_new_path(&file_path, args, &regex_list) {
			Ok(Some(new_path)) => {
				if file_exists(&new_path) {
					println!("Skipped {:?}", file_path);
				} else {
					match fs::rename(&file_path, &new_path) {
						Ok(_) => println!("Renamed {:?} to {:?}", file_path, new_path),
						Err(e) => eprintln!("Error renaming {:?}: {}", file_path, e)
					}
				}
			}
			Ok(None) => {
				println!("Skipped {:?}", file_path);
			}
			Err(err) => {
				println!("Couldn't get path of file {:?}, Err : {:?}", file_path, err);
			}
		}
	}
	Ok(())
}


fn get_video_files_in_directory(source_dir: &String) -> Result<Vec<PathBuf>, Box<dyn Error>> {
	let current_dir = fs::read_dir(source_dir)?;
	let mut video_files = Vec::new();
	for entry in current_dir {
		let file_path = entry?.path();
		if file_path.is_file() && fileutils::get_valid_video_extension(&file_path).is_some() {
			video_files.push(file_path);
		}
	}
	Ok(video_files)
}

fn generate_new_path(file_path: &Path, args: &Args, regex_list: &Vec<Regex>) -> Result<Option<PathBuf>, Box<dyn Error>> {
	if args.use_last_modified {
		return fileutils::use_last_modified_to_generate_new_path(file_path);
	}
	if args.use_file_name {
		return fileutils::rename_file_using_regex(file_path, regex_list);
	}
	Ok(None)
}


