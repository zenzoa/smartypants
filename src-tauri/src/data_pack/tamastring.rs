use std::error::Error;

use super::EntityId;
use crate::data_view::{ DataView, words_to_bytes };
use crate::text::{ Text, FontState };

#[derive(Clone, serde::Serialize)]
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
}

pub fn get_strings(font_state: &FontState, data: &DataView) -> Vec<TamaString> {
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
			value: Text::from_data(font_state, &text_data)
		});
	}

	strings
}

pub fn save_strings(strings: &[TamaString]) -> Result<(Vec<u8>, Vec<u8>), Box<dyn Error>> {
	let mut words: Vec<u16> = Vec::new();
	let mut offsets: Vec<u16> = Vec::new();

	for string in strings {
		offsets.push(words.len() as u16);
		words.push(string.id.to_word());
		words.push(string.unknown1);
		words.push(string.unknown2);
		words.push(string.unknown3);
		words = [words, string.value.data.clone()].concat();
		words.push(0);
	}

	offsets.push(0xFFFF);

	Ok((words_to_bytes(&words), words_to_bytes(&offsets)))
}
