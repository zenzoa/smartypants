use std::error::Error;

use crate::data_view::{ DataView, words_to_bytes };

#[derive(Clone, serde::Serialize)]
pub struct GraphicsNode {
	pub data: Vec<u16>
}

pub fn get_graphics_nodes_offsets(data: &DataView) -> (Vec<usize>, Vec<usize>) {
	let mut offsets = Vec::new();
	for i in 0..(data.len()/2) {
		let offset = data.get_u16(i*2) as usize * 4;
		offsets.push(offset);
	}

	let mut sizes = Vec::new();
	for i in 0..(offsets.len() - 1) {
		sizes.push(offsets[i+1] - offsets[i]);
	}

	let _ = offsets.pop();

	(offsets, sizes)
}

pub fn get_graphics_nodes(data: &DataView, offsets: Vec<usize>, sizes: Vec<usize>) -> Vec<GraphicsNode> {
	let mut graphics_nodes = Vec::new();

	for i in 0..offsets.len() {
		let graphics_node_data = data.chunk(offsets[i], sizes[i]);
		let mut data_u16s = Vec::new();
		for j in 0..(graphics_node_data.len()/2) {
			data_u16s.push(graphics_node_data.get_u16(j*2));
		}
		graphics_nodes.push(GraphicsNode { data: data_u16s })
	}

	graphics_nodes
}

pub fn save_graphics_nodes(graphics_nodes: &[GraphicsNode]) -> Result<(Vec<u8>, Vec<u8>), Box<dyn Error>> {
	let mut offsets = Vec::new();
	let mut data = Vec::new();

	for graphics_node in graphics_nodes {
		let offset = data.len() / 4;
		offsets.extend_from_slice(&(offset as u16).to_le_bytes());
		data.extend_from_slice(&words_to_bytes(&graphics_node.data));
	}

	offsets.extend_from_slice(&(data.len() as u16 / 4).to_le_bytes());

	Ok((offsets, data))
}
