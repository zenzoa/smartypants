use std::error::Error;
use image::Rgba;
use crate::data_view::DataView;

#[derive(Clone, Copy, serde::Serialize)]
pub struct Color {
	pub r: u8,
	pub g: u8,
	pub b: u8,
	pub a: u8
}

impl Color {
	pub fn into_rgba(&self) -> Rgba::<u8> {
		Rgba([self.r, self.g, self.b, self.a])
	}
}

pub fn get_palettes(data: &DataView) -> Result<Vec<Color>, Box<dyn Error>> {
	let mut colors = Vec::new();

	for i in 0..(data.len()/2) {
		let word = data.get_u16(i*2);
		let r = ((word & 0x7c00) >> 7) as u8;
		let g = ((word & 0x03e0) >> 2) as u8;
		let b = ((word & 0x001f) << 3) as u8;
		let a = if (word >> 15) > 0 { 0 } else { 255 };
		colors.push(Color{ r, g, b, a });
	}

	Ok(colors)
}
