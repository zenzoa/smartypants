use crate::text::{ Text, FontState };

#[derive(serde::Serialize)]
pub struct DataView {
	pub data: Vec<u8>
}

impl DataView {
	pub fn new(data: &[u8]) -> DataView {
		DataView { data: data.to_owned() }
	}

	pub fn len(&self) -> usize {
		self.data.len()
	}

	pub fn chunk(&self, start: usize, size: usize) -> DataView {
		DataView { data: self.data[start..(start+size)].to_owned() }
	}

	pub fn get_u8(&self, i: usize) -> u8 {
		self.data[i]
	}

	pub fn get_u16(&self, i: usize) -> u16 {
		u16::from_le_bytes([self.data[i], self.data[i+1]])
	}

	pub fn get_i16(&self, i: usize) -> i16 {
		i16::from_le_bytes([self.data[i], self.data[i+1]])
	}

	pub fn get_u32(&self, i: usize) -> u32 {
		u32::from_le_bytes([self.data[i], self.data[i+1], self.data[i+2], self.data[i+3]])
	}

	pub fn get_bits(&self, i: usize, len: usize) -> Vec<u8> {
		let mut bits = Vec::new();
		for j in 0..len {
			let byte = self.data[i+j];
			for b in (0..8).rev() {
				let bit = (byte >> b) & 1;
				bits.push(bit);
			}
		}
		bits
	}

	pub fn get_text(&self, font_state: &FontState, i: usize, len: usize) -> Text {
		let mut words: Vec<u16> = Vec::new();
		for j in 0..len {
			if i + j*2 < self.len() {
				let word = self.get_u16(i + j*2);
				if word > 0 {
					words.push(word);
				}
			}
		}
		Text::from_data(font_state, &words)
	}

	pub fn find_bytes(&self, bytes: &[u8]) -> Option<usize> {
		(0..self.data.len()).find(|&i| self.data[i..].starts_with(bytes))
	}
}

pub fn bytes_to_words(bytes: &[u8]) -> Vec<u16> {
	let mut words = Vec::new();
	let mut i = 0;
	while i + 1 < bytes.len() {
		let word = u16::from_le_bytes([bytes[i], bytes[i+1]]);
		words.push(word);
		i += 2;
	}
	words
}

pub fn words_to_bytes(words: &[u16]) -> Vec<u8> {
	let mut bytes: Vec<u8> = Vec::new();
	for word in words {
		for byte in u16::to_le_bytes(*word) {
			bytes.push(byte)
		}
	}
	bytes
}

pub fn resize_words(words: &[u16], len: usize) -> Vec<u16> {
	let mut word_vec = words.to_vec();
	word_vec.resize(len, 0);
	word_vec
}

pub struct BitWriter {
	pub bytes: Vec<u8>,
	pub byte_in_progress: u8,
	pub shift: usize
}

impl BitWriter {
	pub fn new() -> BitWriter {
		BitWriter { bytes: Vec::new(), byte_in_progress: 0, shift: 0 }
	}

	// pub fn from(bytes: Vec<u8>) -> BitWriter {
	// 	BitWriter { bytes, byte_in_progress: 0, shift: 0 }
	// }

	// pub fn write_byte(&mut self, byte: u8) {
	// 	self.bytes.push(byte);
	// }

	pub fn write_bit(&mut self, bit: bool) {
		self.byte_in_progress <<= 1;
		if bit {
			self.byte_in_progress |= 0x1;
		}
		self.shift += 1;
		if self.shift == 8 {
			self.bytes.push(self.byte_in_progress);
			self.byte_in_progress = 0;
			self.shift = 0;
		}
	}

	pub fn write_bits(&mut self, mut value: u32, mut num_bits: usize) {
		if num_bits > 32 { num_bits = 32 }
		let mask: u32 = (1 << (num_bits - 1)) as u32;
		for _ in 0..num_bits {
			self.write_bit(value & mask != 0);
			value <<= 1;
		}
	}

	pub fn end(&mut self) {
		if self.shift != 0 {
			self.write_bits(0, 8 - self.shift);
		}
	}
}
