use std::error::Error;
use tauri::{ AppHandle, Manager, State };

use super::EntityId;
use crate::DataState;
use crate::data_view::{ DataView, words_to_bytes };
use crate::text::{ FontState, encode_string_with_length };

#[derive(Clone, serde::Serialize)]
pub struct Item {
	pub id: EntityId,
	pub item_type: ItemType,
	pub name: String,
	pub image_id: EntityId,
	pub worn_image_id: Option<EntityId>,
	pub close_image_id: Option<EntityId>,
	pub unknown1: u16,
	pub price: u16,
	pub unknown2: u16,
	pub unknown3: u16,
	pub unknown4: u16,
	pub unlocked_character: Option<u16>,
	pub game_type: Option<GameType>
}

#[derive(Clone, PartialEq, serde::Serialize)]
pub enum ItemType {
	Unknown,
	Meal,
	Snack,
	Toy,
	AccessoryHead,
	AccessoryFace,
	AccessoryBody,
	AccessoryHand,
	Room,
	Game
}

#[derive(Clone, serde::Serialize)]
pub enum GameType {
	Unknown,
	GuessingGame,
	TimingGame,
	MemoryGame,
	DodgingGame,
	ShakingGame,
	SwipingGame
}

pub fn get_items(font_state: &FontState, data: &DataView) -> Vec<Item> {
	let mut items = Vec::new();

	let mut i = 0;
	while i + 42 <= data.len() {
		let id = EntityId::new(data.get_u16(i));
		let item_type = match data.get_u16(i + 2) {
			0 => ItemType::Meal,
			1 => ItemType::Snack,
			2 => ItemType::Toy,
			3 => ItemType::AccessoryHead,
			4 => ItemType::AccessoryFace,
			5 => ItemType::AccessoryBody,
			6 => ItemType::AccessoryHand,
			7 => ItemType::Room,
			8 => ItemType::Game,
			_ => ItemType::Unknown
		};
		let name = data.get_encoded_string(font_state, i + 4, 10);
		let image_id = EntityId::new(data.get_u16(i + 24));
		let worn_image_id = if data.get_u16(i + 26) > 0 {
			Some(EntityId::new(data.get_u16(i + 26)))
		} else {
			None
		};
		let close_image_id = if data.get_u16(i + 28) > 0 {
			Some(EntityId::new(data.get_u16(i + 28)))
		} else {
			None
		};
		let unknown1 = data.get_u16(i + 30);
		let price = data.get_u16(i + 32);
		let unknown2 = data.get_u16(i + 34);
		let unknown3 = data.get_u16(i + 36);
		let unknown4 = data.get_u16(i + 38);
		let unlocked_character = if item_type == ItemType::Game || data.get_u16(i + 40) == 0 {
			None
		} else {
			Some(data.get_u16(i + 40))
		};
		let game_type = if item_type == ItemType::Game {
			Some(match data.get_u16(i + 40) {
				10 => GameType::GuessingGame,
				11 => GameType::TimingGame,
				12 => GameType::MemoryGame,
				13 => GameType::DodgingGame,
				14 => GameType::ShakingGame,
				15 => GameType::SwipingGame,
				_ => GameType::Unknown
			})
		} else {
			None
		};

		items.push(Item {
			id,
			item_type,
			name,
			image_id,
			worn_image_id,
			close_image_id,
			unknown1,
			price,
			unknown2,
			unknown3,
			unknown4,
			unlocked_character,
			game_type
		});

		i += 42;
	}

	items
}

pub fn save_items(items: &[Item], font_state: State<FontState>) -> Result<Vec<u8>, Box<dyn Error>> {
	let mut words: Vec<u16> = Vec::new();

	for item in items {
		words.push(item.id.to_word());
		words.push(match item.item_type {
			ItemType::Meal => 0,
			ItemType::Snack => 1,
			ItemType::Toy => 2,
			ItemType::AccessoryHead => 3,
			ItemType::AccessoryFace => 4,
			ItemType::AccessoryBody => 5,
			ItemType::AccessoryHand => 6,
			ItemType::Room => 7,
			ItemType::Game => 8,
			ItemType::Unknown => 9,
		});
		words = [words, encode_string_with_length(font_state.clone(), &item.name, 10)].concat();
		words.push(item.image_id.to_word());
		words.push(match &item.worn_image_id {
			Some(id) => id.to_word(),
			None => 0
		});
		words.push(match &item.close_image_id {
			Some(id) => id.to_word(),
			None => 0
		});
		words.push(item.unknown1);
		words.push(item.price);
		words.push(item.unknown2);
		words.push(item.unknown3);
		words.push(item.unknown4);
		words.push(match item.item_type {
			ItemType::Game => match &item.game_type {
				Some(game_type) => match game_type {
					GameType::GuessingGame => 10,
					GameType::TimingGame => 11,
					GameType::MemoryGame => 12,
					GameType::DodgingGame => 13,
					GameType::ShakingGame => 14,
					GameType::SwipingGame => 15,
					GameType::Unknown => 0
				},
				None => 0
			},
			_ => item.unlocked_character.unwrap_or(0)
		});
	}

	Ok(words_to_bytes(&words))
}

#[tauri::command]
pub fn update_item(handle: AppHandle, data_state: State<DataState>, index: usize, name: String) {
	let mut data_pack_opt = data_state.data_pack.lock().unwrap();
	if let Some(data_pack) = data_pack_opt.as_mut() {
		if let Some(item) = data_pack.items.get_mut(index) {
			item.name = name;
		}
		if let Some(item) = data_pack.items.get(index) {
			handle.emit("update_item", (index, item)).unwrap();
		}
	}
}
