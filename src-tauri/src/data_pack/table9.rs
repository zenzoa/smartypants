use crate::data_view::DataView;

pub fn get_entity_offsets(data: &DataView) -> (Vec<usize>, Vec<usize>) {
	let mut offsets = Vec::new();
	for i in 0..(data.len()/2) {
		let offset = data.get_u16(i*2) as usize * 2;
		offsets.push(offset);
	}

	let mut sizes = Vec::new();
	if offsets.len() > 0 {
		for i in 0..(offsets.len() - 1) {
			sizes.push(offsets[i+1] - offsets[i]);
		}
		let _ = offsets.pop();
	}

	(offsets, sizes)
}

pub fn get_entities(data: &DataView, offsets: Vec<usize>, sizes: Vec<usize>) -> Vec<Vec<u16>> {
	let mut entities = Vec::new();

	for i in 0..offsets.len() {
		let entity_data = data.chunk(offsets[i], sizes[i]);
		let mut entity_bytes = Vec::new();
		for j in 0..(entity_data.len()/2) {
			entity_bytes.push(entity_data.get_u16(j*2));
		}
		entities.push(entity_bytes)
	}

	entities
}
