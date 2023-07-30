use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

use filetime::FileTime;
use regex::Regex;

use crate::arguments::Args;
use crate::fileutils;

pub fn rename_images(args: &Args) -> Result<(), Box<dyn Error>> {
	let files = get_image_files_in_directory(&args.source_dir)?;

	let mut regex_str_list = Vec::new();
	regex_str_list.push(r"IMG_(\d{4})(\d{2})(\d{2})_(\d{2})(\d{2})(\d{2}).(jpg)");
	regex_str_list.push(r"IMG_(\d{4})(\d{2})(\d{2})_(\d{2})(\d{2})(\d{2})~\d.(jpg)");
	let regex_list = fileutils::compile_regex(regex_str_list)?;
	for file_path in files {
		match generate_new_path(&file_path, args, &regex_list) {
			Ok(Some(new_path)) => {
				if fileutils::file_exists(&new_path) {
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

fn get_image_files_in_directory(source_dir: &String) -> Result<Vec<PathBuf>, Box<dyn Error>> {
	let current_dir = fs::read_dir(source_dir)?;
	let mut image_files = Vec::new();
	for entry in current_dir {
		let file_path = entry?.path();
		if file_path.is_file() && fileutils::get_valid_image_extension(&file_path).is_some() {
			image_files.push(file_path);
		}
	}
	Ok(image_files)
}

fn generate_new_path(file_path: &Path, args: &Args, regex_list: &Vec<Regex>) -> Result<Option<PathBuf>, Box<dyn Error>> {
	if args.use_image_properties {
		return use_metadata_to_generate_new_path(file_path, args);
	}
	if args.use_last_modified {
		return fileutils::use_last_modified_to_generate_new_path(file_path);
	}
	if args.use_file_name {
		return fileutils::rename_file_using_regex(file_path, regex_list);
	}
	Ok(None)
}

fn use_metadata_to_generate_new_path(file_path: &Path, args: &Args) -> Result<Option<PathBuf>, Box<dyn Error>> {
	return match fileutils::get_image_created_on_date(file_path) {
		Ok(Some(created_on)) => {
			let extension_opt = fileutils::get_file_extension(file_path);
			if extension_opt.is_none() {
				return Ok(None);
			}
			let new_name = format!(
				"{} {}.{}",
				created_on.format("%Y-%m-%d"),
				created_on.format("%H.%M.%S"),
				extension_opt.unwrap()
			);
			let mut new_path = file_path.to_path_buf();

			new_path.set_file_name(new_name);
			if args.change_last_modified && ! fileutils::file_exists(&new_path) {
				let system_time: std::time::SystemTime = created_on.clone().into(); // Convert to SystemTime
				let file_time = FileTime::from(system_time); // Convert to FileTime

				filetime::set_file_mtime(file_path, file_time)
					.unwrap_or_else(|err| {
						eprintln!("Couldn't change modified time of a file, err - {:?}", err)
					});
			}
			Ok(Some(new_path))
		}
		_ => {
			Ok(None)
		}
	};
}