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
	pub width_in_sprites: u8,
	pub height_in_sprites: u8,
	pub first_palette_index: u16,
	pub width: u32,
	pub height: u32,
	pub subimage_defs: Vec<SubImageDef>,
	pub colors_used: Vec<Color>
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SubImageDef {
	pub offset_x: i16,
	pub offset_y: i16,
	pub sprites: Vec<Sprite>
}

impl ImageDef {
	pub fn to_images(&self, palettes: &[Color]) -> Result<Vec<RgbaImage>, Box<dyn Error>> {
		let first_color_index = self.first_palette_index as usize * 4;
		let colors = &palettes[first_color_index..];

		let mut subimages = Vec::new();
		for subimage_def in &self.subimage_defs {
			let mut subimage = RgbaImage::new(self.width, self.height);
			for (i, sprite) in subimage_def.sprites.iter().enumerate() {
				let sprite_img = sprite.to_image(colors);
				let x = (i as u32 % self.width_in_sprites as u32) * sprite.width as u32;
				let y = (i as u32 / self.width_in_sprites as u32) * sprite.height as u32;
				subimage.copy_from(&sprite_img, x, y)?;
			}
			subimages.push(subimage);
		}

		Ok(subimages)
	}

	pub fn to_spritesheet(&self, palettes: &[Color]) -> Result<RgbaImage, Box<dyn Error>> {
		let subimages = self.to_images(palettes)?;
		let spritesheet_width = self.width * self.subimage_defs.len() as u32;
		let mut spritesheet = RgbaImage::new(spritesheet_width, self.height);
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

pub fn get_image_defs(data: &DataView, sprites: &[Sprite]) -> Result<Vec<ImageDef>, Box<dyn Error>> {
	let mut image_defs = Vec::new();

	let mut i = 0;
	while i + 6 <= data.len() {
		let first_sprite_index = data.get_u16(i) as usize;
		let next_sprite_index = if i + 6 < data.len() {
			data.get_u16(i+6) as usize
		} else {
			sprites.len()
		};

		let width_in_sprites = data.get_u8(i + 2);
		let height_in_sprites = data.get_u8(i + 3);

		let width = width_in_sprites as u32 * sprites[first_sprite_index].width as u32;
		let height = height_in_sprites as u32 * sprites[first_sprite_index].height as u32;

		let first_palette_index = data.get_u16(i + 4);

		let sprites_per_subimage = width_in_sprites as usize * height_in_sprites as usize;
		let subimage_count = (next_sprite_index - first_sprite_index) / sprites_per_subimage;
		let mut subimage_defs = Vec::new();
		for j in 0..subimage_count {
			let subimage_first_sprite = first_sprite_index + (j * sprites_per_subimage);
			let subimage_sprites = sprites[subimage_first_sprite..subimage_first_sprite+sprites_per_subimage].to_vec();
			let mut min_x = 128;
			let mut min_y = 128;
			for sprite in &subimage_sprites {
				let half_width = sprite.width as i16 / 2;
				let half_height = sprite.height as i16 / 2;
				min_x = min_x.min(sprite.offset_x - half_width);
				min_y = min_y.min(sprite.offset_y - half_height);
			}
			subimage_defs.push(SubImageDef {
				offset_x: min_x,
				offset_y: min_y,
				sprites: subimage_sprites
			})
		}

		image_defs.push(ImageDef {
			width_in_sprites,
			height_in_sprites,
			first_palette_index,
			width,
			height,
			subimage_defs,
			colors_used: Vec::new()
		});

		i += 6;
	}

	Ok(image_defs)
}

pub fn save_image_defs(image_defs: &[ImageDef]) -> Result<(Vec<u8>, Vec<Sprite>), Box<dyn Error>> {
	let mut data: Vec<u8> = Vec::new();
	let mut sprites = Vec::new();

	for image_def in image_defs {
		let first_sprite_index = sprites.len() as u16;
		for bytes in u16::to_le_bytes(first_sprite_index) {
			data.push(bytes);
		}
		data.push(image_def.width_in_sprites);
		data.push(image_def.height_in_sprites);
		for bytes in u16::to_le_bytes(image_def.first_palette_index) {
			data.push(bytes);
		}

		for subimage_def in &image_def.subimage_defs {
			sprites = [sprites, subimage_def.sprites.clone()].concat();
		}
	}

	Ok((data, sprites))
}

#[tauri::command]
pub fn update_image_def(handle: AppHandle, index: usize, offsets_x: Vec<i16>, offsets_y: Vec<i16>, first_palette_index: Option<u16>) -> Option<ImageDef> {
	let data_state: State<DataState> = handle.state();

	let mut sprite_pack_opt = data_state.sprite_pack.lock().unwrap();
	if let Some(sprite_pack) = sprite_pack_opt.as_mut() {
		if let Some(image_def) = sprite_pack.image_defs.get_mut(index) {
			for (j, subimage_def) in image_def.subimage_defs.iter_mut().enumerate() {
				if let Some(offset_x) = offsets_x.get(j) {
					if let Some(offset_y) = offsets_y.get(j) {
						let dx = *offset_x - subimage_def.offset_x;
						let dy = *offset_y - subimage_def.offset_y;
						for sprite in subimage_def.sprites.iter_mut() {
							sprite.offset_x += dx;
							sprite.offset_y += dy;
						}
						subimage_def.offset_x = *offset_x;
						subimage_def.offset_y = *offset_y;
					}
				}
			}

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
