use std::error::Error;
use std::fs;
use std::path::PathBuf;

use image::ImageReader;
use image::GenericImageView;

use tauri::{ AppHandle, Manager, State, Emitter };
use tauri::async_runtime::spawn;

use rfd::{ FileDialog, MessageButtons, MessageDialog, MessageDialogResult };

use crate::{ DataState, BinType, ImageState, show_error_message, show_spinner, hide_spinner };
use crate::sprite_pack::get_colors_in_image;
use crate::sprite_pack::palette::Color;
use crate::text::{ FontState, CharEncoding, EncodingLanguage, re_decode_strings, refresh_encoding_menu };
use crate::file::{ FileState, set_file_modified };

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

enum StringType {
	Unknown,
	Menu,
	Dialog,
	Item,
	Character,
}

#[tauri::command]
pub fn import_strings(handle: AppHandle) {
	spawn(async move {
		let file_state: State<FileState> = handle.state();
		let data_state: State<DataState> = handle.state();

		let no_data = data_state.data_pack.lock().unwrap().is_none();
		if no_data {
			show_error_message("Open a BIN file to edit first".into());

		} else {
			let mut file_dialog = FileDialog::new()
				.add_filter("CSV", &["csv"]);

			if let Some(base_path) = file_state.base_path.lock().unwrap().as_ref() {
				file_dialog = file_dialog.set_directory(base_path);
			}

			let file_result = file_dialog.pick_file();

			if let Some(path) = file_result {
				show_spinner(&handle);
				match import_strings_from(&handle, &path) {
					Ok(()) => set_file_modified(&handle, true),
					Err(why) => show_error_message(why)
				}
				hide_spinner(&handle);
			}
		}
	});
}

