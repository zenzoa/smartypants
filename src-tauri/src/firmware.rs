use std::error::Error;

use tauri::{ AppHandle, State, Manager };

use crate::DataState;
use crate::data_view::{ DataView, words_to_bytes };
use crate::data_pack::{ DataPack, get_data_pack, save_data_pack };
use crate::sprite_pack::{ SpritePack, get_sprite_pack, save_sprite_pack };
use crate::text::{ Text, FontState, decode_string };

const FIRMWARE_DATA_PACK_SIZE: usize = 0x730000 - 0x6CE000;

#[derive(Clone, serde::Serialize)]
pub struct Firmware {
	pub data_pack: DataPack,
	pub sprite_pack: SpritePack,
	pub menu_strings: Vec<Text>
}

pub fn read_firmware(handle: &AppHandle, data: &DataView) -> Result<Firmware, Box<dyn Error>> {
	let font_state: State<FontState> = handle.state();

	let data_pack = get_data_pack(&font_state, &data.chunk(0x6CE000, FIRMWARE_DATA_PACK_SIZE))?;
	let sprite_pack = get_sprite_pack(&data.chunk(0x730000, data.len() - 0x730000))?;

	let menu_strings = match data.find_bytes(&[0xF9, 0x01, 0xFB, 0x01]) {
		Some(start_index) => {
			read_menu_strings(&font_state, &data.chunk(start_index, data.len() - start_index))?
		},
		None => {
			return Err("Can't find menu strings".into())
		}
	};

	let new_data = save_firmware(handle, &data.data)?;
	if new_data == data.data {
		println!("Firmware export matches!");
	}

	Ok(Firmware { data_pack, sprite_pack, menu_strings })
}

pub fn save_firmware(handle: &AppHandle, original_data: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
	let data_state: State<DataState> = handle.state();
	let font_state: State<FontState> = handle.state();

	let mut new_data = DataView::new(original_data);

	if let Some(old_menu_strings) = data_state.menu_strings.lock().unwrap().as_ref() {
		match new_data.find_bytes(&[0xF9, 0x01, 0xFB, 0x01]) {
			Some(start_index) => {
				let new_menu_strings_data = save_menu_strings(font_state.clone(), old_menu_strings)?;
				let end_index = start_index + new_menu_strings_data.len();
				let _: Vec<_> = new_data.data.splice(start_index..end_index, new_menu_strings_data).collect();
			},
			None => {
				return Err("Can't find menu strings".into());
			}
		}
	}

	if let Some(old_data_pack) = data_state.data_pack.lock().unwrap().as_ref() {
		let data_pack_data_view = new_data.chunk(0x6CE000, FIRMWARE_DATA_PACK_SIZE);
		let new_data_pack_data = save_data_pack(old_data_pack, &data_pack_data_view)?;
		let padding_size = FIRMWARE_DATA_PACK_SIZE - new_data_pack_data.len();
		let padding = vec![0; padding_size];
		let new_data_pack_data = [new_data_pack_data, padding].concat();
		let _: Vec<_> = new_data.data.splice(0x6CE000..0x730000, new_data_pack_data).collect();
	}

	if let Some(old_sprite_pack) = data_state.sprite_pack.lock().unwrap().as_ref() {
		let new_sprite_pack_data = save_sprite_pack(old_sprite_pack)?;
		let end_of_sprite_pack = 0x730000 + new_sprite_pack_data.len();
		let _: Vec<_> = new_data.data.splice(0x730000..end_of_sprite_pack, new_sprite_pack_data).collect();
	}

	Ok(new_data.data)
}

pub fn read_menu_strings(font_state: &FontState, data: &DataView) -> Result<Vec<Text>, Box<dyn Error>> {
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
		menu_strings.push(Text::from_data(font_state, &text_data));
	}

	Ok(menu_strings)
}

pub fn save_menu_strings(font_state: State<FontState>, menu_strings: &[Text]) -> Result<Vec<u8>, Box<dyn Error>> {
	let mut offsets: Vec<u16> = vec![menu_strings.len() as u16 + 2];
	for (i, txt) in menu_strings.iter().enumerate() {
		let last_offset = offsets[i];
		let string_size = decode_string(&font_state, &txt.string).len() as u16;
		offsets.push(last_offset + string_size + 1);
	}

	let mut string_data: Vec<u16> = Vec::new();
	for txt in menu_strings {
		for word in decode_string(&font_state, &txt.string) {
			string_data.push(word);
		}
		string_data.push(0);
	}

	let new_menu_strings_words = [
		vec![menu_strings.len() as u16],
		offsets,
		string_data
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