use super::EntityId;
use crate::data_view::DataView;

#[derive(Clone, serde::Serialize)]
pub struct Scene {
	pub layers: Vec<SceneLayer>
}

#[derive(Clone, serde::Serialize)]
pub struct SceneLayer {
	pub props: u16,
	pub x: i16,
	pub y: i16,
	pub image_id: EntityId,
	pub subimage_index: u16
}

pub fn get_scene_offsets(data: &DataView) -> (Vec<usize>, Vec<usize>) {
	let mut offsets = Vec::new();
	for i in 0..(data.len()/2) {
		let offset = data.get_u16(i*2) as usize * 2;
		offsets.push(offset);
	}

	let mut sizes = Vec::new();
	for i in 0..(offsets.len() - 1) {
		sizes.push(offsets[i+1] - offsets[i]);
	}

	let _ = offsets.pop();

	(offsets, sizes)
}

pub fn get_scene_layer_offsets(data: &DataView, offsets: Vec<usize>, sizes: Vec<usize>) -> Vec<Vec<usize>> {
	let mut scene_layer_offsets = Vec::new();

	for i in 0..offsets.len() {
		let mut layer_offsets = Vec::new();
		let layer_offset_data = data.chunk(offsets[i], sizes[i]);
		for j in 0..(layer_offset_data.len()/2) {
			let layer_offset = layer_offset_data.get_u16(j*2) * 2;
			layer_offsets.push(layer_offset as usize);
		}
		scene_layer_offsets.push(layer_offsets);
	}

	scene_layer_offsets
}

pub fn get_scenes(data: &DataView, offsets: Vec<Vec<usize>>) -> Vec<Scene> {
	let mut scenes = Vec::new();

	for layer_offsets in offsets {
		let mut layers = Vec::new();

		for i in layer_offsets {
			if i + 8 <= data.len() {
				let props = data.get_u16(i);
				let x = data.get_i16(i + 2);
				let y = data.get_i16(i + 4);
				let image_id = EntityId::new(data.get_u16(i + 6));
				let subimage_index = if props == 0x1047 && i + 1 < data.len() {
					data.get_u16(i + 8)
				} else {
					0
				};
				layers.push(SceneLayer { props, x, y, image_id, subimage_index });
			}
		}

		scenes.push(Scene { layers });
	}

	scenes
}
