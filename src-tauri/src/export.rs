use std::error::Error;
use std::path::PathBuf;
use std::fs::File;
use std::io::prelude::*;

use tauri::{ AppHandle, Manager, State };
use tauri::async_runtime::spawn;

use rfd::FileDialog;

use crate::{ DataState, ImageState, show_error_message, show_spinner, hide_spinner };

#[tauri::command]
pub fn export_data(handle: AppHandle) {
	spawn(async move {
		let data_state: State<DataState> = handle.state();

		let no_data = if let None = *data_state.data_pack.lock().unwrap() { true } else { false };
		if no_data {
			show_error_message("No data to export".into());

		} else {
			let mut file_dialog = FileDialog::new()
				.add_filter("JSON", &["json"]);

			if let Some(base_path) = data_state.base_path.lock().unwrap().as_ref() {
				file_dialog = file_dialog.set_directory(base_path);
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
pub fn export_images(handle: AppHandle) {
	spawn(async move {
		let data_state: State<DataState> = handle.state();
		let image_state: State<ImageState> = handle.state();

		if image_state.images.lock().unwrap().len() == 0 || image_state.images.lock().unwrap()[0].len() == 0 {
			show_error_message("No images to export".into());

		} else {
			let mut file_dialog = FileDialog::new()
				.add_filter("PNG image", &["png"]);
			if let Some(base_path) = data_state.base_path.lock().unwrap().as_ref() {
				file_dialog = file_dialog.set_directory(base_path);
			}
			let file_result = file_dialog.save_file();

			if let Some(path) = file_result {
				show_spinner(&handle);
				if let Err(why) = export_images_to(&image_state, &path) {
					show_error_message(why);
				}
				hide_spinner(&handle);
			}
		}

	});
}

pub fn export_images_to(image_state: &ImageState, path: &PathBuf) -> Result<(), Box<dyn Error>> {
	let base_name = path.file_stem().unwrap().to_string_lossy();

	let image_count = image_state.images.lock().unwrap().len();
	for i in 0..image_count {
		let subimage_count = image_state.images.lock().unwrap().get(i).ok_or("")?.len();
		for j in 0..subimage_count {
			let img_path = path.with_file_name(&format!("{}-{}-{}", base_name, i, j)).with_extension("png");
			image_state.images.lock().unwrap()
				.get(i).ok_or("image not found")?
				.get(j).ok_or("subimage not found")?
				.save(img_path)?;
		}
	}

	Ok(())
}
