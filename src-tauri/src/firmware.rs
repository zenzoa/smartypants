use std::fs;
use std::error::Error;

use tauri::{ AppHandle, State, Manager };
use tauri::path::BaseDirectory;

use crate::{ DataState, update_window_title };
use crate::data_view::{ DataView, words_to_bytes };
use crate::data_pack::{ DataPack, get_data_pack, save_data_pack };
use crate::sprite_pack::SpritePack;
use crate::text::{ Text, FontState };
use crate::file::set_file_modified;

const FIRMWARE_DATA_PACK_SIZE: usize = 0x730000 - 0x6CE000;
const PATCH_HEADER_START: [u8; 8] = [0x4F, 0x86, 0xA0, 0x86, 0x0A, 0xFE, 0x84, 0x30];

#[derive(Clone)]
pub struct Firmware {
	pub data_pack: DataPack,
	pub sprite_pack: SpritePack,
	pub menu_strings: Vec<Text>,
	pub use_patch_header: bool
}

pub fn read_firmware(handle: &AppHandle, data: &DataView) -> Result<Firmware, Box<dyn Error>> {
	let use_patch_header = data.data.starts_with(&PATCH_HEADER_START);

	let data_pack_start = if use_patch_header { 0x6CE000 + 1024 } else { 0x6CE000 };
	let sprite_pack_start = if use_patch_header { 0x730000 + 1024 } else { 0x730000 };
	let sprite_pack_size = data.len() - sprite_pack_start;

	let data_pack = get_data_pack(handle, &data.chunk(data_pack_start, FIRMWARE_DATA_PACK_SIZE))?;
	let sprite_pack = SpritePack::from_data(&data.chunk(sprite_pack_start, sprite_pack_size))?;

	let menu_strings = match data.find_bytes(&[0xF9, 0x01, 0xFB, 0x01]) {
		Some(start_index) => {
			read_menu_strings(handle, &data.chunk(start_index, data.len() - start_index))?
		},
		None => {
			return Err("Can't find menu strings".into())
		}
	};

	Ok(Firmware { data_pack, sprite_pack, menu_strings, use_patch_header })
}

pub fn save_firmware(handle: &AppHandle, original_data: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
	let data_state: State<DataState> = handle.state();

	let mut new_data = DataView::new(original_data);

	if let Some(old_menu_strings) = data_state.menu_strings.lock().unwrap().as_ref() {
		match new_data.find_bytes(&[0xF9, 0x01, 0xFB, 0x01]) {
			Some(start_index) => {
				let new_menu_strings_data = save_menu_strings(old_menu_strings)?;
				let end_index = start_index + new_menu_strings_data.len();
				new_data.data.splice(start_index..end_index, new_menu_strings_data);
			},
			None => {
				return Err("Can't find menu strings".into());
			}
		}
	}

	let already_has_header = new_data.data.starts_with(&PATCH_HEADER_START);
	let data_pack_start = if already_has_header { 0x6CE000 + 1024 } else { 0x6CE000 };
	let sprite_pack_start = if already_has_header { 0x730000 + 1024 } else { 0x730000 };

	if let Some(data_pack) = data_state.data_pack.lock().unwrap().as_ref() {
		let mut data_pack_data = save_data_pack(data_pack, data_pack_start)?;
		if data_pack_data[77893] == 0x09 {
			data_pack_data[77893] = 0x89; // Fix a probably irrelevant discrepancy at the end of table 13 (graphic node offsets)
		}
		let padding_size = FIRMWARE_DATA_PACK_SIZE - data_pack_data.len();
		let padding = vec![0; padding_size];
		data_pack_data.extend_from_slice(&padding);
		let end_of_data_pack = data_pack_start + FIRMWARE_DATA_PACK_SIZE;
		new_data.data.splice(data_pack_start..end_of_data_pack, data_pack_data);
	}

	if let Some(sprite_pack) = data_state.sprite_pack.lock().unwrap().as_mut() {
		let sprite_pack_data = sprite_pack.as_bytes()?;
		let end_of_sprite_pack = sprite_pack_start + sprite_pack_data.len();
		new_data.data.splice(sprite_pack_start..end_of_sprite_pack, sprite_pack_data);
	}

	let use_patch_header = *data_state.use_patch_header.lock().unwrap();
	if use_patch_header && !already_has_header {
		let header_path = handle.path().resolve("resources/patch_header.bin", BaseDirectory::Resource)?;
		let header_file = fs::read(header_path)?;
		new_data.data.splice(0..0, header_file);
		new_data.data.splice((new_data.len() - 1024)..new_data.len(), Vec::new());
	} else if !use_patch_header && already_has_header {
		let padding = vec![0xFF; 1024];
		new_data.data.splice(0..1024, Vec::new());
		new_data.data.extend_from_slice(&padding);
	}

	Ok(new_data.data)
}

