use std::error::Error;

use crate::data_view::{ DataView, get_encoded_char, encode_string, words_to_bytes };
use crate::data_pack::{ DataPack, get_data_pack };
use crate::sprite_pack::{ SpritePack, get_sprite_pack };

#[derive(Clone, serde::Serialize)]
pub struct Firmware {
	pub data_pack: DataPack,
	pub sprite_pack: SpritePack,
	pub menu_strings: Vec<String>
}

pub fn read_firmware(data: &DataView) -> Result<Firmware, Box<dyn Error>> {
	let data_pack = get_data_pack(&data.chunk(0x6CE000, 0x730000 - 0x6CE000))?;
	let sprite_pack = get_sprite_pack(&data.chunk(0x730000, data.len() - 0x730000))?;

	let menu_strings = match data.find_bytes(&[0xF9, 0x01, 0xFB, 0x01]) {
		Some(start_index) => {
			read_menu_strings(&data.chunk(start_index, data.len() - start_index))?
		},
		None => {
			return Err("Can't find menu strings".into())
		}
	};

	Ok(Firmware { data_pack, sprite_pack, menu_strings })
}

pub fn read_menu_strings(data: &DataView) -> Result<Vec<String>, Box<dyn Error>> {
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
		sizes.push(size as usize)
	}

	if data.len() < 2 * sizes.last().unwrap() {
		return Err("Cannot read all menu strings".into());
	}

	let mut menu_strings = Vec::new();
	for i in 0..offsets.len() - 1 {
		let mut menu_string = String::new();
		for j in 0..sizes[i] {
			let c = data.get_u16(offsets[i]*2 + j*2);
			if c > 0 {
				menu_string.push_str(&get_encoded_char(c));
			} else {
				break;
			}
		}
		menu_strings.push(menu_string);
	}

	Ok(menu_strings)
}

pub fn save_menu_strings(original_data: &[u8], menu_strings: &[String]) -> Result<Vec<u8>, Box<dyn Error>> {
	let mut offsets: Vec<u16> = vec![menu_strings.len() as u16 + 2];
	for (i, string) in menu_strings.iter().enumerate() {
		let last_offset = offsets[i];
		let string_size = encode_string(string).len() as u16;
		offsets.push(last_offset + string_size + 1);
	}

	let mut string_data: Vec<u16> = Vec::new();
	for string in menu_strings {
		for word in encode_string(string) {
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

	match DataView::new(original_data).find_bytes(&[0xF9, 0x01, 0xFB, 0x01]) {
		Some(start_index) => {
			let mut new_data = Vec::from(original_data);
			let end_index = start_index + new_menu_strings_data.len();
			let _: Vec<_> = new_data.splice(start_index..end_index, new_menu_strings_data).collect();
			Ok(new_data)
		},
		None => {
			Err("Can't find menu strings".into())
		}
	}
}
