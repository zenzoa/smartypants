use super::EntityId;
use crate::data_view::DataView;

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

pub fn get_items(data: &DataView) -> Vec<Item> {
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
		let name = data.get_encoded_string(i + 4, 10);
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
		let unlocked_character = if item_type == ItemType::Game {
			None
		} else if data.get_u16(i + 40) == 0 {
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
