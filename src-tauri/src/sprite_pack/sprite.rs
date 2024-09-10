use std::error::Error;
use std::cmp::Ordering;
use image::RgbaImage;

use crate::data_view::{ DataView, BitWriter, words_to_bytes };
use super::palette::Color;

#[derive(Clone, serde::Serialize)]
pub struct Sprite {
	pub pixel_data_index: u16,
	pub pixel_data_offset: usize,
	pub offset_x: u16,
	pub offset_y: u16,
	pub bits_per_pixel: u8,
	pub is_flipped: u8, // unused on Smart
	pub width: u8,
	pub height: u8,
	pub palette_bank: u8,
	pub draw_depth: u8, // typically not set
	pub blend_enabled: bool, // typically not set
	pub is_quadrupled: bool, // unused on Smart
	pub pixels: Vec<u16>
}

impl Sprite {
	pub fn to_image(&self, colors: &[Color]) -> RgbaImage {
		let mut img = RgbaImage::new(self.width as u32, self.height as u32);
		for (i, pixel) in self.pixels.iter().enumerate() {
			let x = i % self.width as usize;
			let y = i / self.width as usize;
			let color = colors.get(*pixel as usize).cloned().unwrap_or(Color::default());
			img.put_pixel(x as u32, y as u32, color.as_rgba());
		}
		img
	}

	pub fn update_from_image(&mut self, img: &RgbaImage, colors: &[Color], bpp: Option<u8>) -> Result<(), Box<dyn Error>> {
		if img.width() != self.width as u32 || img.height() != self.height as u32 {
			return Err(format!("Image is not the right dimensions for this sprite: {}x{} instead of {}x{}", img.width(), img.height(), self.width, self.height).into());
		}
		let mut new_pixels = Vec::new();
		let mut largest_color_index = 0;
		for pixel in img.pixels() {
			let color = Color::from_rgba(pixel);
			let color_index = colors.iter().position(|c| *c == color || (c.a == 0 && color.a == 0))
				.ok_or(format!("Color index not found for {:?}", color))?;
			if color_index > largest_color_index {
				largest_color_index = color_index;
			}
			new_pixels.push(color_index as u16);
		}

		if let Some(new_bpp) = bpp {
			self.bits_per_pixel = new_bpp;
		}

		self.pixels = new_pixels;

		Ok(())
	}

	pub fn get_pixel_data(&self) -> Vec<u8> {
		let mut bits = BitWriter::new();
		for pixel in &self.pixels {
			bits.write_bits(*pixel as u32, self.bits_per_pixel as usize);
		}
		bits.end();
		bits.bytes
	}
}

#[derive(PartialEq, Eq)]
struct PixelDataChunk {
	start: usize,
	end: usize,
	data: Vec<u8>
}

impl Ord for PixelDataChunk {
	fn cmp(&self, other: &Self) -> Ordering {
		self.start.cmp(&other.start)
	}
}

impl PartialOrd for PixelDataChunk {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}

pub fn get_sprites(def_data: &DataView, pixel_data: &DataView) -> Result<Vec<Sprite>, Box<dyn Error>> {
	let mut sprites = Vec::new();

	let mut max_index = 0;

	let mut i = 0;
	while i + 8 <= def_data.len() {
		let props = def_data.get_u16(i + 6);
		let bits_per_pixel = [2, 4, 6, 8][(props & 0x0003) as usize] as u8;
		let is_flipped = ((props & 0x000c) >> 2) as u8;
		let width = [8, 16, 32, 64][((props & 0x0030) >> 4) as usize] as u8;
		let height = [8, 16, 32, 64][((props & 0x00c0) >> 6) as usize] as u8;
		let palette_bank = ((props & 0x0f00) >> 8) as u8;
		let draw_depth = ((props & 0x3000) >> 12) as u8;
		let blend_enabled = ((props & 0x4000) >> 14) > 0;
		let is_quadrupled = ((props & 0x8000) >> 15) > 0;

		let pixel_data_index = def_data.get_u16(i);
		let offset_x = def_data.get_u16(i + 2);
		let offset_y = def_data.get_u16(i + 4);

		let sprite_size = (width as usize) * (height as usize);
		let byte_count = sprite_size * (bits_per_pixel as usize) / 8;
		let pixel_data_offset = byte_count * (pixel_data_index as usize);
		let bits = pixel_data.get_bits(pixel_data_offset, byte_count);

		if pixel_data_index > max_index {
			max_index = pixel_data_index;
		}

		let mut pixels = Vec::new();
		for i in 0..sprite_size {
			let pixel_offset = i * bits_per_pixel as usize;
			let pixel_bits = bits[pixel_offset..(pixel_offset + bits_per_pixel as usize)].to_vec();
			let bit_string: String = pixel_bits.iter().map(|x| x.to_string()).collect();
			let color_index = u16::from_str_radix(&bit_string, 2)?;
			pixels.push(color_index);
		}

		sprites.push(Sprite {
			pixel_data_index,
			pixel_data_offset,
			offset_x,
			offset_y,
			bits_per_pixel,
			is_flipped,
			width,
			height,
			palette_bank,
			draw_depth,
			blend_enabled,
			is_quadrupled,
			pixels,
		});

		i += 8;
	}

	println!("max pixel data index: {}", max_index);

	Ok(sprites)
}

