use tauri::State;

use super::EntityId;
use crate::data_view::DataView;
use crate::text::{ FontState, encode_string, word_to_char_code };

#[derive(Clone, serde::Serialize)]
pub struct TamaString {
	pub id: EntityId,
	pub unknown1: u16,
	pub unknown2: u16,
	pub unknown3: u16,
	pub value: String
}

impl TamaString {
	pub fn to_words(&self, font_state: State<FontState>) -> Vec<u16> {
		let mut data: Vec<u16> = vec![
			self.id.to_word(),
			self.unknown1,
			self.unknown2,
			self.unknown3
		];
		for word in encode_string(font_state, &self.value) {
			data.push(word);
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

		let mut s = String::new();
		let mut str_len = 0;

		while i + 8 + str_len*2 <= data.len() && data.get_u16(i + 8 + str_len*2) > 0 {
			let c = data.get_u16(i + 8 + str_len*2);
			if let Some(char_code) = word_to_char_code(font_state, c) {
				s.push_str(&char_code);
			}
			str_len += 1;
		}
		i += 10 + str_len*2;

		strings.push(TamaString {id, unknown1, unknown2, unknown3, value: s });
	}

	strings
}
