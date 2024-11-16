use std::error::Error;
use image::{ RgbaImage, GenericImage };
use serde::{ Serialize, Deserialize };
use tauri::{ AppHandle, Manager, State };

use super::sprite::Sprite;
use super::palette::Color;
use crate::data_view::DataView;
use crate::file::set_file_modified;
use crate::{ DataState, update_window_title };

#[derive(Clone)]
pub struct ImageSet {
	pub original_index: usize,
	pub width: u32,
	pub height: u32,
	pub width_in_sprites: u32,
	pub height_in_sprites: u32,
	pub is_quadrupled: bool,
	pub first_palette_index: usize,
	pub palettes: Vec<Vec<Color>>,
	pub subimages: Vec<SubImage>
}

#[derive(Clone)]
pub struct SubImage {
	pub offset_x: i32,
	pub offset_y: i32,
	pub pixel_data: Vec<u32>
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ImageSummary {
	pub width: u32,
	pub height: u32,
	pub palette_count: usize,
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
			palette_count: self.palettes.len(),
			subimages: self.subimages.iter().map(|s|
				SubImageSummary { offset_x: s.offset_x, offset_y: s.offset_y }
			).collect()
		}
	}

	pub fn to_images(&self, palette_index: usize) -> Result<Vec<RgbaImage>, Box<dyn Error>> {
		let mut imgs = Vec::new();
		for subimage in &self.subimages {
			let mut image_buffer = Vec::new();
			for pixel in &subimage.pixel_data {
				let color = self.palettes[palette_index][*pixel as usize];
				image_buffer = [image_buffer, color.as_vec()].concat();
			}
			let img = RgbaImage::from_vec(self.width, self.height, image_buffer).ok_or("Unable to convert image definition to image")?;
			imgs.push(img);
		}

		Ok(imgs)
	}

	pub fn to_spritesheet(&self) -> Result<RgbaImage, Box<dyn Error>> {
		let spritesheet_width = self.width * self.subimages.len() as u32;
		let spritesheet_height = self.height * self.palettes.len() as u32;
		let mut spritesheet = RgbaImage::new(spritesheet_width, spritesheet_height);
		for i in 0..self.palettes.len() {
			let imgs = self.to_images(i)?;
			for (j, img) in imgs.iter().enumerate() {
				let x = self.width * j as u32;
				let y = self.height * i as u32;
				spritesheet.copy_from(img, x, y)?;
			}
		}
		Ok(spritesheet)
	}

	pub fn to_sprites(&self, bpp: u32, sprite_index: usize) -> Result<Vec<Sprite>, Box<dyn Error>> {
		let mut sprites = Vec::new();

		let sprite_width = self.width / self.width_in_sprites;
		let sprite_height = self.height / self.height_in_sprites;

		for subimage in &self.subimages {
			for row in 0..self.height_in_sprites {
				for col in 0..self.width_in_sprites {
					let mut pixels = Vec::new();
					for rel_y in 0..sprite_height {
						for rel_x in 0..sprite_width {
							let abs_x = rel_x + (col * sprite_width);
							let abs_y = rel_y + (row * sprite_height);
							let pixel = subimage.pixel_data[(abs_x + (abs_y * self.width)) as usize];
							pixels.push(pixel as u32);
						}
					}

					let offset_x_mod = (sprite_width / 2) + (col * sprite_width);
					let offset_y_mod = (sprite_height / 2) + (row * sprite_height);

					sprites.push(Sprite {
						index: sprite_index + sprites.len(),
						width: if self.is_quadrupled { sprite_width / 4 } else { sprite_width },
						height: if self.is_quadrupled { sprite_height / 4 } else { sprite_height },
						bpp,
						offset_x: subimage.offset_x + offset_x_mod as i32,
						offset_y: subimage.offset_y + offset_y_mod as i32,
						is_quadrupled: self.is_quadrupled,
						pixels
					})
				}
			}
		}

		Ok(sprites)
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
		let is_quadrupled = first_sprite.is_quadrupled;
		let bpp = first_sprite.bpp;

		let first_palette_index = data.get_u16(i + 4) as usize;
		let colors_per_palette = 2_usize.pow(bpp);

		let colors_start = first_palette_index * 4;
		let colors_end = colors_start + colors_per_palette;
		if colors_start > all_colors.len() || colors_end > all_colors.len() {
			return Err(format!("Unable to find color palette for Image Definition {}", image_sets.len()).into());
		}
		let colors = &all_colors[colors_start..colors_end];

		let subimage_size = width * height;
		let sprites_per_subimage = (width_in_sprites * height_in_sprites) as usize;
		let subimage_count = (next_sprite_index - first_sprite_index) / sprites_per_subimage;
		let mut subimages = Vec::new();

		for j in 0..subimage_count {
			let mut pixel_data = vec![0; subimage_size as usize];

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
					pixel_data[x + (y * width as usize)] = *pixel;
				}
			}

			subimages.push(SubImage{ offset_x, offset_y, pixel_data });
		}

		image_sets.push(ImageSet {
			original_index: image_sets.len(),
			width,
			height,
			width_in_sprites,
			height_in_sprites,
			is_quadrupled,
			first_palette_index,
			palettes: vec![colors.to_vec()],
			subimages
		});

		i += 6;
	}

	image_sets.sort_by_key(|i| i.first_palette_index);
	let palette_indexes: Vec<usize> = image_sets.iter().map(|i| i.first_palette_index).collect();
	for (i, image_set) in image_sets.iter_mut().enumerate() {
		if let Some(next_palette_index) = palette_indexes.get(i+1) {
			let colors_per_palette = image_set.palettes[0].len();
			let total_color_count = 4 * (*next_palette_index - image_set.first_palette_index);
			let palette_count = total_color_count / colors_per_palette;
			if palette_count > 1 {
				for j in 1..palette_count {
					let colors_start = (image_set.first_palette_index * 4) + (j * colors_per_palette);
					let colors_end = colors_start + colors_per_palette;
					image_set.palettes.push(all_colors[colors_start..colors_end].to_vec());
				}
			}
		}
	}
	image_sets.sort_by_key(|i| i.original_index);

	Ok(image_sets)
}

