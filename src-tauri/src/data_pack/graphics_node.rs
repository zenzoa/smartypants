// use super::EntityId;
use crate::data_view::DataView;

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
