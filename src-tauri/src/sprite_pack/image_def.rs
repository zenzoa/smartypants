use std::error::Error;
use tauri::{ AppHandle, Manager, State };

use crate::DataState;
use crate::data_view::DataView;

#[derive(Clone, serde::Serialize)]
pub struct ImageDef {
	pub first_sprite_index: u16,
	pub next_sprite_index: u16,
	pub width_in_sprites: u8,
	pub height_in_sprites: u8,
	pub first_palette_index: u16,
	pub subimage_count: usize
}

pub fn get_image_defs(data: &DataView) -> Result<Vec<ImageDef>, Box<dyn Error>> {
	let mut image_defs = Vec::new();

	let mut i = 0;
	while i + 6 <= data.len() {
		image_defs.push(ImageDef {
			first_sprite_index: data.get_u16(i),
			next_sprite_index: data.get_u16(i),
			width_in_sprites: data.get_u8(i + 2),
			height_in_sprites: data.get_u8(i + 3),
			first_palette_index: data.get_u16(i + 4),
			subimage_count: 0
		});
		i += 6;
	}

	Ok(image_defs)
}

pub fn calc_subimage_counts(image_defs: &mut [ImageDef], sprite_count: usize) {
	for i in 0..image_defs.len() {
		image_defs[i].next_sprite_index = if i+1 < image_defs.len() {
			image_defs[i+1].first_sprite_index
		} else {
			sprite_count as u16
		};
		let sprite_count = (image_defs[i].next_sprite_index - image_defs[i].first_sprite_index) as usize;
		let sprites_per_subimage = image_defs[i].width_in_sprites as usize * image_defs[i].height_in_sprites as usize;
		image_defs[i].subimage_count = sprite_count / sprites_per_subimage;
	}
}

pub fn save_image_defs(image_defs: &[ImageDef]) -> Result<Vec<u8>, Box<dyn Error>> {
	let mut data: Vec<u8> = Vec::new();

	for image_def in image_defs {
		for bytes in u16::to_le_bytes(image_def.first_sprite_index) {
			data.push(bytes);
		}
		data.push(image_def.width_in_sprites);
		data.push(image_def.height_in_sprites);
		for bytes in u16::to_le_bytes(image_def.first_palette_index) {
			data.push(bytes);
		}
	}

	Ok(data)
}

#[tauri::command]
pub fn update_image_def(handle: AppHandle, data_state: State<DataState>, index: usize, first_palette_index: u16) {
	let mut sprite_pack_opt = data_state.sprite_pack.lock().unwrap();
	if let Some(sprite_pack) = sprite_pack_opt.as_mut() {
		if let Some(image_def) = sprite_pack.image_defs.get_mut(index) {
			image_def.first_palette_index = first_palette_index;
		}
		if let Some(image_def) = sprite_pack.image_defs.get(index) {
			handle.emit("update_image_def", (index, image_def)).unwrap();
		}
	}
}
