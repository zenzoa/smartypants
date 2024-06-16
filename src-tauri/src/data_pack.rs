use std::error::Error;

use crate::data_view::DataView;

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
}

#[derive(Clone, serde::Serialize)]
pub struct DataPack {
	particle_emitters: Vec<particle_emitter::ParticleEmitter>,
	scenes: Vec<scene::Scene>,
	strings: Vec<tamastring::TamaString>,
	table9: Vec<Vec<u16>>,
	items: Vec<item::Item>,
	characters: Vec<character::Character>,
	graphics_nodes: Vec<graphics_node::GraphicsNode>,
	frame_groups: Vec<frame::FrameGroup>
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

fn get_table_offsets(data: &DataView) -> Result<(Vec<usize>, Vec<usize>), Box<dyn Error>> {
	if data.len() < 80 {
		return Err("too short".into());
	}

	let mut table_offsets = Vec::new();
	let mut table_sizes = Vec::new();
	for i in 0..20 {
		let offset = data.get_u32(i*4) as usize * 2;
		table_offsets.push(offset);
	}

	for i in 0..20 {
		if i < 19 {
			table_sizes.push(table_offsets[i+1] - table_offsets[i]);
		} else {
			table_sizes.push(data.len() - table_offsets[i]);
		}
	}

	Ok((table_offsets, table_sizes))
}
