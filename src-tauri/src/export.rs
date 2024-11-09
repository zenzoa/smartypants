use std::error::Error;
use std::path::{ PathBuf, Path };
use std::fs::File;
use std::io::prelude::*;

use tauri::{ AppHandle, Manager, State };
use tauri::async_runtime::spawn;

use rfd::FileDialog;

use crate::{ DataState, ImageState, show_error_message, show_spinner, hide_spinner };
use crate::text::FontState;
use crate::file::FileState;

#[tauri::command]
pub fn export_data(handle: AppHandle) {
	spawn(async move {
		let file_state: State<FileState> = handle.state();
		let data_state: State<DataState> = handle.state();

		let no_data = data_state.data_pack.lock().unwrap().is_none();
		if no_data {
			show_error_message("No data to export".into());

		} else {
			let mut file_dialog = FileDialog::new()
				.add_filter("JSON", &["json"]);

			if let Some(base_path) = file_state.base_path.lock().unwrap().as_ref() {
				file_dialog = file_dialog.set_directory(base_path);
			}

			if let Some(file_path) = file_state.file_path.lock().unwrap().as_ref() {
				if let Some(file_stem) = file_path.file_stem() {
					file_dialog = file_dialog.set_file_name(format!("{}.json", file_stem.to_string_lossy()));
				}
			}

			let file_result = file_dialog.save_file();

			if let Some(path) = file_result {
				show_spinner(&handle);
				if let Err(why) = export_data_to(&data_state, &path) {
					show_error_message(why);
				}
				hide_spinner(&handle);
			}
		}
	});
}

pub fn export_data_to(data_state: &DataState, path: &PathBuf) -> Result<(), Box<dyn Error>> {
	if let Some(data_pack) = data_state.data_pack.lock().unwrap().as_ref() {
		let serialized = serde_json::to_string(&data_pack)?;
		let mut file = File::create(path)?;
		file.write_all(serialized.as_bytes())?;
		Ok(())
	} else {
		Err("No data to export".into())
	}
}

#[tauri::command]
pub fn export_strings(handle: AppHandle) {
	spawn(async move {
		let file_state: State<FileState> = handle.state();
		let data_state: State<DataState> = handle.state();

		let no_data = data_state.data_pack.lock().unwrap().is_none();
		if no_data {
			show_error_message("No data to export".into());

		} else {
			let mut file_dialog = FileDialog::new()
				.add_filter("CSV", &["csv"]);

			if let Some(base_path) = file_state.base_path.lock().unwrap().as_ref() {
				file_dialog = file_dialog.set_directory(base_path);
			}

			if let Some(file_path) = file_state.file_path.lock().unwrap().as_ref() {
				if let Some(file_stem) = file_path.file_stem() {
					file_dialog = file_dialog.set_file_name(format!("{}.csv", file_stem.to_string_lossy()));
				}
			}

			let file_result = file_dialog.save_file();

			if let Some(path) = file_result {
				show_spinner(&handle);
				if let Err(why) = export_strings_to(&handle, &path) {
					show_error_message(why);
				}
				hide_spinner(&handle);
			}
		}
	});
}

pub fn export_strings_to(handle: &AppHandle, path: &PathBuf) -> Result<(), Box<dyn Error>> {
	let mut wtr = csv::Writer::from_path(path)?;
	let data_state: State<DataState> = handle.state();

	let blank_line = ["", "", ""];

	wtr.write_record(["ID", "Field", "Original Text"])?;

	let menu_strings_opt = data_state.menu_strings.lock().unwrap();
	if let Some(menu_strings) = menu_strings_opt.as_ref() {

		wtr.write_record(blank_line)?;
		wtr.write_record(["MENUS", "", ""])?;
		wtr.write_record(blank_line)?;

		for (i, str) in menu_strings.iter().enumerate() {
			let mut id_written = false;
			let pages = str.string.split("<hr>");
			for page in pages {
				let lines = page.split("<br>");
				for line in lines {
					if id_written {
						wtr.write_record(["", "", line])?;
					} else {
						wtr.write_record([&i.to_string(), "", line])?;
						id_written = true;
					}
				}
				wtr.write_record(blank_line)?;
			}
			wtr.write_record(blank_line)?;
		}
	}

	let data_pack_opt = data_state.data_pack.lock().unwrap();
	if let Some(data_pack) = data_pack_opt.as_ref() {

		wtr.write_record(blank_line)?;
		wtr.write_record(["STRINGS", "", ""])?;
		wtr.write_record(blank_line)?;

		for (i, tamastring) in data_pack.tamastrings.iter().enumerate() {
			let mut id_written = false;
			let pages = tamastring.value.string.split("<hr>");
			for page in pages {
				let lines = page.split("<br>");
				for line in lines {
					if id_written {
						wtr.write_record(["", "", line])?;
					} else {
						wtr.write_record([&i.to_string(), "", line])?;
						id_written = true;
					}
				}
				wtr.write_record(blank_line)?;
			}
			wtr.write_record(blank_line)?;
		}

		wtr.write_record(blank_line)?;
		wtr.write_record(["ITEMS", "", ""])?;
		wtr.write_record(blank_line)?;

		for (i, item) in data_pack.items.iter().enumerate() {
			wtr.write_record([&i.to_string(), "Name:", &item.name.string])?;
			wtr.write_record(blank_line)?;
		}

		wtr.write_record(blank_line)?;
		wtr.write_record(["CHARACTERS", "", ""])?;
		wtr.write_record(blank_line)?;

		for (i, char) in data_pack.characters.iter().enumerate() {
			wtr.write_record([&i.to_string(), "Name:", &char.name.string])?;
			wtr.write_record(["", "Pronoun:", &char.pronoun.string])?;
			wtr.write_record(["", "Statement Ending:", &char.statement.string])?;
			wtr.write_record(["", "Question Ending 1:", &char.question1.string])?;
			wtr.write_record(["", "Question Ending 2:", &char.question2.string])?;
			wtr.write_record(blank_line)?;
		}
	}

	wtr.flush()?;
	Ok(())
}