pub fn import_strings_from(handle: &AppHandle, path: &PathBuf) -> Result<(), Box<dyn Error>> {
	let data_state: State<DataState> = handle.state();
	let font_state: State<FontState> = handle.state();
	let char_codes = &font_state.char_codes.lock().unwrap();

	let mut current_string_type = StringType::Unknown;

	let mut csv_reader = csv::Reader::from_path(path)?;
	let mut temp_translation = TamaStringTranslation::new(0);
	let mut last_line = String::new();

	let add_string = |string_type: &StringType, id: u16, new_string: &str| {
		match string_type {
			StringType::Menu => {
				let mut menu_strings_opt = data_state.menu_strings.lock().unwrap();
				if let Some(menu_strings) = menu_strings_opt.as_mut() {
					if let Some(menu_string) = menu_strings.get_mut(id as usize) {
						menu_string.set_string(char_codes, new_string);
					}
				}
			},

			StringType::Dialog => {
				let mut data_pack_opt = data_state.data_pack.lock().unwrap();
				if let Some(data_pack) = data_pack_opt.as_mut() {
					if let Some(tamastring) = data_pack.tamastrings.get_mut(id as usize) {
						tamastring.value.set_string(char_codes, new_string);
					}
				}
			},

			StringType::Item => {
				let mut data_pack_opt = data_state.data_pack.lock().unwrap();
				if let Some(data_pack) = data_pack_opt.as_mut() {
					if let Some(item) = data_pack.items.get_mut(id as usize) {
						item.name.set_string(char_codes, new_string);
					}
				}
			},

			StringType::Character => {
				let substrings = new_string.split("<br>");

				let mut new_name = String::new();
				let mut new_pronoun = String::new();
				let mut new_statement = String::new();
				let mut new_question1 = String::new();
				let mut new_question2 = String::new();

				for (i, substring) in substrings.enumerate() {
					match i {
						0 => new_name = substring.to_string(),
						1 => new_pronoun = substring.to_string(),
						2 => new_statement = substring.to_string(),
						3 => new_question1 = substring.to_string(),
						4 => new_question2 = substring.to_string(),
						_ => {}
					}
				}

				let mut data_pack_opt = data_state.data_pack.lock().unwrap();
				if let Some(data_pack) = data_pack_opt.as_mut() {
					if let Some(character) = data_pack.characters.get_mut(id as usize) {
						character.name.set_string(char_codes, &new_name);
						character.pronoun.set_string(char_codes, &new_pronoun);
						character.statement.set_string(char_codes, &new_statement);
						character.question1.set_string(char_codes, &new_question1);
						character.question2.set_string(char_codes, &new_question2);
					}
				}
			},
			_ => {}
		}
	};

	for result in csv_reader.records() {
		let record = result?;

		if let Some(id) = record.get(0) {

			if let Ok(id) = id.parse::<u16>() {
				if id > 0 {
					add_string(&current_string_type, temp_translation.id, &temp_translation.value);
				}

				temp_translation = TamaStringTranslation::new(id);
				if let Some(line) = record.get(3) {
					temp_translation.value = line.to_string();
					last_line = line.to_string();
				}

			} else if id.is_empty() && !temp_translation.value.is_empty() {
				if let Some(line) = record.get(3) {
					if !line.is_empty() {
						temp_translation.line_count += 1;
						if last_line.is_empty() {
							temp_translation.value = format!("{}<hr>{}", temp_translation.value, line);
						} else {
							temp_translation.value = format!("{}<br>{}", temp_translation.value, line);
						}
					}
					last_line = line.to_string();
				}
			} else {
				add_string(&current_string_type, temp_translation.id, &temp_translation.value);
				match id.to_uppercase().as_str() {
					"MENUS" => current_string_type = StringType::Menu,
					"DIALOG" | "STRINGS" => current_string_type = StringType::Dialog,
					"ITEMS" => current_string_type = StringType::Item,
					"CHARACTERS" => current_string_type = StringType::Character,
					_ => {}
				}
			}
		}
	}

	add_string(&current_string_type, temp_translation.id, &temp_translation.value);

	let mut menu_strings_opt = data_state.menu_strings.lock().unwrap();
	if let Some(menu_strings) = menu_strings_opt.as_mut() {
		handle.emit("update_menu_strings", (&menu_strings, false)).unwrap();
	}

	let mut data_pack_opt = data_state.data_pack.lock().unwrap();
	if let Some(data_pack) = data_pack_opt.as_mut() {
		handle.emit("update_tamastrings", (&data_pack.tamastrings, false)).unwrap();
		handle.emit("update_items", (&data_pack.items, false)).unwrap();
		handle.emit("update_characters", (&data_pack.characters, false)).unwrap();
	}

	Ok(())
}

#[tauri::command]
pub fn import_image_spritesheet(handle: AppHandle, image_index: usize) {
	spawn(async move {
		let file_state: State<FileState> = handle.state();

		let mut file_dialog = FileDialog::new()
			.add_filter("PNG", &["png"]);

		if let Some(base_path) = file_state.base_path.lock().unwrap().as_ref() {
			file_dialog = file_dialog.set_directory(base_path);
		}

		let file_result = file_dialog.pick_file();

		if let Some(path) = file_result {
			show_spinner(&handle);
			if let Err(why) = import_image_spritesheet_from(&handle, image_index, &path) {
				show_error_message(why);
			}
			hide_spinner(&handle);
		}
	});
}

