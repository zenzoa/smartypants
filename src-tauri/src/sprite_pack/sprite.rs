use std::error::Error;
use crate::data_view::{ DataView, BitWriter, words_to_bytes };

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

pub fn get_sprites(def_data: &DataView, pixel_data: &DataView) -> Result<Vec<Sprite>, Box<dyn Error>> {
	let mut sprites = Vec::new();

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

	Ok(sprites)
}

pub fn save_sprites(sprites: &[Sprite]) -> Result<(Vec<u8>, Vec<u8>), Box<dyn Error>> {
	let mut sprite_defs: Vec<u16> = Vec::new();
	let mut pixel_data_len: usize = 0;

	for sprite in sprites {
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
		} << 4 as u16;
		let height = match sprite.height {
			8 => 0,
			16 => 1,
			32 => 2,
			64 => 3,
			_ => return Err("Invalid height".into())
		} << 6 as u16;
		let palette_bank = (sprite.palette_bank as u16) << 8;
		let draw_depth = (sprite.draw_depth as u16) << 12;
		let blend_enabled = if sprite.blend_enabled { 1 << 14 } else { 0 };
		let is_quadrupled = if sprite.is_quadrupled { 1 << 15 } else { 0 };

		let props = bits_per_pixel | is_flipped | width | height | palette_bank | draw_depth | blend_enabled | is_quadrupled;
		sprite_defs.push(props);

		let byte_count = (sprite.width as usize) * (sprite.height as usize) * (sprite.bits_per_pixel as usize) / 8;
		let min_len = sprite.pixel_data_offset + byte_count;
		if pixel_data_len < min_len {
			pixel_data_len = min_len;
		}
	}

	let mut pixel_data = vec![0; pixel_data_len];
	for sprite in sprites {
		let mut bits = BitWriter::new();
		let bpp = sprite.bits_per_pixel;
		for pixel in &sprite.pixels {
			bits.write_bits(*pixel as u32, bpp as usize);
		}
		bits.end();
		for (i, byte) in bits.bytes.iter().enumerate() {
			pixel_data[sprite.pixel_data_offset + i] = *byte;
		}
	}

	Ok((words_to_bytes(&sprite_defs), pixel_data))
}
