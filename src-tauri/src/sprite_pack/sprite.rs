use std::error::Error;
use std::cmp::Ordering;

use crate::data_view::{ DataView, BitWriter };

#[derive(Eq)]
pub struct SpriteDef {
	pub index: usize,
	pub pixel_data_index: u16,
	pub offset_x: i16,
	pub offset_y: i16,
	pub bpp: u8,
	pub width: u8,
	pub height: u8,
	pub is_quadrupled: bool
}

impl PartialEq for SpriteDef {
	fn eq(&self, other: &Self) -> bool {
		self.index == other.index
	}
}

impl Ord for SpriteDef {
	fn cmp(&self, other: &Self) -> Ordering {
		self.index.cmp(&other.index)
	}
}

impl PartialOrd for SpriteDef {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}

#[derive(Clone, Eq)]
pub struct Sprite {
	pub index: usize,
	pub width: u32,
	pub height: u32,
	pub bpp: u32,
	pub offset_x: i32,
	pub offset_y: i32,
	pub is_quadrupled: bool,
	pub pixels: Vec<u32>
}

impl PartialEq for Sprite {
	fn eq(&self, other: &Self) -> bool {
		self.width * self.height * self.bpp == other.width * other.height * other.bpp
	}
}

impl Ord for Sprite {
	fn cmp(&self, other: &Self) -> Ordering {
		let self_byte_len = self.width * self.height * self.bpp;
		let other_byte_len = other.width * other.height * other.bpp;
		self_byte_len.cmp(&other_byte_len)
	}
}

impl PartialOrd for Sprite {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}

pub fn get_sprites(data: &DataView, all_pixel_data: &DataView) -> Result<Vec<Sprite>, Box<dyn Error>> {
	let mut sprites = Vec::new();

	let mut i = 0;
	while i + 8 <= data.len() {
		let pixel_data_index = data.get_u16(i) as usize;
		let offset_x = data.get_i16(i + 2) as i32;
		let offset_y = data.get_i16(i + 4) as i32;
		let props = data.get_u16(i + 6);

		let bpp = [2, 4, 6, 8][(props & 0x0003) as usize];
		let _is_flipped = (props & 0x000c) >> 2;			// unused on Smart
		let mut width = [8, 16, 32, 64][((props & 0x0030) >> 4) as usize];
		let mut height = [8, 16, 32, 64][((props & 0x00c0) >> 6) as usize];
		let _palette_bank = (props & 0x0f00) >> 8;			// unused on Smart
		let _draw_depth = (props & 0x3000) >> 12;			// unused on Smart
		let _blend_enabled = ((props & 0x4000) >> 14) > 0;	// unused on Smart
		let is_quadrupled = ((props & 0x8000) >> 15) > 0;
		if is_quadrupled {
			width *= 4;
			height *= 4;
		}

		let byte_count = width * height * bpp / 8;
		let pixel_data_offset = pixel_data_index * byte_count;
		let bits = all_pixel_data.get_bits(pixel_data_offset, byte_count);
		let mut pixels = Vec::new();
		for i in 0..(width * height) {
			let pixel_offset = i * bpp;
			let pixel_bits = bits[pixel_offset..(pixel_offset + bpp)].to_vec();
			let bit_string: String = pixel_bits.iter().map(|x| x.to_string()).collect();
			let color_index = u32::from_str_radix(&bit_string, 2)?;
			pixels.push(color_index);
		}

		sprites.push(Sprite {
			index: sprites.len(),
			width: width as u32,
			height: height as u32,
			bpp: bpp as u32,
			offset_x,
			offset_y,
			is_quadrupled,
			pixels
		});

		i += 8;
	}

	Ok(sprites)
}

pub fn save_pixel_data(sprites: &[Sprite]) -> (Vec<u8>, Vec<SpriteDef>) {
	let mut data = Vec::new();
	let mut sprite_defs = Vec::new();

	let mut sorted_sprites = sprites.to_vec();
	sorted_sprites.sort();

	for sprite in sorted_sprites {
		let mut bits = BitWriter::new();
		for pixel in &sprite.pixels {
			bits.write_bits(*pixel, sprite.bpp as usize);
		}
		bits.end();
		let pixel_data = bits.bytes;

		let byte_len = pixel_data.len();
		let mut index = 0;
		let mut offset = 0;
		let mut overlap_found = false;

		while offset + byte_len < data.len() {
			if data[offset..(offset + byte_len)] == pixel_data {
				overlap_found = true;
				break;
			} else {
				index += 1;
				offset = index * byte_len;
			}
		}

		if !overlap_found {
			if !data.is_empty() {
				index += 1;
				offset = index * byte_len;
			}
			if offset + byte_len > data.len() {
				data.resize(offset + byte_len, 0);
			}
			data.splice(offset..(offset + byte_len), pixel_data.clone());
		}

		sprite_defs.push(SpriteDef {
			index: sprite.index,
			pixel_data_index: index as u16,
			offset_x: sprite.offset_x as i16,
			offset_y: sprite.offset_y as i16,
			bpp: sprite.bpp as u8,
			width: sprite.width as u8,
			height: sprite.height as u8,
			is_quadrupled: sprite.is_quadrupled
		})
	}

	sprite_defs.sort();

	(data, sprite_defs)
}

pub fn save_sprites(sprite_defs: &[SpriteDef]) -> Result<Vec<u8>, Box<dyn Error>> {
	let mut data = Vec::new();

	for sprite_def in sprite_defs {
		for bytes in u16::to_le_bytes(sprite_def.pixel_data_index) {
			data.push(bytes);
		}

		for bytes in i16::to_le_bytes(sprite_def.offset_x) {
			data.push(bytes);
		}

		for bytes in i16::to_le_bytes(sprite_def.offset_y) {
			data.push(bytes);
		}

		let bpp = match sprite_def.bpp {
			2 => 0,
			4 => 1,
			6 => 2,
			8 => 3,
			_ => return Err("Invalid bits per pixel".into())
		} as u16;

		let width = match sprite_def.width {
			8 => 0,
			16 => 1,
			32 => 2,
			64 => 3,
			_ => return Err("Invalid width".into())
		} << 4;

		let height = match sprite_def.height {
			8 => 0,
			16 => 1,
			32 => 2,
			64 => 3,
			_ => return Err("Invalid height".into())
		} << 6;

		let is_quadrupled = if sprite_def.is_quadrupled {
			1 << 15
		} else {
			0
		};

		let props = bpp | width | height | is_quadrupled;
		for bytes in u16::to_le_bytes(props) {
			data.push(bytes);
		}
	}

	Ok(data)
}
