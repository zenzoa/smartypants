use std::error::Error;

use serde::{ Serialize, Deserialize };

use tauri::{ AppHandle, Manager, State };

use super::EntityId;
use crate::{ DataState, update_window_title };
use crate::data_view::{ DataView, words_to_bytes, resize_words };
use crate::text::{ Text, FontState };
use crate::file::set_file_modified;

#[derive(Clone, Serialize, Deserialize)]
pub enum CharacterType {
	Unknown,
	Egg,
	Baby,
	Child,
	Teen,
	Adult,
	Npc
}

#[derive(Clone, Serialize, Deserialize)]
pub enum Gender {
	Female,
	Male
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Character {
	pub id: EntityId,
	pub character_type: CharacterType,
	pub name: Text,
	pub profile_image_id: EntityId,
	pub icon_image_id: EntityId,
	pub composition_id: EntityId,
	pub unknown1: EntityId,
	pub pronoun: Text,
	pub statement: Text,
	pub question1: Text,
	pub question2: Text,
	pub unknown2: u16,
	pub unknown3: u16,
	pub global_id: EntityId,
	pub unknown4: u16,
	pub unknown5: u16,
	pub unknown6: u16,
	pub unknown7: u16,
	pub gender: Gender
}

pub fn get_characters(handle: &AppHandle, data: &DataView) -> Vec<Character> {
	let mut characters = Vec::new();

	let mut i = 0;
	while i + 96 <= data.len() {
		let id = EntityId::new(data.get_u16(i));
		let character_type = match data.get_u16(i + 2) {
			1 => CharacterType::Egg,
			2 => CharacterType::Baby,
			3 => CharacterType::Child,
			4 => CharacterType::Teen,
			5 => CharacterType::Adult,
			6 => CharacterType::Npc,
			_ => CharacterType::Unknown,
		};
		let name = data.get_text(handle, i + 4, 10);
		let profile_image_id = EntityId::new(data.get_u16(i + 24));
		let icon_image_id = EntityId::new(data.get_u16(i + 26));
		let composition_id = EntityId::new(data.get_u16(i + 28));
		let unknown1 = EntityId::new(data.get_u16(i + 30));
		let pronoun = data.get_text(handle, i + 32, 6);
		let statement = data.get_text(handle, i + 44, 6);
		let question1 = data.get_text(handle, i + 56, 6);
		let question2 = data.get_text(handle, i + 68, 6);
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

pub fn save_characters(characters: &[Character]) -> Result<Vec<u8>, Box<dyn Error>> {
	let mut words: Vec<u16> = Vec::new();

	for character in characters {
		words.push(character.id.to_word());
		words.push(match character.character_type {
			CharacterType::Unknown => 0,
			CharacterType::Egg => 1,
			CharacterType::Baby => 2,
			CharacterType::Child => 3,
			CharacterType::Teen => 4,
			CharacterType::Adult => 5,
			CharacterType::Npc => 6
		});
		words = [words, resize_words(&character.name.data, 10)].concat();
		words.push(character.profile_image_id.to_word());
		words.push(character.icon_image_id.to_word());
		words.push(character.composition_id.to_word());
		words.push(character.unknown1.to_word());
		words = [words, resize_words(&character.pronoun.data, 6)].concat();
		words = [words, resize_words(&character.statement.data, 6)].concat();
		words = [words, resize_words(&character.question1.data, 6)].concat();
		words = [words, resize_words(&character.question2.data, 6)].concat();
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
pub fn update_character(handle: AppHandle, index: usize, new_character: Character) -> Option<Character> {
	let data_state: State<DataState> = handle.state();
	let font_state: State<FontState> = handle.state();
	let char_codes = &font_state.char_codes.lock().unwrap();

	let mut data_pack_opt = data_state.data_pack.lock().unwrap();
	if let Some(data_pack) = data_pack_opt.as_mut() {
		if let Some(character) = data_pack.characters.get_mut(index) {
			character.name.set_string(char_codes, &new_character.name.string);
			character.profile_image_id = new_character.profile_image_id;
			character.icon_image_id = new_character.icon_image_id;
			character.composition_id = new_character.composition_id;
			character.pronoun.set_string(char_codes, &new_character.pronoun.string);
			character.statement.set_string(char_codes, &new_character.statement.string);
			character.question1.set_string(char_codes, &new_character.question1.string);
			character.question2.set_string(char_codes, &new_character.question2.string);
			character.unknown2 = new_character.unknown2;
			character.unknown3 = new_character.unknown3;
			character.unknown4 = new_character.unknown4;
			character.unknown5 = new_character.unknown5;
			character.unknown6 = new_character.unknown6;
			character.unknown7 = new_character.unknown7;
			character.gender = new_character.gender;

			set_file_modified(&handle, true);
			update_window_title(&handle);
			return Some(character.clone());
		}
	}

	None
}
