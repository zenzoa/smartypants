use std::error::Error;
use std::cmp::Ordering;
use image::Rgba;
use serde::{ Serialize, Deserialize };

#[derive(Debug, Default, Clone, Copy, Eq, Serialize, Deserialize)]
pub struct Color(u8, u8, u8, u8);

impl Color {
	pub fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
		Self(r, g, b, a)
	}

	pub fn as_vec(&self) -> Vec<u8> {
		vec![self.0, self.1, self.2, self.3]
	}

	pub fn as_rgba(&self) -> Rgba::<u8> {
		Rgba([self.0, self.1, self.2, self.3])
	}

	pub fn from_rgba(rgba: &Rgba::<u8>) -> Self {
		let alpha = rgba[3];
		if alpha < 255 {
			Self(0, 0, 0, 0)
		} else {
			Self(
				((((rgba[0] as u16) << 7) & 0x7c00) >> 7) as u8,
				((((rgba[1] as u16) << 2) & 0x03e0) >> 2) as u8,
				((((rgba[2] as u16) >> 3) & 0x001f) << 3) as u8,
				255
			)
		}
	}

	pub fn as_word(&self) -> u16 {
		let a = if self.3 == 0 { 1 << 15 } else { 0 };
		let r = (self.0 as u16) << 7;
		let g = (self.1 as u16) << 2;
		let b = (self.2 as u16) >> 3;
		a | r | g | b
	}

	pub fn from_word(word: u16) -> Self {
		let r = ((word & 0x7c00) >> 7) as u8;
		let g = ((word & 0x03e0) >> 2) as u8;
		let b = ((word & 0x001f) << 3) as u8;
		let a = if (word >> 15) > 0 { 0 } else { 255 };
		Self(r, g, b, a)
	}

	pub fn from_bytes(data: [u8; 2]) -> Self {
		let word = u16::from_le_bytes(data);
		let r = ((word & 0x7c00) >> 7) as u8;
		let g = ((word & 0x03e0) >> 2) as u8;
		let b = ((word & 0x001f) << 3) as u8;
		let a = if (word >> 15) > 0 { 0 } else { 255 };
		Self(r, g, b, a)
	}

	pub fn as_bytes(&self) -> [u8; 2] {
		let a = if self.3 == 0 { 1 << 15 } else { 0 };
		let r = (self.0 as u16) << 7;
		let g = (self.1 as u16) << 2;
		let b = (self.2 as u16) >> 3;
		let word = a | r | g | b;
		word.to_le_bytes()
	}
}

impl PartialEq for Color {
	fn eq(&self, other: &Self) -> bool {
		self.as_bytes() == other.as_bytes()
	}
}

impl Ord for Color {
	fn cmp(&self, other: &Self) -> Ordering {
		self.as_bytes().cmp(&other.as_bytes())
	}
}

impl PartialOrd for Color {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}

pub fn get_palettes(data: &[u8]) -> Result<Vec<Color>, Box<dyn Error>> {
	let mut colors = Vec::new();
	for i in 0..data.len()/2 {
		let color = Color::from_bytes([data[i*2], data[i*2+1]]);
		colors.push(color);
	}
	Ok(colors)
}

pub fn save_palettes(colors: &[Color]) -> Result<Vec<u8>, Box<dyn Error>> {
	let mut data = Vec::new();
	for color in colors {
		data.extend_from_slice(&color.as_bytes());
	}
	Ok(data)
}