#[tauri::command]
pub fn export_images(handle: AppHandle) {
	spawn(async move {
		let file_state: State<FileState> = handle.state();
		let image_state: State<ImageState> = handle.state();

		if image_state.images.lock().unwrap().is_empty() || image_state.images.lock().unwrap()[0].is_empty() {
			show_error_message("No images to export".into());

		} else {
			let mut file_dialog = FileDialog::new()
				.add_filter("PNG image", &["png"]);

			if let Some(base_path) = file_state.base_path.lock().unwrap().as_ref() {
				file_dialog = file_dialog.set_directory(base_path);
			}

			if let Some(file_path) = file_state.file_path.lock().unwrap().as_ref() {
				if let Some(file_stem) = file_path.file_stem() {
					file_dialog = file_dialog.set_file_name(format!("{}.png", file_stem.to_string_lossy()));
				}
			}

			let file_result = file_dialog.save_file();

			if let Some(path) = file_result {
				show_spinner(&handle);
				if let Err(why) = export_images_to(&handle, &image_state, &path) {
					show_error_message(why);
				}
				hide_spinner(&handle);
			}
		}
	});
}

pub fn export_images_to(handle: &AppHandle, image_state: &ImageState, path: &Path) -> Result<(), Box<dyn Error>> {
	let base_name = path.file_stem().unwrap().to_string_lossy();
	let image_count = image_state.images.lock().unwrap().len();
	for i in 0..image_count {
		let image_path = path.with_file_name(format!("{}-{}", base_name, i)).with_extension("png");
		export_image_spritesheet_to(handle, i, &image_path)?;
	}
	Ok(())
}

#[tauri::command]
pub fn export_image_spritesheet(handle: AppHandle, image_index: usize) {
	spawn(async move {
		let file_state: State<FileState> = handle.state();

		let mut file_dialog = FileDialog::new()
			.add_filter("PNG image", &["png"]);

		if let Some(base_path) = file_state.base_path.lock().unwrap().as_ref() {
			file_dialog = file_dialog.set_directory(base_path);
		}

		if let Some(file_path) = file_state.file_path.lock().unwrap().as_ref() {
			if let Some(file_stem) = file_path.file_stem() {
				file_dialog = file_dialog.set_file_name(format!("{}-{}.png", file_stem.to_string_lossy(), image_index));
			}
		}

		let file_result = file_dialog.save_file();

		if let Some(path) = file_result {
			show_spinner(&handle);
			if let Err(why) = export_image_spritesheet_to(&handle, image_index, &path) {
				show_error_message(why);
			}
			hide_spinner(&handle);
		}
	});
}

fn export_image_spritesheet_to(handle: &AppHandle, image_index: usize, path: &Path) -> Result<(), Box<dyn Error>> {
	let data_state: State<DataState> = handle.state();
	let sprite_pack_guard = data_state.sprite_pack.lock().unwrap();
	let sprite_pack = sprite_pack_guard.as_ref().ok_or("No sprite pack found")?;
	let image_set = sprite_pack.image_sets.get(image_index)
		.ok_or(format!("Unable to find image definition {}", image_index))?;
	let spritesheet = match image_set.to_spritesheet() {
		Ok(spritesheet) => spritesheet,
		Err(why) => return Err(format!("Image Def {}: {}", image_index, why).into())
	};
	spritesheet.save(path)?;
	Ok(())
}

#[tauri::command]
pub fn export_encoding(handle: AppHandle) {
	spawn(async move {
		let file_state: State<FileState> = handle.state();
		let font_state: State<FontState> = handle.state();

		let mut file_dialog = FileDialog::new()
			.add_filter("JSON", &["json"]);

		if let Some(base_path) = file_state.base_path.lock().unwrap().as_ref() {
			file_dialog = file_dialog.set_directory(base_path);
		}

		let file_result = file_dialog.save_file();

		if let Some(path) = file_result {
			show_spinner(&handle);
			if let Err(why) = export_encoding_to(&font_state, &path) {
				show_error_message(why);
			}
			hide_spinner(&handle);
		}
	});
}

fn export_encoding_to(font_state: &FontState, path: &PathBuf) -> Result<(), Box<dyn Error>> {
	let serialized = serde_json::to_string(&font_state.char_codes)?;
	let mut file = File::create(path)?;
	file.write_all(serialized.as_bytes())?;
	Ok(())
}
