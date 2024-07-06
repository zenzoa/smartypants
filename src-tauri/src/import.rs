use std::error::Error;
use std::path::PathBuf;
use std::collections::HashMap;
use csv;

use tauri::{ AppHandle, Manager, State };
use tauri::async_runtime::spawn;

use rfd::FileDialog;

use crate::{ DataState, show_error_message, show_spinner, hide_spinner, update_window_title };

#[derive(Clone, Debug, serde::Deserialize)]
struct TamaStringTranslation {
	id: u16,
	value: String,
	line_count: usize
}

impl TamaStringTranslation {
	pub fn new(id: u16) -> TamaStringTranslation {
		TamaStringTranslation {
			id,
			value: String::new(),
			line_count: 0
		}
	}
}

#[tauri::command]
pub fn import_strings(handle: AppHandle) {
	import_strings_base(handle, import_strings_from);
}

#[tauri::command]
pub fn import_menu_strings(handle: AppHandle) {
	import_strings_base(handle, import_menu_strings_from);
}

pub fn import_strings_base(handle: AppHandle, callback: fn(&AppHandle, &PathBuf) -> Result<(), Box<dyn Error>>) {
	spawn(async move {
		let data_state: State<DataState> = handle.state();

		let no_data = if let None = *data_state.data_pack.lock().unwrap() { true } else { false };
		if no_data {
			show_error_message("Open a BIN file to edit first".into());

		} else {
			let mut file_dialog = FileDialog::new()
				.add_filter("CSV", &["csv"]);

			if let Some(base_path) = data_state.base_path.lock().unwrap().as_ref() {
				file_dialog = file_dialog.set_directory(base_path);
			}

			let file_result = file_dialog.pick_file();

			if let Some(path) = file_result {
				show_spinner(&handle);
				match callback(&handle, &path) {
					Ok(()) => {
						*data_state.is_modified.lock().unwrap() = true;
						update_window_title(&handle);
					}
					Err(why) => show_error_message(why)
				}
				hide_spinner(&handle);
			}
		}
	});
}

pub fn import_strings_from(handle: &AppHandle, path: &PathBuf) -> Result<(), Box<dyn Error>> {
	let data_state: State<DataState> = handle.state();

	let translation_list = parse_csv(path)?;

	if let Some(data_pack) = data_state.data_pack.lock().unwrap().as_mut() {
		for tamastring in data_pack.strings.iter_mut() {
			if let Some(new_string) = translation_list.get(&tamastring.id.entity_id) {
				tamastring.value = new_string.value.clone();
			}
		}
		handle.emit("update_strings", data_pack.strings.clone()).unwrap();
	}

	Ok(())
}

pub fn import_menu_strings_from(handle: &AppHandle, path: &PathBuf) -> Result<(), Box<dyn Error>> {
	let data_state: State<DataState> = handle.state();

	let translation_list = parse_csv(path)?;

	let mut new_menu_strings = Vec::new();

	if let Some(menu_strings) = data_state.menu_strings.lock().unwrap().as_ref() {
		for i in 0..menu_strings.len() {
			if let Some(new_string) = translation_list.get(&(i as u16)) {
				new_menu_strings.push(new_string.value.clone());
			} else {
				return Err(format!("Missing menu string {}", i).into());
			}
		}
		handle.emit("update_menu_strings", &new_menu_strings).unwrap();
	}

	*data_state.menu_strings.lock().unwrap() = Some(new_menu_strings);

	Ok(())
}

fn parse_csv(path: &PathBuf) -> Result<HashMap<u16, TamaStringTranslation>, Box<dyn Error>> {
	let mut csv_reader = csv::Reader::from_path(&path)?;
	let mut translation_list = HashMap::new();
	let mut temp_translation = TamaStringTranslation::new(0);

	for result in csv_reader.records() {
		let record = result?;

		if let Some(id) = record.get(0) {
			if let Ok(id) = u16::from_str_radix(id, 10) {
				if id > 0 {
					translation_list.insert(temp_translation.id, temp_translation);
				}

				temp_translation = TamaStringTranslation::new(id);
				if let Some(line) = record.get(2) {
					temp_translation.value = line.to_string();
				}

			} else if temp_translation.value.len() > 0 {
				if let Some(line) = record.get(2) {
					if line.len() > 0 {
						temp_translation.line_count += 1;
						if temp_translation.line_count == 2 || temp_translation.line_count == 4 {
							temp_translation.value = format!("{}<hr>{}", temp_translation.value, line);
						} else {
							temp_translation.value = format!("{}<br>{}", temp_translation.value, line);
						}
					}
				}
			}
		}
	}

	translation_list.insert(temp_translation.id, temp_translation);

	Ok(translation_list)
}
