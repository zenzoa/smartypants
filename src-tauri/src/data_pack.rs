use std::error::Error;

use crate::data_view::{ DataView, words_to_bytes };

mod particle_emitter;
mod scene;
mod tamastring;
mod table9;
mod item;
mod character;
mod graphics_node;
mod frame;

#[derive(Clone, Debug, serde::Serialize)]
pub struct EntityId {
	pub card_id: Option<u8>,
	pub entity_id: u16
}

impl EntityId {
	pub fn new(word: u16) -> EntityId {
		let is_card = (word >> 15) > 0;
		if is_card {
			EntityId {
				card_id: Some(((word >> 8) & 0x7f) as u8),
				entity_id: word & 0xff
			}
		} else {
			EntityId {
				card_id: None,
				entity_id: word & 0x7fff
			}
		}
	}

	pub fn to_word(&self) -> u16 {
		match self.card_id {
			Some(card_id) => (card_id as u16) << 8 & self.entity_id,
			None => self.entity_id
		}
	}
}

#[derive(Clone, serde::Serialize)]
pub struct DataPack {
	pub particle_emitters: Vec<particle_emitter::ParticleEmitter>,
	pub scenes: Vec<scene::Scene>,
	pub strings: Vec<tamastring::TamaString>,
	pub table9: Vec<Vec<u16>>,
	pub items: Vec<item::Item>,
	pub characters: Vec<character::Character>,
	pub graphics_nodes: Vec<graphics_node::GraphicsNode>,
	pub frame_groups: Vec<frame::FrameGroup>
}

pub fn get_data_pack(data: &DataView) -> Result<DataPack, Box<dyn Error>> {
	let (table_offsets, table_sizes) = get_table_offsets(&data)?;

	let get_table_data = |i: usize| -> DataView {
		data.chunk(table_offsets[i], table_sizes[i])
	};

	let particle_emitters = particle_emitter::get_particle_emitters(&get_table_data(2));

	let (scene_offsets, scene_sizes) = scene::get_scene_offsets(&get_table_data(3));
	let scene_layer_offsets = scene::get_scene_layer_offsets(&get_table_data(4), scene_offsets, scene_sizes);
	let scenes = scene::get_scenes(&get_table_data(5), scene_layer_offsets);

	let strings = tamastring::get_strings(&get_table_data(6));

	let (table9_offsets, table9_sizes) = table9::get_entity_offsets(&get_table_data(8));
	let table9 = table9::get_entities(&get_table_data(9), table9_offsets, table9_sizes);

	let items = item::get_items(&get_table_data(10));

	let characters = character::get_characters(&get_table_data(11));

	let (graphics_nodes_offsets, graphics_nodes_sizes) = graphics_node::get_graphics_nodes_offsets(&get_table_data(13));
	let graphics_nodes = graphics_node::get_graphics_nodes(&get_table_data(14), graphics_nodes_offsets, graphics_nodes_sizes);

	let frame_layers = frame::get_frame_layers(&get_table_data(15));
	let frame_groups = frame::get_frame_groups(&get_table_data(18), frame_layers);

	Ok(DataPack {
		particle_emitters,
		scenes,
		strings,
		table9,
		items,
		characters,
		graphics_nodes,
		frame_groups
	})
}

pub fn get_table_offsets(data: &DataView) -> Result<(Vec<usize>, Vec<usize>), Box<dyn Error>> {
	if data.len() < 80 {
		return Err("Unable to read data table offsets: too short".into());
	}

	let mut table_offsets = Vec::new();
	let mut table_sizes = Vec::new();
	for i in 0..20 {
		let offset = data.get_u32(i*4) as usize * 2;
		table_offsets.push(offset);
	}

	for i in 0..20 {
		if i < 19 {
			if table_offsets[i+1] < table_offsets[i] {
				return Err("Unable to read data table offsets: invalid offsets".into());
			}
			table_sizes.push(table_offsets[i+1] - table_offsets[i]);
		} else {
			if data.len() < table_offsets[i] {
				return Err("Unable to read data table offsets: invalid offsets".into());
			}
			table_sizes.push(data.len() - table_offsets[i]);
		}
	}

	Ok((table_offsets, table_sizes))
}

pub fn save_data_pack(original_data: &[u8], data_pack: &DataPack) -> Result<Vec<u8>, Box<dyn Error>> {
	let data = DataView::new(&original_data);
	let data_pack_data_view = data.chunk(0x6CE000, 0x730000 - 0x6CE000);
	let (table_offsets, table_sizes) = get_table_offsets(&data_pack_data_view)?;

	let mut tables: Vec<Vec<u8>> = Vec::new();
	for i in 0..20 {
		let table_data = data_pack_data_view.chunk(table_offsets[i], table_sizes[i]).data;
		tables.push(table_data);
	}

	let mut string_offsets: Vec<u16> = Vec::new();
	let mut string_data: Vec<u16> = Vec::new();
	for string in &data_pack.strings {
		string_offsets.push(string_data.len() as u16);
		for word in string.to_words() {
			string_data.push(word);
		}
	}
	string_offsets.push(0xFFFF);
	tables[6] = words_to_bytes(&string_data);
	tables[7] = words_to_bytes(&string_offsets);

	let mut table_offset_data: Vec<u8> = Vec::new();
	let mut new_table_offsets: Vec<u32> = Vec::new();
	let mut last_offset: u32 = 80;
	for table in &tables {
		new_table_offsets.push(last_offset);
		for byte in u32::to_le_bytes(last_offset / 2) {
			table_offset_data.push(byte);
		}
		let table_size = table.len() as u32;
		last_offset += table_size;
	}

	let new_data_pack = [
			table_offset_data,
			tables[..].concat()
		].concat();

	let mut new_data = Vec::from(original_data);
	let _: Vec<_> = new_data.splice(0x6CE000..0x730000, new_data_pack).collect();

	Ok(new_data)
}
