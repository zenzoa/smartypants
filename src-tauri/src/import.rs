use std::error::Error;
use std::fs;
use std::path::PathBuf;

use regex::Regex;

use image::{ ImageReader, GenericImageView, RgbaImage };

use tauri::{ AppHandle, Manager, State, Emitter };
use tauri::async_runtime::spawn;

use rfd::{ FileDialog, MessageButtons, MessageDialog, MessageDialogResult };

use crate::{ DataState, BinType, ImageState, show_error_message, show_spinner, hide_spinner };
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
			spawn(async move {
				match import_strings_from(&handle, &path) {
					Ok(()) => set_file_modified(&handle, true),
					Err(why) => show_error_message(why)
				}
				hide_spinner(&handle);
			});
		}
	}
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
				if let Some(line) = record.get(2) {
					temp_translation.value = line.to_string();
					last_line = line.to_string();
				}

			} else if id.is_empty() && !temp_translation.value.is_empty() {
				if let Some(line) = record.get(2) {
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
pub fn import_images(handle: AppHandle) {
	let file_state: State<FileState> = handle.state();

	let mut file_dialog = FileDialog::new();

	if let Some(base_path) = file_state.base_path.lock().unwrap().as_ref() {
		file_dialog = file_dialog.set_directory(base_path);
	}

	let file_result = file_dialog.pick_folder();

	if let Some(path) = file_result {
		show_spinner(&handle);
		spawn(async move {
			match import_images_from(&handle, &path) {
				Ok(()) => handle.emit("update_images", ()).unwrap(),
				Err(why) => show_error_message(why)
			}
			hide_spinner(&handle);
		});
	}
}

fn import_images_from(handle: &AppHandle, path: &PathBuf) -> Result<(), Box<dyn Error>> {
	let re = Regex::new(r".+-(\d+).[Pp][Nn][Gg]$")?;
	for entry in path.read_dir()? {
		let entry = entry?;
		let entry_path = entry.path();
		if let Some(filename) = entry_path.file_name() {
			if let Some(caps) = re.captures(&filename.to_string_lossy()) {
				if let Some(image_index_str) = caps.get(1) {
					if let Ok(image_index) = usize::from_str_radix(image_index_str.as_str(), 10) {
						import_image_spritesheet_from(&handle, image_index, &entry_path)?;
					}
				}
			}
		}
	}
	Ok(())
}

#[tauri::command]
pub fn import_image_spritesheet(handle: AppHandle, image_index: usize) {
	let file_state: State<FileState> = handle.state();

	let mut file_dialog = FileDialog::new()
		.add_filter("PNG", &["png"]);

	if let Some(base_path) = file_state.base_path.lock().unwrap().as_ref() {
		file_dialog = file_dialog.set_directory(base_path);
	}

	let file_result = file_dialog.pick_file();

	if let Some(path) = file_result {
		show_spinner(&handle);
		spawn(async move {
			match import_image_spritesheet_from(&handle, image_index, &path) {
				Ok(()) => handle.emit("update_image", image_index).unwrap(),
				Err(why) => show_error_message(why)
			}
			hide_spinner(&handle);
		});
	}
}

fn import_image_spritesheet_from(handle: &AppHandle, image_index: usize, path: &PathBuf) -> Result<(), Box<dyn Error>> {
	let spritesheet = ImageReader::open(path)?.decode()?;
	let data_state: State<DataState> = handle.state();
	let font_state: State<FontState> = handle.state();
	let image_state: State<ImageState> = handle.state();

	let mut sprite_pack_opt = data_state.sprite_pack.lock().unwrap();
	let sprite_pack = sprite_pack_opt.as_mut().ok_or("Can't find sprite pack")?;
	let image_set = sprite_pack.image_sets.get_mut(image_index)
		.ok_or(format!("Can't find image def for image {}", image_index))?;

	let palette_count = image_set.palettes.len();
	let subimage_count = image_set.subimages.len();

	// make sure the spritesheet has the expected dimensions
	let expected_width = image_set.width * subimage_count as u32;
	let expected_height = image_set.height * palette_count as u32;
	if spritesheet.width() != expected_width || spritesheet.height() != expected_height {
		return Err(format!("Spritesheet does not match expected dimensions: {}x{}", expected_width, expected_height).into());
	}

	let mut images = image_state.images.lock().unwrap();
	let subimage_imgs = images.get_mut(image_index)
		.ok_or(format!("Can't find subimages for image {}", image_index))?;

	// get main palette
	let img = spritesheet.view(0, 0, expected_width, image_set.height).to_image();
	let mut main_palette = Vec::new();
	let mut color_indexes = Vec::new();
	for (i, color) in img.pixels().enumerate() {
		let color = Color::from_rgba(color);
		if !main_palette.contains(&color) {
			main_palette.push(color.clone());
			color_indexes.push(i);
		}
	}

	// get additional palettes
	image_set.palettes = vec![main_palette];
	if palette_count > 1 {
		for i in 1..palette_count {
			let y = i as u32 * image_set.height;
			let mut palette = Vec::new();
			let img = spritesheet.view(0, y, expected_width, image_set.height).to_image();
			let pixels: Vec<Color> = img.pixels().map(|p| Color::from_rgba(p)).collect();
			for color_index in &color_indexes {
				palette.push(pixels[*color_index]);
			}
			image_set.palettes.push(palette);
		}
	}

	// get pixel data and images
	for (i, subimage) in image_set.subimages.iter_mut().enumerate() {
		let x = i as u32 * image_set.width;
		let img = spritesheet.view(x, 0, image_set.width, image_set.height).to_image();
		subimage.pixel_data = Vec::new();
		for color in img.pixels() {
			let color = Color::from_rgba(color);
			let pixel = image_set.palettes[0].iter().position(|c| *c == color).ok_or("Can't find color")?;
			subimage.pixel_data.push(pixel as u32);
		}
	}

	// update images
	let mut new_subimage_imgs = Vec::new();
	for i in 0..palette_count {
		new_subimage_imgs = [new_subimage_imgs, image_set.to_images(i)?].concat();
	}
	*subimage_imgs = new_subimage_imgs;

	if let Some(BinType::Firmware) = *data_state.bin_type.lock().unwrap() {
		match image_index {
			98 => font_state.small_font_images.lock().unwrap().clone_from(subimage_imgs),
			99 => font_state.large_font_images.lock().unwrap().clone_from(subimage_imgs),
			_ => {}
		}
	}

	Ok(())
}

pub fn spritesheet_to_images(path: &PathBuf, subimage_count: u32, palette_count: u32) -> Result<Vec<RgbaImage>, Box<dyn Error>> {
	let spritesheet = ImageReader::open(path)?.decode()?;
	let mut subimages = Vec::new();

	let subimage_width = spritesheet.width() / subimage_count;
	let subimage_height = spritesheet.height() / palette_count;

	for y in 0..palette_count {
		for x in 0..subimage_count {
			let subimage = spritesheet.view(x * subimage_width, y * subimage_height, subimage_width, subimage_height);
			subimages.push(subimage.to_image());
		}
	}

	Ok(subimages)
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
			show_spinner(&handle);
			spawn(async move {
				match import_encoding_from(&handle, &path) {
					Ok(()) => {
						let font_state: State<FontState> = handle.state();
						let char_codes = font_state.char_codes.lock().unwrap();
						handle.emit("update_char_codes", char_codes.clone()).unwrap();
						handle.emit("open_encoding_dialog", ()).unwrap();
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
