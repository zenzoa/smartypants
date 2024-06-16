use super::EntityId;
use crate::data_view::{ DataView, get_encoded_char };

#[derive(Clone, serde::Serialize)]
pub struct TamaString {
	pub id: EntityId,
	pub unknown1: u16,
	pub unknown2: u16,
	pub unknown3: u16,
	pub value: String
}

pub fn get_strings(data: &DataView) -> Vec<TamaString> {
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
			s.push_str(&get_encoded_char(c));
			str_len += 1;
		}
		i += 10 + str_len*2;
		strings.push(TamaString {id, unknown1, unknown2, unknown3, value: s });
	}

	strings
}
