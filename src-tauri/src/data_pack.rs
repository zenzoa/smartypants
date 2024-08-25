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
	pub table1: Vec<Vec<u16>>,
	pub particle_emitters: Vec<particle_emitter::ParticleEmitter>,
	pub scenes: Vec<scene::Scene>,
	pub tamastrings: Vec<tamastring::TamaString>,
	pub table9: Vec<Vec<u16>>,
	pub items: Vec<item::Item>,
	pub characters: Vec<character::Character>,
	pub graphics_nodes: Vec<graphics_node::GraphicsNode>,
	pub frame_groups: Vec<frame::FrameGroup>,
	pub card_id: u16
}

pub fn get_data_pack(font_state: &FontState, data: &DataView) -> Result<DataPack, Box<dyn Error>> {
	let (table_offsets, table_sizes) = get_table_offsets(data)?;

	let get_table_data = |i: usize| -> DataView {
		data.chunk(table_offsets[i], table_sizes[i])
	};

	let table1_offsets = table1::get_entity_offsets(&get_table_data(0));
	let table1 = table1::get_entities(&get_table_data(1), table1_offsets);

	let particle_emitters = particle_emitter::get_particle_emitters(&get_table_data(2));

	let (scene_offsets, scene_sizes) = scene::get_scene_offsets(&get_table_data(3));
	let scene_layer_offsets = scene::get_scene_layer_offsets(&get_table_data(4), scene_offsets, scene_sizes);
	let scenes = scene::get_scenes(&get_table_data(5), scene_layer_offsets);

	let tamastrings = tamastring::get_tamastrings(font_state, &get_table_data(6));

	let (table9_offsets, table9_sizes) = table9::get_entity_offsets(&get_table_data(8));
	let table9 = table9::get_entities(&get_table_data(9), table9_offsets, table9_sizes);

	let items = item::get_items(font_state, &get_table_data(10));

	let characters = character::get_characters(font_state, &get_table_data(11));

	let graphics_nodes_offsets = graphics_node::get_graphics_nodes_offsets(&get_table_data(13));
	let graphics_nodes = graphics_node::get_graphics_nodes(&get_table_data(14), graphics_nodes_offsets);

	let frame_layers = frame::get_frame_layers(&get_table_data(15));
	let frame_groups = frame::get_frame_groups(&get_table_data(18), frame_layers);

	let card_id = get_table_data(19).get_u16(0);

	let data_pack = DataPack {
		table1,
		particle_emitters,
		scenes,
		tamastrings,
		table9,
		items,
		characters,
		graphics_nodes,
		frame_groups,
		card_id
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
			table_sizes.push(2);
		}
	}

	Ok((table_offsets, table_sizes))
}

pub fn save_data_pack(data_pack: &DataPack, offset: usize) -> Result<Vec<u8>, Box<dyn Error>> {
	let mut tables: Vec<Vec<u8>> = vec![vec![]; 20];

	let (table1_offsets, table1_data) = table1::save_entities(&data_pack.table1)?;
	tables[0] = table1_offsets;
	tables[1] = table1_data;

	tables[2] = particle_emitter::save_particle_emitters(&data_pack.particle_emitters)?;

	let (scene_offsets, scene_layer_offsets, scene_data) = scene::save_scenes(&data_pack.scenes)?;
	tables[3] = scene_offsets;
	tables[4] = scene_layer_offsets;
	tables[5] = scene_data;

	let (string_offsets, string_data) = tamastring::save_tamastrings(&data_pack.tamastrings)?;
	tables[6] = string_data;
	tables[7] = string_offsets;

	let (table9_offsets, table9_data) = table9::save_entities(&data_pack.table9)?;
	tables[8] = table9_offsets;
	tables[9] = table9_data;

	tables[10] = item::save_items(&data_pack.items)?;

	tables[11] = character::save_characters(&data_pack.characters)?;

	tables[12] = Vec::new();

	let (graphics_node_offsets, graphics_node_data) = graphics_node::save_graphics_nodes(&data_pack.graphics_nodes)?;
	tables[13] = graphics_node_offsets;
	tables[14] = graphics_node_data;

	let (frame_layer_offsets, frame_layer_data, frame_group_data) = frame::save_frame_groups(&data_pack.frame_groups)?;
	tables[15] = frame_layer_data;
	tables[16] = frame_layer_offsets;
	tables[18] = frame_group_data;

	tables[17] = Vec::new();

	tables[19] = data_pack.card_id.to_le_bytes().to_vec();

	let mut offsets = Vec::new();
	let mut data = vec![0; 80];
	for (i, table) in tables.iter().enumerate() {
		offsets.extend_from_slice(&(data.len() as u32 / 2).to_le_bytes());
		if i == 12 || i == 17 {
			let mut next_table_offset = data.len();
			while (offset + next_table_offset) % 8 != 0 {
				next_table_offset += 1;
			}
			let padding_size = next_table_offset - data.len();
			data.extend_from_slice(&vec![0; padding_size]);
		} else {
			data.extend_from_slice(&table);
		}
	}

	data.splice(0..80, offsets);

	Ok(data)
}
