use std::error::Error;
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

pub fn get_images(data: &DataView) -> Result<Vec<ImageDef>, Box<dyn Error>> {
	let mut images = Vec::new();

	let mut i = 0;
	while i + 6 <= data.len() {
		images.push(ImageDef {
			first_sprite_index: data.get_u16(i),
			next_sprite_index: data.get_u16(i),
			width_in_sprites: data.get_u8(i + 2),
			height_in_sprites: data.get_u8(i + 3),
			first_palette_index: data.get_u16(i + 4),
			subimage_count: 0
		});
		i += 6;
	}

	Ok(images)
}

pub fn calc_subimage_counts(image_defs: &mut Vec<ImageDef>, sprite_count: usize) {
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
