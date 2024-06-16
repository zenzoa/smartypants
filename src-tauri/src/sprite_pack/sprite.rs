use std::error::Error;
use crate::data_view::DataView;

#[derive(Clone, serde::Serialize)]
pub struct Sprite {
	pub pixel_data_index: u16,
	pub offset_x: u16,
	pub offset_y: u16,
	pub bits_per_pixel: u8,
	pub is_flipped: bool, // unused on Smart
	pub width: u8,
	pub height: u8,
	pub palette_bank: u16,
	pub draw_depth: u16, // typically not set
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
		let is_flipped = ((props & 0x000c) >> 2) > 0;
		let width = [8, 16, 32, 64][((props & 0x0030) >> 4) as usize] as u8;
		let height = [8, 16, 32, 64][((props & 0x00c0) >> 6) as usize] as u8;
		let palette_bank = (props & 0x0f00) >> 8;
		let draw_depth = (props & 0x3000) >> 12;
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
