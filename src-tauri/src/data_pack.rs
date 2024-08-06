use std::error::Error;

use crate::text::FontState;
use crate::data_view::DataView;

pub mod table1;
pub mod particle_emitter;
pub mod scene;
pub mod tamastring;
pub mod table9;
pub mod item;
pub mod character;
pub mod graphics_node;
pub mod frame;

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
			Some(card_id) => (1 << 15) | ((card_id as u16) << 8) | self.entity_id,
			None => self.entity_id
		}
	}
}

#[derive(Clone, serde::Serialize)]
pub struct DataPack {
	pub table1: Vec<u16>,
	pub particle_emitters: Vec<particle_emitter::ParticleEmitter>,
	pub scenes: Vec<scene::Scene>,
	pub strings: Vec<tamastring::TamaString>,
	pub table9: Vec<Vec<u16>>,
	pub items: Vec<item::Item>,
	pub characters: Vec<character::Character>,
	pub graphics_nodes: Vec<graphics_node::GraphicsNode>,
	pub frame_groups: Vec<frame::FrameGroup>
}

pub fn get_data_pack(font_state: &FontState, data: &DataView) -> Result<DataPack, Box<dyn Error>> {
	let (table_offsets, table_sizes) = get_table_offsets(data)?;

	let get_table_data = |i: usize| -> DataView {
		data.chunk(table_offsets[i], table_sizes[i])
	};

	let table1 = get_table_data(1).to_words();

	let particle_emitters = particle_emitter::get_particle_emitters(&get_table_data(2));

	let (scene_offsets, scene_sizes) = scene::get_scene_offsets(&get_table_data(3));
	let scene_layer_offsets = scene::get_scene_layer_offsets(&get_table_data(4), scene_offsets, scene_sizes);
	let scenes = scene::get_scenes(&get_table_data(5), scene_layer_offsets);

	let strings = tamastring::get_strings(font_state, &get_table_data(6));

	let (table9_offsets, table9_sizes) = table9::get_entity_offsets(&get_table_data(8));
	let table9 = table9::get_entities(&get_table_data(9), table9_offsets, table9_sizes);

	let items = item::get_items(font_state, &get_table_data(10));

	let characters = character::get_characters(font_state, &get_table_data(11));

	let (graphics_nodes_offsets, graphics_nodes_sizes) = graphics_node::get_graphics_nodes_offsets(&get_table_data(13));
	let graphics_nodes = graphics_node::get_graphics_nodes(&get_table_data(14), graphics_nodes_offsets, graphics_nodes_sizes);

	let frame_layers = frame::get_frame_layers(&get_table_data(15));
	let frame_groups = frame::get_frame_groups(&get_table_data(18), frame_layers);

	let data_pack = DataPack {
		table1,
		particle_emitters,
		scenes,
		strings,
		table9,
		items,
		characters,
		graphics_nodes,
		frame_groups
	};

	Ok(data_pack)
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
			table_sizes.push(0);
		}
	}

	Ok((table_offsets, table_sizes))
}

pub fn save_data_pack(data_pack: &DataPack, original_data: &DataView) -> Result<Vec<u8>, Box<dyn Error>> {
	let (table_offsets, table_sizes) = get_table_offsets(original_data)?;

	let mut tables: Vec<Vec<u8>> = Vec::new();
	for i in 0..20 {
		let table_data = original_data.chunk(table_offsets[i], table_sizes[i]).data;
		tables.push(table_data);
	}

	let (string_data, string_offsets) = tamastring::save_strings(&data_pack.strings)?;
	tables[6] = string_data;
	tables[7] = string_offsets;

	tables[10] = item::save_items(&data_pack.items)?;

	tables[11] = character::save_characters(&data_pack.characters)?;

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

	let data = [
			table_offset_data,
			tables[..].concat(),
			// vec![2, 0]
		].concat();

	Ok(data)
}
