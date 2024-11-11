use std::error::Error;

use serde::{ Serialize, Deserialize };

use tauri::{ AppHandle, Manager, State };

use super::EntityId;
use crate::{ DataState, update_window_title };
use crate::data_view::{ DataView, words_to_bytes, resize_words };
use crate::text::{ Text, FontState };
use crate::file::set_file_modified;

#[derive(Clone, Serialize, Deserialize)]
pub struct Item {
	pub id: EntityId,
	pub item_type: ItemType,
	pub name: Text,
	pub image_id: Option<EntityId>,
	pub worn_image_id: Option<EntityId>,
	pub close_image_id: Option<EntityId>,
	pub animation_id: Option<EntityId>,
	pub price: u16,
	pub unknown1: u16,
	pub unknown2: u16,
	pub unknown3: u16,
	pub unlocked_character: Option<u16>,
	pub game_type: Option<GameType>
}

impl Item {
	pub fn set_card_id(&mut self, old_card_id: u8, new_card_id: u8) {
		self.id.set_card_id(old_card_id, new_card_id);
		if let Some(image_id) = &mut self.image_id {
			image_id.set_card_id(old_card_id, new_card_id);
		}
		if let Some(worn_image_id) = &mut self.worn_image_id {
			worn_image_id.set_card_id(old_card_id, new_card_id);
		}
		if let Some(close_image_id) = &mut self.close_image_id {
			close_image_id.set_card_id(old_card_id, new_card_id);
		}
		if let Some(animation_id) = &mut self.animation_id {
			animation_id.set_card_id(old_card_id, new_card_id);
		}
	}
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
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

impl ItemType {
	pub fn is_accessory(&self) -> bool {
		*self == ItemType::AccessoryHead || *self == ItemType::AccessoryFace || *self == ItemType::AccessoryBody || *self == ItemType::AccessoryHand
	}
}

#[derive(Clone, Serialize, Deserialize)]
pub enum GameType {
	Unknown,
	GuessingGame,
	TimingGame,
	MemoryGame,
	DodgingGame,
	ShakingGame,
	SwipingGame
}

pub fn get_items(handle: &AppHandle, data: &DataView) -> Vec<Item> {
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
		let name = data.get_text(handle, i + 4, 10);
		let image_id = if data.get_u16(i + 24) > 0 {
			Some(EntityId::new(data.get_u16(i + 24)))
		} else {
			None
		};
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
		let animation_id = if data.get_u16(i + 30) > 0 {
			Some(EntityId::new(data.get_u16(i + 30)))
		} else {
			None
		};
		let price = data.get_u16(i + 32);
		let unknown1 = data.get_u16(i + 34);
		let unknown2 = data.get_u16(i + 36);
		let unknown3 = data.get_u16(i + 38);
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
			animation_id,
			price,
			unknown1,
			unknown2,
			unknown3,
			unlocked_character,
			game_type
		});

		i += 42;
	}

	items
}

pub fn save_items(items: &[Item]) -> Result<Vec<u8>, Box<dyn Error>> {
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
		words = [words, resize_words(&item.name.data, 10)].concat();
		words.push(match &item.image_id {
			Some(id) => id.to_word(),
			None => 0
		});
		words.push(match &item.worn_image_id {
			Some(id) => id.to_word(),
			None => 0
		});
		words.push(match &item.close_image_id {
			Some(id) => id.to_word(),
			None => 0
		});
		words.push(match &item.animation_id {
			Some(id) => id.to_word(),
			None => 0
		});
		words.push(item.price);
		words.push(item.unknown1);
		words.push(item.unknown2);
		words.push(item.unknown3);
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
pub fn update_item(handle: AppHandle, index: usize, new_item: Item) -> Option<Item> {
	let data_state: State<DataState> = handle.state();
	let font_state: State<FontState> = handle.state();
	let char_codes = &font_state.char_codes.lock().unwrap();

	let mut data_pack_opt = data_state.data_pack.lock().unwrap();
	if let Some(data_pack) = data_pack_opt.as_mut() {
		if let Some(item) = data_pack.items.get_mut(index) {
			item.item_type = new_item.item_type;
			item.name.set_string(char_codes, &new_item.name.string);
			item.image_id = new_item.image_id;
			item.worn_image_id = new_item.worn_image_id;
			item.close_image_id = new_item.close_image_id;
			item.animation_id = new_item.animation_id;
			item.price = new_item.price;
			item.unknown1 = new_item.unknown1;
			item.unknown2 = new_item.unknown2;
			item.unknown3 = new_item.unknown3;
			item.game_type = new_item.game_type;
			item.unlocked_character = new_item.unlocked_character;

			set_file_modified(&handle, true);
			update_window_title(&handle);
			return Some(item.clone());
		}
	}

	None
}
