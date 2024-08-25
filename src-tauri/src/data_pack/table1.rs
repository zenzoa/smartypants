use std::error::Error;

use crate::data_view::{ DataView, words_to_bytes };

pub fn get_entity_offsets(data: &DataView) -> Vec<usize> {
	let mut offsets = Vec::new();
	for i in 0..(data.len()/2) {
		let offset = data.get_u16(i*2) as usize * 2;
		offsets.push(offset);
	}

	println!("\noffsets: {:?}", &offsets);

	offsets
}

pub fn get_entities(data: &DataView, offsets: Vec<usize>) -> Vec<Vec<u16>> {
	let mut entities = Vec::new();

	for i in 0..offsets.len() {
		let size = if i+1 < offsets.len() {
			offsets[i+1] - offsets[i]
		} else {
			data.len() - offsets[i]
		};
		let entity_data = data.chunk(offsets[i], size);
		let mut entity_words = Vec::new();
		for j in 0..(entity_data.len()/2) {
			entity_words.push(entity_data.get_u16(j*2));
		}
		println!("\nentity {}: {:?}", i, &entity_words.iter().map(|x| format!("0x{:x}", x)).collect::<Vec<String>>());
		entities.push(entity_words)
	}


	entities
}

pub fn save_entities(entities: &[Vec<u16>]) -> Result<(Vec<u8>, Vec<u8>), Box<dyn Error>> {
	let mut offsets = Vec::new();
	let mut data = Vec::new();

	for entity in entities {
		offsets.extend_from_slice(&(data.len() as u16 / 2).to_le_bytes());
		data.extend_from_slice(&words_to_bytes(entity));
	}

	Ok((offsets, data))
}
