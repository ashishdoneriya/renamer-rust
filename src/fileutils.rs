use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Local, NaiveDate, NaiveDateTime, NaiveTime, TimeZone};
use exif::{In, Tag};
use filetime::FileTime;
use regex::Regex;

pub fn file_exists(file_path: &Path) -> bool {
	if let Ok(metadata) = fs::metadata(file_path) {
		metadata.is_file() // Check if it's a regular file
	} else {
		false // File does not exist or error occurred during metadata retrieval
	}
}

pub fn get_file_name(file_path: &Path) -> Option<&str> {
	return file_path.file_name()?.to_str();
}

pub fn get_file_extension(file_path: &Path) -> Option<String> {
	Some(file_path.extension()?.to_string_lossy().to_lowercase())
}

pub fn get_last_modified_date(file_path: &Path) -> Result<DateTime<Local>, Box<dyn Error>> {
	let metadata = fs::metadata(file_path)?;
	let modified = metadata.modified()?;
	Ok(DateTime::from(modified))
}

pub fn get_image_created_on_date(image_path: &Path) -> Result<Option<DateTime<Local>>, Box<dyn Error>> {
	let file = fs::File::open(image_path)?;
	let mut bufreader = std::io::BufReader::new(&file);
	let exifreader = exif::Reader::new();
	let exif = exifreader.read_from_container(&mut bufreader)?;
	for f in exif.fields() {
		if f.tag == Tag::DateTimeOriginal && f.ifd_num == In::PRIMARY {
			let value = f.display_value().with_unit(&exif).to_string();
			let parsed_date_time = NaiveDateTime::parse_from_str(&value, "%Y-%m-%d %H:%M:%S")?;
			return Ok(Some(Local.from_local_datetime(&parsed_date_time).single().unwrap()));
		}
	}
	return Ok(None);
}

pub fn use_last_modified_to_generate_new_path(file_path: &Path) -> Result<Option<PathBuf>, Box<dyn Error>> {
	let ext_opt = file_path.extension();
	if ext_opt.is_none() {
		return Ok(None);
	}
	let extension_opt = get_file_extension(file_path);
	if extension_opt.is_none() {
		return Ok(None);
	}
	let modified_date = get_last_modified_date(file_path)?;
	let new_name = format!(
		"{} {}.{}",
		modified_date.format("%Y-%m-%d"),
		modified_date.format("%H.%M.%S"),
		extension_opt.unwrap()
	);
	let mut new_path = file_path.to_path_buf();
	new_path.set_file_name(new_name);
	return Ok(Some(new_path));
}

pub fn rename_file_using_regex(file_path: &Path, regex_list: &Vec<Regex>) -> Result<Option<PathBuf>, Box<dyn Error>> {
	let new_name_opt = get_date_time_formatted_name(file_path, regex_list)?;
	return match new_name_opt {
		Some(new_name) => {
			let mut new_path = file_path.to_path_buf();
			new_path.set_file_name(new_name);
			Ok(Some(new_path))
		}
		_ => {
			Ok(None)
		}
	};
}

pub fn compile_regex(regex_str_list: Vec<&str>) -> Result<Vec<Regex>, Box<dyn Error>> {
	let mut regex_list = Vec::new();
	for regex_str in regex_str_list {
		regex_list.push(Regex::new(regex_str)?);
	}
	return Ok(regex_list);
}

fn get_date_time_formatted_name(file_path: &Path, regex_list: &Vec<Regex>) -> Result<Option<String>, Box<dyn Error>> {
	let file_name_opt = get_file_name(file_path);
	if file_name_opt.is_none() {
		return Ok(None);
	}
	for regex in regex_list {
		let file_name = file_name_opt.unwrap();
		if let Some(tokens) = regex.captures(&file_name) {
			return Ok(Some(format!(
				"{}-{}-{} {}.{}.{}.{}",
				&tokens[1],
				&tokens[2],
				&tokens[3],
				&tokens[4],
				&tokens[5],
				&tokens[6],
				&tokens[7].to_string().to_lowercase()
			)));
		};
	}
	return Ok(None);
}

pub fn move_files(src_dir: &String) -> Result<(), Box<dyn Error>> {
	for entry in fs::read_dir(src_dir)? {
		let file_path = entry?.path();
		if !file_path.is_file() || !has_valid_extension(&file_path) {
			continue;
		}
		let file_name_without_ext_op = get_file_name_without_extension(&file_path);
		if file_name_without_ext_op.is_none() {
			continue;
		}
		let file_name_without_ext = file_name_without_ext_op.unwrap();
		let regex = Regex::new(r"(\d{4})-(\d{2})-(\d{2}) (\d{2}).(\d{2}).(\d{2})")?;
		let capture_opt = regex.captures(&file_name_without_ext);
		if capture_opt.is_none() {
			continue;
		}
		let file_name_opt = get_file_name(&file_path);
		if file_name_opt.is_none() {
			continue;
		}
		let file_name = file_name_opt.unwrap();
		let token = capture_opt.unwrap();
		let existing_parent_dir_opt = get_parent_dir(&file_path);
		if existing_parent_dir_opt.is_none() {
			continue;
		}
		let existing_parent_dir = existing_parent_dir_opt.unwrap();
		let new_parent_dir = format!("{}", &token[1]);
		let new_child_dir = format!("{}", &token[2]);
		let new_path = generate_complete_path(&*existing_parent_dir,
											  &*new_parent_dir, &*new_child_dir, file_name);
		move_file(&file_path, new_path)?
	}
	Ok(())
}