pub fn save_image_sets(image_sets: &[ImageSet]) -> Result<(Vec<u8>, Vec<Sprite>, Vec<Color>), Box<dyn Error>> {
	let mut data = Vec::new();
	let mut sprites = Vec::new();

	let mut palettes: Vec<Vec<Color>> = Vec::new();
	let mut palette_indexes = Vec::new();
	let mut palette_chunks = 0;

	for (i, image_set) in image_sets.iter().enumerate() {
		// determine bits per pixel (color depth)
		let color_count = image_set.palettes[0].len();
		let (bpp, goal_colors) = if color_count <= 4 {
			(2, 4)
		} else if color_count <= 16 {
			(4, 16)
		} else if color_count <= 64 {
			(6, 64)
		} else if color_count <= 256 {
			(8, 256)
		} else {
			return Err(format!("Too many colors used in image {} ({}/256)", i, color_count).into());
		};

		// buffer and combine color palettes
		let mut colors = Vec::new();
		for palette in &image_set.palettes {
			let mut palette = palette.clone();
			palette.resize(goal_colors, Color::new(0, 0, 0, 255));
			colors = [colors, palette].concat();
		}
		let palette_count = colors.len() / 4;

		// use existing palette or add new one
		let first_palette_index = match palettes.iter().position(|p| *p == colors) {
			Some(j) => palette_indexes[j],
			None => {
				palettes.push(colors);
				palette_indexes.push(palette_chunks);
				palette_chunks += palette_count;
				*palette_indexes.last().unwrap()
			}
		};

		// add sprites
		let first_sprite_index = sprites.len();
		sprites = [sprites, image_set.to_sprites(bpp, first_sprite_index)?].concat();

		// write image def data
		for bytes in u16::to_le_bytes(first_sprite_index as u16) {
			data.push(bytes);
		}
		data.push(image_set.width_in_sprites as u8);
		data.push(image_set.height_in_sprites as u8);
		for bytes in u16::to_le_bytes(first_palette_index as u16) {
			data.push(bytes);
		}
	}

	let all_colors: Vec<Color> = palettes.into_iter().flatten().collect();

	Ok((data, sprites, all_colors))
}

#[tauri::command]
pub fn update_image_set(handle: AppHandle, index: usize, offsets_x: Vec<i32>, offsets_y: Vec<i32>) -> Option<ImageSummary> {
	let data_state: State<DataState> = handle.state();

	let mut sprite_pack_opt = data_state.sprite_pack.lock().unwrap();
	if let Some(sprite_pack) = sprite_pack_opt.as_mut() {
		if let Some(image_set) = sprite_pack.image_sets.get_mut(index) {
			for (i, subimage) in image_set.subimages.iter_mut().enumerate() {
				if let Some(offset_x) = offsets_x.get(i) {
					subimage.offset_x = *offset_x;
				}
				if let Some(offset_y) = offsets_y.get(i) {
					subimage.offset_y = *offset_y;
				}
			}
			set_file_modified(&handle, true);
			update_window_title(&handle);
			return Some(image_set.to_summary());
		}
	}

	None
}
