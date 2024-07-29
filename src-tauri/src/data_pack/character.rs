use std::error::Error;
use tauri::{ AppHandle, Manager, State };

use super::EntityId;
use crate::DataState;
use crate::data_view::{ DataView, words_to_bytes };
use crate::text::{ FontState, encode_string_with_length };

#[derive(Clone, serde::Serialize)]
pub enum Gender {
	Female,
	Male
}

#[derive(Clone, serde::Serialize)]
pub struct Character {
	pub id: EntityId,
	pub character_type: u16,
	pub name: String,
	pub profile_image_id: EntityId,
	pub icon_image_id: EntityId,
	pub composition_id: EntityId,
	pub unknown1: EntityId,
	pub pronoun: String,
	pub statement: String,
	pub question1: String,
	pub question2: String,
	pub unknown2: u16,
	pub unknown3: u16,
	pub global_id: EntityId,
	pub unknown4: u16,
	pub unknown5: u16,
	pub unknown6: u16,
	pub unknown7: u16,
	pub gender: Gender
}

pub fn get_characters(font_state: &FontState, data: &DataView) -> Vec<Character> {

	let mut characters = Vec::new();

	let mut i = 0;
	while i + 96 <= data.len() {
		let id = EntityId::new(data.get_u16(i));
		let character_type = data.get_u16(i + 2);
		let name = data.get_encoded_string(font_state, i + 4, 10);
		let profile_image_id = EntityId::new(data.get_u16(i + 24));
		let icon_image_id = EntityId::new(data.get_u16(i + 26));
		let composition_id = EntityId::new(data.get_u16(i + 28));
		let unknown1 = EntityId::new(data.get_u16(i + 30));
		let pronoun = data.get_encoded_string(font_state, i + 32, 6);
		let statement = data.get_encoded_string(font_state, i + 44, 6);
		let question1 = data.get_encoded_string(font_state, i + 56, 6);
		let question2 = data.get_encoded_string(font_state, i + 68, 6);
		let unknown2 = data.get_u16(i + 80);
		let unknown3 = data.get_u16(i + 82);
		let global_id = EntityId::new(data.get_u16(i + 84));
		let unknown4 = data.get_u16(i + 86);
		let unknown5 = data.get_u16(i + 88);
		let unknown6 = data.get_u16(i + 90);
		let unknown7 = data.get_u16(i + 92);
		let gender = match data.get_u16(i + 94) {
			0 => Gender::Female,
			_ => Gender::Male
		};

		characters.push(Character {
			id,
			character_type,
			name,
			profile_image_id,
			icon_image_id,
			composition_id,
			unknown1,
			pronoun,
			statement,
			question1,
			question2,
			unknown2,
			unknown3,
			global_id,
			unknown4,
			unknown5,
			unknown6,
			unknown7,
			gender
		});

		i += 96;
	}

	characters
}

pub fn save_characters(characters: &[Character], font_state: State<FontState>) -> Result<Vec<u8>, Box<dyn Error>> {
	let mut words: Vec<u16> = Vec::new();

	for character in characters {
		words.push(character.id.to_word());
		words.push(character.character_type);
		words = [words, encode_string_with_length(font_state.clone(), &character.name, 10)].concat();
		words.push(character.profile_image_id.to_word());
		words.push(character.icon_image_id.to_word());
		words.push(character.composition_id.to_word());
		words.push(character.unknown1.to_word());
		words = [words, encode_string_with_length(font_state.clone(), &character.pronoun, 6)].concat();
		words = [words, encode_string_with_length(font_state.clone(), &character.statement, 6)].concat();
		words = [words, encode_string_with_length(font_state.clone(), &character.question1, 6)].concat();
		words = [words, encode_string_with_length(font_state.clone(), &character.question2, 6)].concat();
		words.push(character.unknown2);
		words.push(character.unknown3);
		words.push(character.global_id.to_word());
		words.push(character.unknown4);
		words.push(character.unknown5);
		words.push(character.unknown6);
		words.push(character.unknown7);
		words.push(match character.gender {
			Gender::Female => 0,
			Gender::Male => 1
		});
	}

	Ok(words_to_bytes(&words))
}

#[tauri::command]
pub fn update_character(handle: AppHandle, data_state: State<DataState>, index: usize, name: String) {
	let mut data_pack_opt = data_state.data_pack.lock().unwrap();
	if let Some(data_pack) = data_pack_opt.as_mut() {
		if let Some(character) = data_pack.characters.get_mut(index) {
			character.name = name;
		}
		if let Some(character) = data_pack.characters.get(index) {
			handle.emit("update_character", (index, character)).unwrap();
		}
	}
}
