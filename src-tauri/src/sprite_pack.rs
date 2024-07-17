use std::error::Error;
use image::RgbaImage;
use crate::data_view::DataView;

pub mod image_def;
pub mod palette;
pub mod sprite;

#[derive(Clone, serde::Serialize)]
pub struct SpritePack {
	pub image_defs: Vec<image_def::ImageDef>,
	pub palettes: Vec<palette::Color>,
	pub sprites: Vec<sprite::Sprite>
}

pub fn get_sprite_pack(data: &DataView) -> Result<SpritePack, Box<dyn Error>> {
	let image_defs_offset = data.get_u32(0) as usize;
	let sprite_defs_offset = data.get_u32(4) as usize;
	let palettes_offset = data.get_u32(8) as usize;
	let pixel_data_offset = data.get_u32(12) as usize;

	let image_def_data = data.chunk(image_defs_offset, sprite_defs_offset - image_defs_offset);
	let mut image_defs = image_def::get_image_defs(&image_def_data)?;

	let palette_data = data.chunk(palettes_offset, pixel_data_offset - palettes_offset);
	let palettes = palette::get_palettes(&palette_data)?;

	let sprite_def_data = data.chunk(sprite_defs_offset, palettes_offset - sprite_defs_offset);
	let pixel_data = data.chunk(pixel_data_offset, data.len() - pixel_data_offset);
	let sprites = sprite::get_sprites(&sprite_def_data, &pixel_data)?;

	image_def::calc_subimage_counts(&mut image_defs, sprites.len());

	let sprite_pack = SpritePack { image_defs, palettes, sprites };

	Ok(sprite_pack)
}

pub fn get_image_data(sprite_pack: &SpritePack) -> Result<Vec<Vec<RgbaImage>>, Box<dyn Error>> {
	let image_defs = &sprite_pack.image_defs;
	let palettes = &sprite_pack.palettes;
	let sprites = &sprite_pack.sprites;

	let mut images = Vec::new();

	for i in 0..image_defs.len() {
		let image_def = &image_defs[i];

		let first_sprite_index = image_def.first_sprite_index as usize;
		let next_sprite_index = if i+1 < image_defs.len() {
			image_defs[i+1].first_sprite_index as usize
		} else {
			sprites.len()
		};
		let image_sprites = &sprites[first_sprite_index..next_sprite_index];
		let sprites_per_subimage = image_def.width_in_sprites as usize * image_def.height_in_sprites as usize;
		let subimage_count = image_sprites.len() / sprites_per_subimage;

		let image_colors = &palettes[(4 * image_def.first_palette_index as usize)..];

		let mut subimages = Vec::new();

		for j in 0..subimage_count {
			let first_subimage_sprite_index = sprites_per_subimage as usize * j;
			let first_subimage_sprite = &image_sprites[first_subimage_sprite_index];
			let subimage_width = image_def.width_in_sprites as u32 * first_subimage_sprite.width as u32;
			let subimage_height = image_def.height_in_sprites as u32 * first_subimage_sprite.height as u32;

			let mut img = RgbaImage::new(subimage_width, subimage_height);
			for row in 0..image_def.height_in_sprites as usize {
				for col in 0..image_def.width_in_sprites as usize {
					let sprite_index = first_subimage_sprite_index + (row * image_def.width_in_sprites as usize) + col;
					let sprite = &image_sprites[sprite_index];
					let sprite_width = sprite.width as usize;
					let sprite_height = sprite.height as usize;
					for y in 0..sprite_height {
						for x in 0..sprite_width {
							let adj_x = x + (col * sprite_width);
							let adj_y = y + (row * sprite_height);
							let color_index = sprite.pixels.get(y * sprite_width + x).ok_or("pixel data not found")?;
							let color = image_colors.get(*color_index as usize).ok_or("color not found")?;
							img.put_pixel(adj_x as u32, adj_y as u32, color.into_rgba());
						}
					}
				}
			}

			subimages.push(img);
		}

		images.push(subimages);
	}

	Ok(images)
}

pub fn save_sprite_pack(sprite_pack: &SpritePack) -> Result<Vec<u8>, Box<dyn Error>> {
	let mut data: Vec<u8> = Vec::new();

	let image_def_data = image_def::save_image_defs(&sprite_pack.image_defs)?;
	let palette_data = palette::save_palettes(&sprite_pack.palettes)?;
	let (sprite_def_data, pixel_data) = sprite::save_sprites(&sprite_pack.sprites)?;

	let image_defs_offset = 16;
	let sprite_defs_offset = image_defs_offset + image_def_data.len();
	let palettes_offset = sprite_defs_offset + sprite_def_data.len();
	let pixel_data_offset = palettes_offset + palette_data.len();

	for bytes in u32::to_le_bytes(image_defs_offset as u32) {
		data.push(bytes);
	}
	for bytes in u32::to_le_bytes(sprite_defs_offset as u32) {
		data.push(bytes);
	}
	for bytes in u32::to_le_bytes(palettes_offset as u32) {
		data.push(bytes);
	}
	for bytes in u32::to_le_bytes(pixel_data_offset as u32) {
		data.push(bytes);
	}

	data = [data, image_def_data, sprite_def_data, palette_data, pixel_data].concat();

	Ok(data)
}

pub fn get_spritesheet_dims(subimages: &[RgbaImage]) -> (u32, u32) {
	let mut width = 0;
	let mut height = 0;
	for subimage in subimages {
		width += subimage.width();
		if subimage.height() > height {
			height = subimage.height();
		}
	}
	(width, height)
}
