use std::error::Error;

use serde::{ Serialize, Deserialize };

use tauri::{ AppHandle, Manager, State };

use super::EntityId;
use crate::{ DataState, update_window_title };
use crate::data_view::DataView;
use crate::file::set_file_modified;

#[derive(Clone, Serialize, Deserialize)]
pub struct Scene {
	pub layers: Vec<SceneLayer>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneLayer {
	pub x: i16,
	pub y: i16,
	pub image_id: Option<EntityId>,
	pub subimage_index: u16,
	pub unknown1: i16,
	pub unknown2: u16,
	pub unknown3: u16,
	pub unknown4: u16,
	pub unknown5: u16,
	pub unknown6: u16,
	pub unknown7: u16,
	pub unknown8: u16,
	pub flag1: bool,
	pub flag2: bool,
	pub flag3: bool,
	pub flag4: bool
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
			if i + 2 < data.len() {
				let mut local_i = i;

				let bitmask = data.get_u16(i);
				let mut flags = [false; 16];
				for (i, flag) in flags.iter_mut().enumerate() {
					*flag = bitmask & (1 << i) != 0;
				}

				let x = if flags[0] && local_i + 2 < data.len() {
					local_i += 2;
					data.get_i16(local_i)
				} else {
					0
				};

				let y = if flags[1] && local_i + 2 < data.len() {
					local_i += 2;
					data.get_i16(local_i)
				} else {
					0
				};

				let image_id = if flags[2] && local_i + 2 < data.len() {
					local_i += 2;
					Some(EntityId::new(data.get_u16(local_i)))
				} else {
					None
				};

				let unknown1 = if flags[3] && local_i + 2 < data.len() {
					local_i += 2;
					data.get_i16(local_i)
				} else {
					0
				};

				let unknown2 = if flags[4] && local_i + 2 < data.len() {
					local_i += 2;
					data.get_u16(local_i)
				} else {
					0
				};

				let unknown3 = if flags[5] && local_i + 2 < data.len() {
					local_i += 2;
					data.get_u16(local_i)
				} else {
					0
				};

				let subimage_index = if flags[6] && local_i + 2 < data.len() {
					local_i += 2;
					data.get_u16(local_i)
				} else {
					0
				};

				let unknown4 = if flags[7] && local_i + 2 < data.len() {
					local_i += 2;
					data.get_u16(local_i)
				} else {
					0
				};

				let unknown5 = if flags[8] && local_i + 2 < data.len() {
					local_i += 2;
					data.get_u16(local_i)
				} else {
					0
				};

				let unknown6 = if flags[9] && local_i + 2 < data.len() {
					local_i += 2;
					data.get_u16(local_i)
				} else {
					0
				};

				let unknown7 = if flags[10] && local_i + 2 < data.len() {
					local_i += 2;
					data.get_u16(local_i)
				} else {
					0
				};

				let unknown8 = if flags[11] && local_i + 2 < data.len() {
					local_i += 2;
					data.get_u16(local_i)
				} else {
					0
				};

				let flag1 = flags[12];

				let flag2 = flags[13];

				let flag3 = flags[14];

				let flag4 = flags[15];

				layers.push(SceneLayer {
					x,
					y,
					image_id,
					subimage_index,
					unknown1,
					unknown2,
					unknown3,
					unknown4,
					unknown5,
					unknown6,
					unknown7,
					unknown8,
					flag1,
					flag2,
					flag3,
					flag4
				});
			}
		}

		scenes.push(Scene { layers });
	}

	scenes
}

pub fn save_scenes(scenes: &[Scene]) -> Result<(Vec<u8>, Vec<u8>, Vec<u8>), Box<dyn Error>> {
	let mut scene_offsets = Vec::new();
	let mut layer_offsets = Vec::new();
	let mut layer_data = Vec::new();

	for scene in scenes {

		scene_offsets.extend_from_slice(&(layer_offsets.len() as u16 / 2).to_le_bytes());
		for layer in &scene.layers {
			let mut bitmask: u16 = 0;
			let mut this_layer_data = vec![0, 0];
			layer_offsets.extend_from_slice(&(layer_data.len() as u16 / 2).to_le_bytes());

			if layer.x != 0 {
				bitmask |= 1 << 0;
				this_layer_data.extend_from_slice(&layer.x.to_le_bytes());
			}

			if layer.y != 0 {
				bitmask |= 1 << 1;
				this_layer_data.extend_from_slice(&layer.y.to_le_bytes());
			}

			if let Some(image_id) = &layer.image_id {
				bitmask |= 1 << 2;
				this_layer_data.extend_from_slice(&image_id.to_word().to_le_bytes());
			}

			if layer.unknown1 != 0 {
				bitmask |= 1 << 3;
				this_layer_data.extend_from_slice(&layer.unknown1.to_le_bytes());
			}

			if layer.unknown2 != 0 {
				bitmask |= 1 << 4;
				this_layer_data.extend_from_slice(&layer.unknown2.to_le_bytes());
			}

			if layer.unknown3 != 0 {
				bitmask |= 1 << 5;
				this_layer_data.extend_from_slice(&layer.unknown3.to_le_bytes());
			}

			if layer.subimage_index != 0 {
				bitmask |= 1 << 6;
				this_layer_data.extend_from_slice(&layer.subimage_index.to_le_bytes());
			}

			if layer.unknown4 != 0 {
				bitmask |= 1 << 7;
				this_layer_data.extend_from_slice(&layer.unknown4.to_le_bytes());
			}

			if layer.unknown5 != 0 {
				bitmask |= 1 << 8;
				this_layer_data.extend_from_slice(&layer.unknown5.to_le_bytes());
			}

			if layer.unknown6 != 0 {
				bitmask |= 1 << 9;
				this_layer_data.extend_from_slice(&layer.unknown6.to_le_bytes());
			}

			if layer.unknown7 != 0 {
				bitmask |= 1 << 10;
				this_layer_data.extend_from_slice(&layer.unknown7.to_le_bytes());
			}

			if layer.unknown8 != 0 {
				bitmask |= 1 << 11;
				this_layer_data.extend_from_slice(&layer.unknown8.to_le_bytes());
			}

			if layer.flag1 {
				bitmask |= 1 << 12;
			}

			if layer.flag2 {
				bitmask |= 1 << 13;
			}

			if layer.flag3 {
				bitmask |= 1 << 14;
			}

			if layer.flag4 {
				bitmask |= 1 << 15;
			}

			this_layer_data.splice(0..2, bitmask.to_le_bytes().into_iter());
			layer_data.extend_from_slice(&this_layer_data);
		}

		layer_data.extend_from_slice(&[0, 0]);
	}

	scene_offsets.extend_from_slice(&(layer_offsets.len() as u16 / 2).to_le_bytes());

	Ok((scene_offsets, layer_offsets, layer_data))
}

#[tauri::command]
pub fn update_scene_layer(handle: AppHandle, scene_index: usize, layer_index: usize, new_layer: SceneLayer) -> Option<SceneLayer> {
	let data_state: State<DataState> = handle.state();

	let mut data_pack_opt = data_state.data_pack.lock().unwrap();
	if let Some(data_pack) = data_pack_opt.as_mut() {
		if let Some(scene) = data_pack.scenes.get_mut(scene_index) {
			if let Some(layer) = scene.layers.get_mut(layer_index) {
				*layer = new_layer;
				set_file_modified(&handle, true);
				update_window_title(&handle);
				return Some(layer.clone());
			}
		}
	}

	None
}
