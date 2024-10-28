use std::error::Error;

use image::{ RgbaImage, GenericImage };
use serde::{ Serialize, Deserialize };
use tauri::{ AppHandle, Manager, State };

use super::Sprite;
use super::palette::Color;
use crate::{ DataState, update_window_title };
use crate::data_view::DataView;
use crate::file::set_file_modified;

#[derive(Clone, Serialize, Deserialize)]
pub struct ImageDef {
	pub first_sprite_index: u16,
	pub next_sprite_index: u16,
	pub width_in_sprites: u8,
	pub height_in_sprites: u8,
	pub first_palette_index: u16,
	pub subimage_count: usize,
	pub colors_used: Vec<Color>,
	pub subimage_width: u32,
	pub subimage_height: u32,
	pub offset_x: i16,
	pub offset_y: i16
}

impl ImageDef {
	pub fn to_images(&self, sprites: &[Sprite], colors: &[Color]) -> Result<Vec<RgbaImage>, Box<dyn Error>> {
		let first_color_index = self.first_palette_index as usize * 4;
		let colors_in_image = &colors[first_color_index..];

		let first_sprite = sprites.get(self.first_sprite_index as usize)
			.ok_or("Unable to find first sprite for image def")?;
		let sprite_width = first_sprite.width as u32;
		let sprite_height = first_sprite.height as u32;
		let subimage_width = sprite_width * self.width_in_sprites as u32;
		let subimage_height = sprite_height * self.height_in_sprites as u32;

		let mut subimages = Vec::new();
		let sprites_per_subimage = self.width_in_sprites as usize * self.height_in_sprites as usize;
		for i in 0..self.subimage_count {
			let mut img = RgbaImage::new(subimage_width, subimage_height);
			for j in 0..sprites_per_subimage {
				let sprite_index = self.first_sprite_index as usize + i*sprites_per_subimage + j;
				let sprite = sprites.get(sprite_index)
					.ok_or(format!("Sprite {} not found", sprite_index))?;
				let sprite_img = sprite.to_image(colors_in_image);
				let x = (j as u32 % self.width_in_sprites as u32) * sprite_width;
				let y = (j as u32 / self.width_in_sprites as u32) * sprite_height;
				img.copy_from(&sprite_img, x, y)?;
			}
			subimages.push(img);
		}

		Ok(subimages)
	}

	pub fn to_spritesheet(&self, sprites: &[Sprite], colors: &[Color]) -> Result<RgbaImage, Box<dyn Error>> {
		let subimages = self.to_images(sprites, colors)?;
		let first_subimage = subimages.first().ok_or("No subimages found")?;
		let spritesheet_width = first_subimage.width() * self.subimage_count as u32;
		let mut spritesheet = RgbaImage::new(spritesheet_width, first_subimage.height());
		let mut x = 0;
		for subimage in subimages {
			spritesheet.copy_from(&subimage, x, 0)?;
			x += subimage.width();
		}
		Ok(spritesheet)
	}

	pub fn update_first_palette_index(&mut self, colors: &[Color]) -> Result<(), Box<dyn Error>> {
		let mut i = 0;
		let mut palette_found = false;
		while i < colors.len() {
			if colors[i..].starts_with(&self.colors_used) {
				palette_found = true;
				self.first_palette_index = i as u16 / 4;
				break;
			}
			i += 4;
		}
		if palette_found {
			Ok(())
		} else {
			Err("Unable to find first palette index".into())
		}
	}
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
			subimage_count: 0,
			colors_used: Vec::new(),
			subimage_width: 0,
			subimage_height: 0,
			offset_x: 0,
			offset_y: 0
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
pub fn update_image_def(handle: AppHandle, index: usize, offset_x: i16, offset_y: i16, first_palette_index: Option<u16>) -> Option<ImageDef> {
	let data_state: State<DataState> = handle.state();

	let mut sprite_pack_opt = data_state.sprite_pack.lock().unwrap();
	if let Some(sprite_pack) = sprite_pack_opt.as_mut() {
		if let Some(image_def) = sprite_pack.image_defs.get_mut(index) {
			let dx = offset_x - image_def.offset_x;
			let dy = offset_y - image_def.offset_y;
			let sprites_per_subimage = image_def.width_in_sprites as usize * image_def.height_in_sprites as usize;
			let sprite_count = sprites_per_subimage * image_def.subimage_count;
			let first_sprite_index = image_def.first_sprite_index as usize;
			let last_sprite_index = first_sprite_index + sprite_count;

			for sprite in sprite_pack.sprites[first_sprite_index..last_sprite_index].iter_mut() {
				sprite.offset_x += dx;
				sprite.offset_y += dy;
			}

			image_def.offset_x = offset_x;
			image_def.offset_y = offset_y;

			if let Some(first_palette_index) = first_palette_index {
				image_def.first_palette_index = first_palette_index as u16;
			}

			set_file_modified(&handle, true);
			update_window_title(&handle);
			return Some(image_def.clone());
		}
	}

	None
}