fn import_image_spritesheet_from(handle: &AppHandle, image_index: usize, path: &PathBuf) -> Result<(), Box<dyn Error>> {
	let spritesheet = ImageReader::open(path)?.decode()?;
	let data_state: State<DataState> = handle.state();
	let font_state: State<FontState> = handle.state();
	let image_state: State<ImageState> = handle.state();

	let mut images = image_state.images.lock().unwrap();
	let subimages = images.get_mut(image_index)
		.ok_or(format!("Can't find subimages for image {}", image_index))?;

	let mut sprite_pack_opt = data_state.sprite_pack.lock().unwrap();
	let sprite_pack = sprite_pack_opt.as_mut().ok_or("Can't find sprite pack")?;
	let image_def = sprite_pack.image_defs.get_mut(image_index)
		.ok_or(format!("Can't find image def for image {}", image_index))?;

	let expected_width = image_def.subimage_width * image_def.subimage_count as u32;
	let expected_height = image_def.subimage_height;
	if spritesheet.width() != expected_width || spritesheet.height() != expected_height {
		return Err(format!("Spritesheet does not match expected dimensions: {}x{}", expected_width, expected_height).into());
	}

	let colors_in_spritesheet = get_colors_in_image(&spritesheet.to_rgba8());
	if colors_in_spritesheet.len() > 16 {
		return Err(format!("Spritesheet uses too many colors (the maximum is 16): {}", colors_in_spritesheet.len()).into());
	}
	if *data_state.lock_colors.lock().unwrap() {
		if image_def.colors_used != colors_in_spritesheet {
			return Err("Spritesheet uses colors not in original image. Try unlocking colors first.".into());
		}
	} else {
		image_def.colors_used = colors_in_spritesheet;
	}

	let mut x = 0;
	for subimage in subimages.iter_mut() {
		let mut new_subimage = spritesheet.view(x, 0, subimage.width(), subimage.height()).to_image();
		for pixel in new_subimage.pixels_mut() {
			*pixel = Color::from_rgba(pixel).as_rgba();
		}
		*subimage = new_subimage;
		x += subimage.width();
	}

	if let Some(BinType::Firmware) = *data_state.bin_type.lock().unwrap() {
		match image_index {
			98 => font_state.small_font_images.lock().unwrap().clone_from(subimages),
			99 => font_state.large_font_images.lock().unwrap().clone_from(subimages),
			_ => {}
		}
	}

	handle.emit("update_image", image_index).unwrap();

	Ok(())
}

#[tauri::command]
pub fn import_encoding(handle: AppHandle) {
	let do_the_thing = |handle: AppHandle| {
		let file_state: State<FileState> = handle.state();

		let mut file_dialog = FileDialog::new()
			.add_filter("JSON", &["json"]);

		if let Some(base_path) = file_state.base_path.lock().unwrap().as_ref() {
			file_dialog = file_dialog.set_directory(base_path);
		}

		let file_result = file_dialog.pick_file();

		if let Some(path) = file_result {
			spawn(async move {
				show_spinner(&handle);
				match import_encoding_from(&handle, &path) {
					Ok(()) => {
						let font_state: State<FontState> = handle.state();
						let char_codes = font_state.char_codes.lock().unwrap();
						handle.emit("update_char_codes", char_codes.clone()).unwrap();
						re_decode_strings(&handle, &char_codes)
					},
					Err(why) => show_error_message(why)
				}
				hide_spinner(&handle);
				refresh_encoding_menu(&handle);
			});
		} else {
			refresh_encoding_menu(&handle);
		}
	};

	let font_state: State<FontState> = handle.state();
	let current_encoding = font_state.encoding_language.lock().unwrap().clone();
	if current_encoding == EncodingLanguage::Custom {
		let dialog_result = MessageDialog::new()
			.set_title("Import Text Encoding")
			.set_description("This will overwrite your existing text encoding. Are you sure you want to continue?")
			.set_buttons(MessageButtons::YesNo)
			.show();

		if dialog_result == MessageDialogResult::Yes {
			do_the_thing(handle);
		} else {
			refresh_encoding_menu(&handle);
		}
	} else {
		do_the_thing(handle);
	}
}

pub fn import_encoding_from(handle: &AppHandle, path: &PathBuf) -> Result<(), Box<dyn Error>> {
	let file_string = fs::read_to_string(path)?;

	let font_state: State<FontState> = handle.state();
	let char_codes: Vec<CharEncoding> = serde_json::from_str(&file_string)?;

	*font_state.char_codes.lock().unwrap() = char_codes;
	*font_state.encoding_language.lock().unwrap() = EncodingLanguage::Custom;

	Ok(())
}
