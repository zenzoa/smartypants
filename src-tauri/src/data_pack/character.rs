use super::EntityId;
use crate::data_view::DataView;

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

pub fn get_characters(data: &DataView) -> Vec<Character> {

	let mut characters = Vec::new();

	let mut i = 0;
	while i + 96 <= data.len() {
		let id = EntityId::new(data.get_u16(i));
		let character_type = data.get_u16(i + 2);
		let name = data.get_encoded_string(i + 4, 10);
		let profile_image_id = EntityId::new(data.get_u16(i + 24));
		let icon_image_id = EntityId::new(data.get_u16(i + 26));
		let composition_id = EntityId::new(data.get_u16(i + 28));
		let unknown1 = EntityId::new(data.get_u16(i + 30));
		let pronoun = data.get_encoded_string(i + 32, 6);
		let statement = data.get_encoded_string(i + 44, 6);
		let question1 = data.get_encoded_string(i + 56, 6);
		let question2 = data.get_encoded_string(i + 68, 6);
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
