use std::error::Error;
use serde::{ Serialize, Deserialize };

use crate::data_view::{ DataView, BitWriter };

pub struct SpriteDef {
	pub pixel_data: Vec<u8>,
	pub pixel_data_offset: usize,
	pub pixel_data_index: u16,
	pub offset_x: i16,
	pub offset_y: i16,
	pub bpp: u8,
	pub width: u8,
	pub height: u8,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Sprite {
	pub width: u32,
	pub height: u32,
	pub bpp: u32,
	pub offset_x: i32,
	pub offset_y: i32,
	pub pixels: Vec<u32>
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
		let width = [8, 16, 32, 64][((props & 0x0030) >> 4) as usize];
		let height = [8, 16, 32, 64][((props & 0x00c0) >> 6) as usize];
		let _palette_bank = (props & 0x0f00) >> 8;			// unused on Smart
		let _draw_depth = (props & 0x3000) >> 12;			// unused on Smart
		let _blend_enabled = ((props & 0x4000) >> 14) > 0;	// unused on Smart
		let _is_quadrupled = ((props & 0x8000) >> 15) > 0;	// unused on Smart

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
			width: width as u32,
			height: height as u32,
			bpp: bpp as u32,
			offset_x,
			offset_y,
			pixels
		});

		i += 8;
	}

	Ok(sprites)
}

pub fn save_sprites(sprite_defs: &[SpriteDef]) -> Result<Vec<u8>, Box<dyn Error>> {
	let mut data = Vec::new();

	let mut pixel_data_parts: Vec<Vec<u8>> = Vec::new();
	let mut pixel_data_len: usize = 0;

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

		let props = bpp | width | height;
		for bytes in u16::to_le_bytes(props) {
			data.push(bytes);
		}

		let mut bits = BitWriter::new();
		for pixel in &sprite_def.pixel_data {
			bits.write_bits(*pixel as u32, sprite_def.bpp as usize);
		}
		bits.end();
		pixel_data_parts.push(bits.bytes);

		let byte_count = (sprite_def.width as usize) * (sprite_def.height as usize) * (sprite_def.bpp as usize) / 8;
		let min_len = sprite_def.pixel_data_offset + byte_count;
		if pixel_data_len < min_len {
			pixel_data_len = min_len;
		}
	}

	Ok(data)
}

pub fn save_pixel_data(sprites: &[Sprite]) -> (Vec<u8>, Vec<SpriteDef>) {
	let mut data = Vec::new();
	let mut sprite_defs = Vec::new();

	for sprite in sprites {
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

		while offset+byte_len < data.len() {
			if data[offset..offset+byte_len] == pixel_data {
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
			if offset+byte_len > data.len() {
				data.resize(offset+byte_len, 0);
			}
			data.splice(offset..offset+byte_len, pixel_data.clone());
		}

		sprite_defs.push(SpriteDef {
			pixel_data,
			pixel_data_offset: offset,
			pixel_data_index: index as u16,
			offset_x: sprite.offset_x as i16,
			offset_y: sprite.offset_y as i16,
			bpp: sprite.bpp as u8,
			width: sprite.width as u8,
			height: sprite.height as u8
		})
	}

	(data, sprite_defs)
}
