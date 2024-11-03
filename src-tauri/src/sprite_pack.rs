use std::error::Error;
use image::{ RgbaImage, GenericImageView };

use crate::data_view::DataView;

pub mod image_def;
pub mod palette;
pub mod sprite;

use image_def::{ ImageDef, get_image_defs, save_image_defs };
use palette::{ Color, get_palettes, save_palettes, generate_palettes };
use sprite::{ Sprite, get_sprites };

#[derive(Clone, serde::Serialize)]
pub struct SpritePack {
	pub image_defs: Vec<ImageDef>,
	pub palettes: Vec<Color>
}

impl SpritePack {
	pub fn from_data(data: &DataView) -> Result<Self, Box<dyn Error>> {
		let image_defs_offset = data.get_u32(0) as usize;
		let sprite_defs_offset = data.get_u32(4) as usize;
		let palettes_offset = data.get_u32(8) as usize;
		let pixel_data_offset = data.get_u32(12) as usize;

		let palettes = get_palettes(
			&data.data[palettes_offset..pixel_data_offset]
		)?;

		let sprites = get_sprites(
			&data.chunk(sprite_defs_offset, palettes_offset-sprite_defs_offset),
			&data.chunk(pixel_data_offset, data.len()-pixel_data_offset)
		)?;

		let image_defs = get_image_defs(
			&data.chunk(image_defs_offset, sprite_defs_offset-image_defs_offset),
			&sprites
		)?;

		let sprite_pack = Self { image_defs, palettes };

		Ok(sprite_pack)
	}

	pub fn as_bytes(&mut self) -> Result<Vec<u8>, Box<dyn Error>> {
		let (image_def_data, mut sprites) = save_image_defs(&self.image_defs)?;
		let mut palette_data = save_palettes(&self.palettes)?;
		let (sprite_def_data, pixel_data) = sprite::save_sprites(&mut sprites)?;

		let image_defs_offset = 16;
		let sprite_defs_offset = image_defs_offset + image_def_data.len();
		let palettes_offset = sprite_defs_offset + sprite_def_data.len();
		let mut pixel_data_offset = palettes_offset + palette_data.len();

		while pixel_data_offset % 32 != 0 {
			pixel_data_offset += 1;
		}
		let padded_palette_size = pixel_data_offset - palettes_offset;
		palette_data.resize(padded_palette_size, 0);

		let mut data: Vec<u8> = Vec::new();

		data.extend_from_slice(&u32::to_le_bytes(image_defs_offset as u32));
		data.extend_from_slice(&u32::to_le_bytes(sprite_defs_offset as u32));
		data.extend_from_slice(&u32::to_le_bytes(palettes_offset as u32));
		data.extend_from_slice(&u32::to_le_bytes(pixel_data_offset as u32));

		data.extend_from_slice(&image_def_data);
		data.extend_from_slice(&sprite_def_data);
		data.extend_from_slice(&palette_data);
		data.extend_from_slice(&pixel_data);

		Ok(data)
	}

	pub fn get_image_data(&mut self) -> Result<Vec<Vec<RgbaImage>>, Box<dyn Error>> {
		let palettes = &self.palettes;

		let mut images = Vec::new();

		for (i, image_def) in self.image_defs.iter_mut().enumerate() {
			let subimages = match image_def.to_images(palettes) {
				Ok(subimages) => subimages,
				Err(why) => return Err(format!("Image Def {}: {}", i, why).into())
			};
			image_def.colors_used = get_colors_in_images(&subimages);
			image_def.width = subimages.first().unwrap().width();
			image_def.height = subimages.first().unwrap().height();
			images.push(subimages);
		}

		Ok(images)
	}

	pub fn update_image_data(&mut self, images: &[Vec<RgbaImage>], lock_colors: bool) -> Result<(), Box<dyn Error>> {
		if !lock_colors {
			self.palettes = generate_palettes(images);
		}

		for (i, image_def) in self.image_defs.iter_mut().enumerate() {
			let subimages = images.get(i)
				.ok_or(format!("Image corresponding to image def {} not found", i))?;

			if !lock_colors {
				if let Err(why) = image_def.update_first_palette_index(&self.palettes) {
					return Err(format!("Image Def {}: {}", i, why).into());
				}
				let color_count = get_colors_in_images(subimages).len();
				if color_count > 256 {
					return Err(format!("Too many colors used ({})", color_count).into());
				}
			}

			let palette = &self.palettes[image_def.first_palette_index as usize * 4..];
			for (j, subimage_def) in image_def.subimage_defs.iter_mut().enumerate() {
				let subimage = subimages.get(j).ok_or(format!("Unable to find subimage {} in image {}", j, i))?;
				let sprite_images = divide_image(subimage, image_def.width_in_sprites as u32, image_def.height_in_sprites as u32);
				for (k, sprite) in subimage_def.sprites.iter_mut().enumerate() {
					let sprite_image = sprite_images.get(k).ok_or(format!("Unable to find sprite {} in image {}, subimage {}", k, i, j))?;
					if let Err(why) = sprite.update_from_image(sprite_image, palette) {
						return Err(format!("Sprite {}-{}: {}", i, j, why).into());
					}
				}
			}
		}

		Ok(())
	}
}

pub fn get_colors_in_image(img: &RgbaImage) -> Vec<Color> {
	let mut colors = Vec::new();
	for pixel in img.pixels() {
		let color = Color::from_rgba(pixel);
		if !colors.contains(&color) {
			colors.push(color);
		}
	}
	colors.sort();
	colors
}

pub fn get_colors_in_images(images: &[RgbaImage]) -> Vec<Color> {
	let mut colors = Vec::new();
	for img in images {
		colors.extend_from_slice(&get_colors_in_image(img));
	}
	colors.sort();
	colors.dedup();
	colors
}

pub fn divide_image(img: &RgbaImage, cols: u32, rows: u32) -> Vec<RgbaImage> {
	let mut subimages = Vec::new();
	let sub_width = img.width() / cols;
	let sub_height = img.height() / rows;
	for row in 0..rows {
		for col in 0..cols {
			let subimage = img.view(col * sub_width, row * sub_height, sub_width, sub_height);
			subimages.push(subimage.to_image());
		}
	}
	subimages
}