pub fn save_sprites(sprites: &mut [Sprite]) -> Result<(Vec<u8>, Vec<u8>), Box<dyn Error>> {
	let mut sprite_defs: Vec<u16> = Vec::new();
	let mut pixel_data_parts: Vec<Vec<u8>> = Vec::new();
	let mut pixel_data_len: usize = 0;

	let pixel_data = save_pixel_data(sprites);

	for sprite in sprites.iter_mut() {
		sprite_defs.push(sprite.pixel_data_index);
		sprite_defs.push(sprite.offset_x);
		sprite_defs.push(sprite.offset_y);

		let bits_per_pixel = match sprite.bits_per_pixel {
			2 => 0,
			4 => 1,
			6 => 2,
			8 => 3,
			_ => return Err("Invalid bits per pixel".into())
		} as u16;
		let is_flipped = (sprite.is_flipped as u16) << 2;
		let width = match sprite.width {
			8 => 0,
			16 => 1,
			32 => 2,
			64 => 3,
			_ => return Err("Invalid width".into())
		} << 4;
		let height = match sprite.height {
			8 => 0,
			16 => 1,
			32 => 2,
			64 => 3,
			_ => return Err("Invalid height".into())
		} << 6;
		let palette_bank = (sprite.palette_bank as u16) << 8;
		let draw_depth = (sprite.draw_depth as u16) << 12;
		let blend_enabled = if sprite.blend_enabled { 1 << 14 } else { 0 };
		let is_quadrupled = if sprite.is_quadrupled { 1 << 15 } else { 0 };

		let props = bits_per_pixel | is_flipped | width | height | palette_bank | draw_depth | blend_enabled | is_quadrupled;
		sprite_defs.push(props);

		let mut bits = BitWriter::new();
		for pixel in &sprite.pixels {
			bits.write_bits(*pixel as u32, sprite.bits_per_pixel as usize);
		}
		bits.end();
		pixel_data_parts.push(bits.bytes);

		let byte_count = (sprite.width as usize) * (sprite.height as usize) * (sprite.bits_per_pixel as usize) / 8;
		let min_len = sprite.pixel_data_offset + byte_count;
		if pixel_data_len < min_len {
			pixel_data_len = min_len;
		}
	}

	Ok((words_to_bytes(&sprite_defs), pixel_data))
}

fn save_pixel_data(sprites: &mut [Sprite]) -> Vec<u8> {
	let mut chunks: Vec<PixelDataChunk> = Vec::new();

	let mut max_index = 0;

	let mut sorted_sprites = sprites.iter_mut().collect::<Vec<&mut Sprite>>();
	sorted_sprites.sort_by(|a, b| {
		a.pixels.len().cmp(&b.pixels.len())
	});

	for sprite in sorted_sprites {
		let mut offset = 0;

		let pixel_data = sprite.get_pixel_data();
		let byte_len = pixel_data.len();
		let mut add_chunk = true;

		for (i, chunk) in chunks.iter().enumerate() {
			offset = find_next_offset(chunk.start, byte_len);
			if offset + byte_len < chunk.end && chunk.data[(offset - chunk.start)..].starts_with(&pixel_data) {
				add_chunk = false;
				break;
			} else {
				offset = find_next_offset(chunk.end, byte_len);
				if i+1 < chunks.len() && chunks[i+1].start > offset + byte_len {
					break;
				}
			}
		}

		if add_chunk {
			chunks.push(PixelDataChunk{
				start: offset,
				end: offset + byte_len,
				data: sprite.get_pixel_data()
			});
			chunks.sort();
		}

		sprite.pixel_data_offset = offset;
		sprite.pixel_data_index = (offset / byte_len) as u16;

		if sprite.pixel_data_index > max_index {
			max_index = sprite.pixel_data_index;
		}
	}

	let mut data = Vec::new();
	for (i, chunk) in chunks.iter().enumerate() {
		let end = if i > 0 { chunks[i-1].end } else { 0 };
		let padding = chunk.start - end;
		data.extend_from_slice(&vec![0_u8; padding]);
		data.extend_from_slice(&chunk.data);
	}

	println!("max pixel data index: {}", max_index);

	data
}

fn find_next_offset(start: usize, byte_len: usize) -> usize {
	let mut offset = start;
	while offset % byte_len != 0 {
		offset += 1;
	}
	offset
}
