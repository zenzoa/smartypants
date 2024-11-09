use std::error::Error;
use image::{ RgbaImage, GenericImage };
use serde::{ Serialize, Deserialize };

use super::sprite::Sprite;
use super::palette::Color;
use crate::data_view::DataView;


#[derive(Clone)]
pub struct ImageDef {
	pub first_sprite_index: u16,
	pub width_in_sprites: u8,
	pub height_in_sprites: u8,
	pub first_palette_index: usize,
}

#[derive(Clone)]
pub struct ImageSet {
	pub width: u32,
	pub height: u32,
	pub width_in_sprites: u32,
	pub height_in_sprites: u32,
	pub subimages: Vec<SubImage>
}

#[derive(Clone)]
pub struct SubImage {
	pub offset_x: i32,
	pub offset_y: i32,
	pub color_data: Vec<Color>
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ImageSummary {
	pub width: u32,
	pub height: u32,
	pub subimages: Vec<SubImageSummary>
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SubImageSummary {
	pub offset_x: i32,
	pub offset_y: i32,
}

impl ImageSet {
	pub fn to_summary(&self) -> ImageSummary {
		ImageSummary {
			width: self.width,
			height: self.height,
			subimages: self.subimages.iter().map(|s|
				SubImageSummary { offset_x: s.offset_x, offset_y: s.offset_y }
			).collect()
		}
	}

	pub fn to_images(&self) -> Result<Vec<RgbaImage>, Box<dyn Error>> {
		let mut imgs = Vec::new();
		for subimage in &self.subimages {
			let mut image_buffer = Vec::new();
			for pixel in &subimage.color_data {
				image_buffer = [image_buffer, pixel.as_vec()].concat();
			}
			let img = RgbaImage::from_vec(self.width, self.height, image_buffer).ok_or("Unable to convert image definition to image")?;
			imgs.push(img);
		}

		Ok(imgs)
	}

	pub fn to_spritesheet(&self) -> Result<RgbaImage, Box<dyn Error>> {
		let imgs = self.to_images()?;
		let spritesheet_width = self.width * self.subimages.len() as u32;
		let mut spritesheet = RgbaImage::new(spritesheet_width, self.height);
		let mut x = 0;
		for img in imgs {
			spritesheet.copy_from(&img, x, 0)?;
			x += img.width();
		}
		Ok(spritesheet)
	}
}

pub fn get_image_sets(data: &DataView, sprites: &[Sprite], all_colors: &[Color]) -> Result<Vec<ImageSet>, Box<dyn Error>> {
	let mut image_sets = Vec::new();

	let mut i = 0;
	while i + 6 <= data.len() {
		let first_sprite_index = data.get_u16(i) as usize;
		let next_sprite_index = if i + 6 < data.len() {
			data.get_u16(i+6) as usize
		} else {
			sprites.len()
		};

		let width_in_sprites = data.get_u8(i + 2) as u32;
		let height_in_sprites = data.get_u8(i + 3) as u32;

		let first_sprite = sprites.get(first_sprite_index).ok_or("Sprite index not found")?;
		let width = width_in_sprites * first_sprite.width;
		let height = height_in_sprites * first_sprite.height;

		let first_color_index = data.get_u16(i + 4) as usize * 4;
		let colors = all_colors[first_color_index..].to_vec();

		let subimage_size = width * height;
		let sprites_per_subimage = (width_in_sprites * height_in_sprites) as usize;
		let subimage_count = (next_sprite_index - first_sprite_index) / sprites_per_subimage;
		let mut subimages = Vec::new();

		for j in 0..subimage_count {
			let mut color_data = vec![Color::new(0, 0, 0, 0); subimage_size as usize];

			let subimage_first_sprite = first_sprite_index + (j * sprites_per_subimage);
			let subimage_sprites = sprites[subimage_first_sprite..subimage_first_sprite+sprites_per_subimage].to_vec();

			let mut offset_x = 128;
			let mut offset_y = 128;

			for (m, sprite) in subimage_sprites.iter().enumerate() {
				offset_x = offset_x.min(sprite.offset_x - (sprite.width as i32 / 2));
				offset_y = offset_y.min(sprite.offset_y - (sprite.height as i32 / 2));
				let col = m % width_in_sprites as usize;
				let row = m / width_in_sprites as usize;
				for (n, pixel) in sprite.pixels.iter().enumerate() {
					let x = (n % sprite.width as usize) + (col * sprite.width as usize);
					let y = (n / sprite.width as usize) + (row * sprite.height as usize);
					color_data[x + (y * width as usize)] = colors[*pixel as usize];
				}
			}

			subimages.push(SubImage {
				offset_x,
				offset_y,
				color_data
			})
		}

		image_sets.push(ImageSet {
			width,
			height,
			width_in_sprites,
			height_in_sprites,
			subimages
		});

		i += 6;
	}

	Ok(image_sets)
}

pub fn save_image_sets(image_sets: &[ImageSet]) -> Result<(Vec<u8>, Vec<Sprite>, Vec<Color>), Box<dyn Error>> {
	let mut data = Vec::new();
	let mut sprites = Vec::new();
	let mut palettes: Vec<Vec<Color>> = Vec::new();

	let mut image_defs = Vec::new();

	for image_set in image_sets {
		// get colors used in image
		let mut colors = Vec::new();
		for subimage in &image_set.subimages {
			for color in &subimage.color_data {
				if !colors.contains(color) {
					colors.push(*color);
				}
			}
		}
		colors.sort();

		// look for existing color palette
		let mut palette_exists = false;
		let mut palette_index = 0;
		for (i, palette) in palettes.iter_mut().enumerate() {
			if palette.starts_with(&colors) {
				palette_exists = true;
				palette_index = i;
				break;
			} else if colors.starts_with(palette) {
				palette.clone_from(&colors);
				palette_exists = true;
				palette_index = i;
				break;
			}
		}

		// if none found, add new color palette
		if !palette_exists {
			palettes.push(colors.clone());
			palette_index = palettes.len() - 1;
		}

		// save image def info for later
		image_defs.push(ImageDef {
			first_sprite_index: sprites.len() as u16,
			width_in_sprites: image_set.width_in_sprites as u8,
			height_in_sprites: image_set.height_in_sprites as u8,
			first_palette_index: palette_index
		});

		let sprite_width = image_set.width / image_set.width_in_sprites;
		let sprite_height = image_set.height / image_set.height_in_sprites;

		// determine bits per pixel (aka color depth)
		let bpp = if colors.len() < 4 {
			2
		} else if colors.len() < 16 {
			4
		} else if colors.len() < 64 {
			6
		} else {
			8
		};

		// add sprites
		for subimage in &image_set.subimages {
			for row in 0..image_set.height_in_sprites {
				for col in 0..image_set.width_in_sprites {
					let mut pixels = Vec::new();
					for y in 0..sprite_height {
						for x in 0..sprite_width {
							let sprite_x = x + (col * sprite_width);
							let sprite_y = y + (row * sprite_height);
							let color = subimage.color_data[(sprite_x + (sprite_y * image_set.width)) as usize];
							let pixel_index = colors.iter().position(|&c| c == color).ok_or("Color not found")? as u32;
							pixels.push(pixel_index);
						}
					}

					sprites.push(Sprite {
						width: sprite_width,
						height: sprite_height,
						bpp,
						offset_x: subimage.offset_x + (col * sprite_width) as i32,
						offset_y: subimage.offset_y + (row * sprite_height) as i32,
						pixels
					})
				}
			}
		}
	}

	// fill out palettes so they're all multiples of 4
	let mut current_palette_index = 0;
	let mut palette_indexes = Vec::new();
	for palette in palettes.iter_mut() {
		while palette.len() % 4 != 0 {
			palette.push(Color::new(0, 0, 0, 0));
		}
		palette_indexes.push(current_palette_index);
		current_palette_index += palette.len() / 4;
	}

	for image_def in image_defs {
		// determine first_palette_index for each image_def
		let real_first_palette_index = palette_indexes[image_def.first_palette_index] as u16;

		// write image def data
		for bytes in u16::to_le_bytes(image_def.first_sprite_index) {
			data.push(bytes);
		}
		data.push(image_def.width_in_sprites);
		data.push(image_def.height_in_sprites);
		for bytes in u16::to_le_bytes(real_first_palette_index) {
			data.push(bytes);
		}
	}

	let all_colors = palettes.into_iter().flatten().collect();

	Ok((data, sprites, all_colors))
}