pub fn update_last_modified(src_dir: &String) -> Result<(), Box<dyn Error>> {
	for entry in fs::read_dir(src_dir)? {
		let file_path = entry?.path();
		if !file_path.is_file() || !has_valid_extension(&file_path) {
			eprintln!("Invalid Extension {:?}", file_path);
			continue;
		}
		let file_name_without_ext_op = get_file_name_without_extension(&file_path);
		if file_name_without_ext_op.is_none() {
			eprintln!("Invalid file_name_without_ext_op {:?}", file_path);
			continue;
		}
		let file_name_without_ext = file_name_without_ext_op.unwrap();
		let regex = Regex::new(r"(\d{4})-(\d{2})-(\d{2}) (\d{2}).(\d{2}).(\d{2})")?;
		let capture_opt = regex.captures(&file_name_without_ext);
		if capture_opt.is_none() {
			eprintln!("Pattern not matched {:?}", file_path);
			continue;
		}
		let token = capture_opt.unwrap();
		let year: i32 = token[1].to_string().parse().unwrap();
		let month: u32 = token[2].to_string().parse().unwrap();
		let day: u32 = token[3].to_string().parse().unwrap();
		let date_opt = NaiveDate::from_ymd_opt(year, month, day);
		if date_opt.is_none() {
			eprintln!("date not matched {:?}", file_path);
			continue;
		}
		let hour: u32 = token[4].to_string().parse().unwrap();
		let minute: u32 = token[5].to_string().parse().unwrap();
		let second: u32 = token[6].to_string().parse().unwrap();

		// Create a specific time
		let time_opt = NaiveTime::from_hms_opt(hour, minute, second);
		if time_opt.is_none() {
			eprintln!("time not matched {:?}", file_path);
			continue;
		}
		// Combine the date and time to create a datetime
		let datetime = NaiveDateTime::new(date_opt.unwrap(), time_opt.unwrap());
		change_last_modified_time(&file_path, datetime);
	}
	Ok(())
}

fn change_last_modified_time(file_path: &Path, datetime: NaiveDateTime) {
	// Convert the NaiveDateTime to a Unix timestamp with nanoseconds set to 0
	let unix_timestamp = datetime.timestamp() - 19800;
	let nanoseconds = 0;
	let file_time = FileTime::from_unix_time(unix_timestamp, nanoseconds);

	// Set the new modification time for the file
	match filetime::set_file_mtime(file_path, file_time) {
		Ok(_) => println!("Last modified time of the file has been changed."),
		Err(err) => eprintln!("Error changing last modified time: {}", err),
	}
}

fn get_parent_dir(file_path: &Path) -> Option<&str> {
	file_path.parent()?.to_str()
}

fn get_file_name_without_extension(file_path: &Path) -> Option<String> {
	if let Some(file_name) = file_path.file_name() {
		if let Some(file_name_str) = file_name.to_str() {
			if let Some(pos) = file_name_str.rfind('.') {
				return Some(file_name_str[..pos].to_string());
			}
		}
	}
	None
}

fn generate_complete_path(base_dir: &str, parent_dir: &str, child_dir: &str, file_name: &str) -> String {
	let mut path = std::path::PathBuf::from(base_dir);
	path.push(parent_dir);
	path.push(child_dir);
	path.push(file_name);
	path.to_string_lossy().to_string()
}

fn move_file(src_path: &PathBuf, target_path: String) -> Result<(), Box<dyn Error>> {
	// Convert the target_path to a PathBuf.
	let target_path_buf = Path::new(&target_path).to_path_buf();

	// Check if the target path exists, and skip the move if it does.
	if target_path_buf.exists() {
		println!("Target file already exists. Skipping move.");
		return Ok(());
	}
	// Convert the target_path to a PathBuf.
	let target_path_buf = Path::new(&target_path).to_path_buf();
	// Create parent directories if they do not exist.
	if let Some(parent_dir) = target_path_buf.parent() {
		fs::create_dir_all(parent_dir)?;
	}
	// Move the file.
	fs::rename(src_path, target_path_buf)?;
	Ok(())
}

fn has_valid_extension(file_path: &Path) -> bool {
	let opt1 = get_valid_image_extension(file_path);
	if opt1.is_some() {
		return true;
	}
	let opt2 = get_valid_video_extension(file_path);
	if opt2.is_some() {
		return true;
	}
	return false;
}

pub fn get_valid_image_extension(file_path: &Path) -> Option<&str> {
	let ext_opt = file_path.extension()?.to_str()?;
	return match ext_opt {
		"avif" | "jpg" | "jpeg" | "png" | "gif" | "webp" => Some(ext_opt),
		_ => None
	};
}

pub fn get_valid_video_extension(file_path: &Path) -> Option<&str> {
	let ext_opt = file_path.extension()?.to_str()?;
	return match ext_opt {
		"mp4" => Some(ext_opt),
		_ => None
	};
}

