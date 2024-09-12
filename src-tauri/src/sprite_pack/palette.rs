use std::error::Error;
use std::cmp::Ordering;
use image::{ Rgba, RgbaImage };

use super::get_colors_in_images;

#[derive(Debug, Default, Clone, Copy, Eq, serde::Serialize)]
pub struct Color {
	pub r: u8,
	pub g: u8,
	pub b: u8,
	pub a: u8
}

impl Color {
	pub fn as_rgba(&self) -> Rgba::<u8> {
		Rgba([self.r, self.g, self.b, self.a])
	}

	pub fn from_rgba(rgba: &Rgba::<u8>) -> Self {
		let alpha = rgba[3];
		if alpha < 255 {
			Color{ r: 0, g: 0, b: 0, a: 0 }
		} else {
			Color{
				r: ((((rgba[0] as u16) << 7) & 0x7c00) >> 7) as u8,
				g: ((((rgba[1] as u16) << 2) & 0x03e0) >> 2) as u8,
				b: ((((rgba[2] as u16) >> 3) & 0x001f) << 3) as u8,
				a: 255
			}
		}
	}

	pub fn as_word(&self) -> u16 {
		let a = if self.a == 0 { 1 << 15 } else { 0 };
		let r = (self.r as u16) << 7;
		let g = (self.g as u16) << 2;
		let b = (self.b as u16) >> 3;
		a | r | g | b
	}

	pub fn from_word(word: u16) -> Self {
		let r = ((word & 0x7c00) >> 7) as u8;
		let g = ((word & 0x03e0) >> 2) as u8;
		let b = ((word & 0x001f) << 3) as u8;
		let a = if (word >> 15) > 0 { 0 } else { 255 };
		Color{ r, g, b, a }
	}

	pub fn from_bytes(data: [u8; 2]) -> Self {
		let word = u16::from_le_bytes(data);
		let r = ((word & 0x7c00) >> 7) as u8;
		let g = ((word & 0x03e0) >> 2) as u8;
		let b = ((word & 0x001f) << 3) as u8;
		let a = if (word >> 15) > 0 { 0 } else { 255 };
		Self{ r, g, b, a }
	}

	pub fn as_bytes(&self) -> [u8; 2] {
		let a = if self.a == 0 { 1 << 15 } else { 0 };
		let r = (self.r as u16) << 7;
		let g = (self.g as u16) << 2;
		let b = (self.b as u16) >> 3;
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

pub fn generate_palettes(images: &[Vec<RgbaImage>]) -> Vec<Color> {
	let mut palettes: Vec<Vec<Color>> = Vec::new();

	for subimages in images {
		let colors = get_colors_in_images(subimages);
		let mut palette_exists = false;
		for palette in palettes.iter_mut() {
			if palette.starts_with(&colors) {
				palette_exists = true;
				break;
			} else if colors.starts_with(palette) {
				palette.clone_from(&colors);
				palette_exists = true;
				break;
			}
		}
		if !palette_exists {
			palettes.push(colors);
		}
	}

	palettes.sort();
	palettes.dedup();

	for palette in palettes.iter_mut() {
		while palette.len() % 4 != 0 {
			palette.push(Color::default());
		}
	}

	palettes.into_iter().flatten().collect()
}
