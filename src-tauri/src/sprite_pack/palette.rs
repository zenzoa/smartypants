use std::error::Error;
use image::Rgba;
use crate::data_view::{ DataView, words_to_bytes };

#[derive(Clone, Copy, PartialEq, serde::Serialize)]
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

	pub fn from_rgba(rgba: Rgba::<u8>) -> Color {
		Color{ r: rgba[0], g: rgba[1], b: rgba[2], a: rgba[3] }
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

pub fn save_palettes(colors: &[Color]) -> Result<Vec<u8>, Box<dyn Error>> {
	let mut words: Vec<u16> = Vec::new();
	for color in colors {
		let a = if color.a == 0 { 1 << 15 } else { 0 };
		let r = (color.r as u16) << 7;
		let g = (color.g as u16) << 2;
		let b = (color.b as u16) >> 3;
		let word = a | r | g | b;
		words.push(word);
	}
	let data = words_to_bytes(&words);
	Ok(data)
}
