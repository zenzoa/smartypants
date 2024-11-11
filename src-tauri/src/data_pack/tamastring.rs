use std::error::Error;
use serde::{ Serialize, Deserialize };
use tauri::{ AppHandle, Manager, State };

use super::EntityId;
use crate::{ DataState, update_window_title };
use crate::data_view::{ DataView, words_to_bytes };
use crate::text::{ Text, FontState };
use crate::file::set_file_modified;

#[derive(Clone, Serialize, Deserialize)]
pub struct TamaString {
	pub id: EntityId,
	pub unknown1: u16,
	pub unknown2: u16,
	pub unknown3: u16,
	pub value: Text
}

impl TamaString {
	pub fn to_words(&self) -> Vec<u16> {
		let mut data: Vec<u16> = vec![
			self.id.to_word(),
			self.unknown1,
			self.unknown2,
			self.unknown3
		];
		for word in &self.value.data {
			data.push(*word);
		}
		data.push(0);
		data
	}

	pub fn set_card_id(&mut self, old_card_id: u8, new_card_id: u8) {
		self.id.set_card_id(old_card_id, new_card_id);
	}
}

pub fn get_tamastrings(handle: &AppHandle, data: &DataView) -> Vec<TamaString> {
	let font_state: State<FontState> = handle.state();
	let char_codes = &font_state.char_codes.lock().unwrap();

	let mut strings = Vec::new();

	let mut i = 0;
	while i + 10 <= data.len() {
		let id = EntityId::new(data.get_u16(i));

		let unknown1 = data.get_u16(i+2);
		let unknown2 = data.get_u16(i+4);
		let unknown3 = data.get_u16(i+6);

		let mut text_data = Vec::new();
		let mut str_len = 0;

		while i + 8 + str_len*2 < data.len() && data.get_u16(i + 8 + str_len*2) > 0 {
			let word = data.get_u16(i + 8 + str_len*2);
			text_data.push(word);
			str_len += 1;
		}
		i += 10 + str_len*2;

		strings.push(TamaString {
			id,
			unknown1,
			unknown2,
			unknown3,
			value: Text::from_data(char_codes, &text_data)
		});
	}

	strings
}

pub fn save_tamastrings(tamastrings: &[TamaString]) -> Result<(Vec<u8>, Vec<u8>), Box<dyn Error>> {
	let mut words: Vec<u16> = Vec::new();
	let mut offsets: Vec<u16> = Vec::new();

	for tamastring in tamastrings {
		offsets.push(words.len() as u16);
		words.push(tamastring.id.to_word());
		words.push(tamastring.unknown1);
		words.push(tamastring.unknown2);
		words.push(tamastring.unknown3);
		words.extend_from_slice(&tamastring.value.data);
		words.push(0);
	}

	offsets.push(0xFFFF);

	Ok((words_to_bytes(&offsets), words_to_bytes(&words)))
}

#[tauri::command]
pub fn update_tamastring(handle: AppHandle, index: usize, new_tamastring: TamaString) -> Option<TamaString> {
	let data_state: State<DataState> = handle.state();
	let font_state: State<FontState> = handle.state();
	let char_codes = &font_state.char_codes.lock().unwrap();

	let mut data_pack_opt = data_state.data_pack.lock().unwrap();
	if let Some(data_pack) = data_pack_opt.as_mut() {
		if let Some(tamastring) = data_pack.tamastrings.get_mut(index) {
			tamastring.unknown1 = new_tamastring.unknown1;
			tamastring.unknown2 = new_tamastring.unknown2;
			tamastring.unknown3 = new_tamastring.unknown3;
			tamastring.value.set_string(char_codes, &new_tamastring.value.string);

			set_file_modified(&handle, true);
			update_window_title(&handle);
			return Some(tamastring.clone());
		}
	}

	None
}
