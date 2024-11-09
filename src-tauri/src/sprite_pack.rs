use std::error::Error;
use image::RgbaImage;

use crate::data_view::DataView;

pub mod image_def;
pub mod palette;
pub mod sprite;

use image_def::{ ImageSet, get_image_sets, save_image_sets };
use palette::{ get_palettes, save_palettes };
use sprite::{ get_sprites, save_sprites, save_pixel_data };

#[derive(Clone)]
pub struct SpritePack {
	pub image_sets: Vec<ImageSet>
}

impl SpritePack {
	pub fn from_data(data: &DataView) -> Result<Self, Box<dyn Error>> {
		let image_defs_offset = data.get_u32(0) as usize;
		let sprite_defs_offset = data.get_u32(4) as usize;
		let palettes_offset = data.get_u32(8) as usize;
		let pixel_data_offset = data.get_u32(12) as usize;

		let colors = get_palettes(
			&data.data[palettes_offset..pixel_data_offset]
		)?;

		let sprites = get_sprites(
			&data.chunk(sprite_defs_offset, palettes_offset-sprite_defs_offset),
			&data.chunk(pixel_data_offset, data.len()-pixel_data_offset)
		)?;

		let image_sets = get_image_sets(
			&data.chunk(image_defs_offset, sprite_defs_offset-image_defs_offset),
			&sprites,
			&colors
		)?;

		let sprite_pack = Self { image_sets };

		Ok(sprite_pack)
	}

	pub fn as_bytes(&mut self) -> Result<Vec<u8>, Box<dyn Error>> {
		let (image_def_data, sprites, colors) = save_image_sets(&self.image_sets)?;
		let mut palette_data = save_palettes(&colors)?;
		let (pixel_data, sprite_defs) = save_pixel_data(&sprites);
		let sprite_data = save_sprites(&sprite_defs)?;

		let image_defs_offset = 16;
		let sprites_offset = image_defs_offset + image_def_data.len();
		let palettes_offset = sprites_offset + sprite_data.len();
		let mut pixel_data_offset = palettes_offset + palette_data.len();

		while pixel_data_offset % 32 != 0 {
			pixel_data_offset += 1;
		}
		let padded_palette_size = pixel_data_offset - palettes_offset;
		palette_data.resize(padded_palette_size, 0);

		let mut data: Vec<u8> = Vec::new();

		data.extend_from_slice(&u32::to_le_bytes(image_defs_offset as u32));
		data.extend_from_slice(&u32::to_le_bytes(sprites_offset as u32));
		data.extend_from_slice(&u32::to_le_bytes(palettes_offset as u32));
		data.extend_from_slice(&u32::to_le_bytes(pixel_data_offset as u32));

		data.extend_from_slice(&image_def_data);
		data.extend_from_slice(&sprite_data);
		data.extend_from_slice(&palette_data);
		data.extend_from_slice(&pixel_data);

		Ok(data)
	}

	pub fn get_image_data(&self) -> Result<Vec<Vec<RgbaImage>>, Box<dyn Error>> {
		let mut images = Vec::new();
		for image_set in &self.image_sets {
			let subimages = image_set.to_images()?;
			images.push(subimages);
		}
		Ok(images)
	}
}