pub fn read_menu_strings(handle: &AppHandle, data: &DataView) -> Result<Vec<Text>, Box<dyn Error>> {
	let font_state: State<FontState> = handle.state();
	let char_codes = &font_state.char_codes.lock().unwrap();

	let num_strings = data.get_u16(0) as usize;

	if data.len() < 2 * num_strings + 2 {
		return Err("Cannot read all menu string offsets".into());
	}

	let mut offsets = Vec::new();
	for i in 0..num_strings+1 {
		let offset = data.get_u16((i+1)*2) as usize;
		offsets.push(offset);
	}

	let mut sizes = Vec::new();
	for i in 0..num_strings {
		let size = offsets[i+1] - offsets[i];
		sizes.push(size)
	}

	if data.len() < 2 * sizes.last().unwrap() {
		return Err("Cannot read all menu strings".into());
	}

	let mut menu_strings: Vec<Text> = Vec::new();
	for i in 0..offsets.len() - 1 {
		let mut text_data: Vec<u16> = Vec::new();
		for j in 0..sizes[i] {
			let word = data.get_u16(offsets[i]*2 + j*2);
			if word > 0 {
				text_data.push(word);
			}
		}
		menu_strings.push(Text::from_data(char_codes, &text_data));
	}

	Ok(menu_strings)
}

pub fn save_menu_strings(menu_strings: &[Text]) -> Result<Vec<u8>, Box<dyn Error>> {
	let mut offsets: Vec<u16> = vec![menu_strings.len() as u16 + 2];
	for (i, menu_string) in menu_strings.iter().enumerate() {
		let last_offset = offsets[i];
		let string_size = menu_string.data.len() as u16;
		offsets.push(last_offset + string_size + 1);
	}

	let mut words: Vec<u16> = Vec::new();
	for menu_string in menu_strings {
		words = [words, menu_string.data.clone()].concat();
		words.push(0);
	}

	let new_menu_strings_words = [
		vec![menu_strings.len() as u16],
		offsets,
		words
	].concat();
	let new_menu_strings_data = words_to_bytes(&new_menu_strings_words);

	if new_menu_strings_data.len() > 29990 {
		return Err(format!("Menu strings ({} bytes) don't fit in available space (29990 bytes)", new_menu_strings_data.len()).into());
	}

	let padding_size = 29990 - new_menu_strings_data.len();
	let padding = vec![0; padding_size];
	let new_menu_strings_data = [new_menu_strings_data, padding].concat();

	Ok(new_menu_strings_data)
}

#[tauri::command]
pub fn update_menu_string(handle: AppHandle, index: usize, new_menu_string: String) -> Option<Text> {
	let data_state: State<DataState> = handle.state();
	let font_state: State<FontState> = handle.state();
	let char_codes = &font_state.char_codes.lock().unwrap();

	let mut menu_strings_opt = data_state.menu_strings.lock().unwrap();
	if let Some(menu_strings) = menu_strings_opt.as_mut() {
		if let Some(menu_string) = menu_strings.get_mut(index) {
			menu_string.set_string(char_codes, &new_menu_string);
			set_file_modified(&handle, true);
			update_window_title(&handle);
			return Some(menu_string.clone());
		}
	}

	None
}

#[tauri::command]
pub fn set_patch_header(handle: AppHandle, enable: bool) {
	let data_state: State<DataState> = handle.state();
	*data_state.use_patch_header.lock().unwrap() = enable;
}
