use std::error::Error;

use crate::data_view::{ DataView, words_to_bytes };

#[derive(Clone, serde::Serialize)]
pub struct ParticleEmitter {
	pub data: Vec<u16>
}

pub fn get_particle_emitters(data: &DataView) -> Vec<ParticleEmitter> {
	let mut particle_emitters = Vec::new();

	let row_count = data.len() / 66;
	for i in 0..row_count {
		let mut particle_emitter_data: Vec<u16> = Vec::new();
		for j in 0..33 {
			particle_emitter_data.push(data.get_u16(i*66 + j*2));
		}
		particle_emitters.push(ParticleEmitter { data: particle_emitter_data });
	}

	particle_emitters
}

pub fn save_particle_emitters(particle_emitters: &[ParticleEmitter]) -> Result<Vec<u8>, Box<dyn Error>> {
	let mut data = Vec::new();

	for particle_emitter in particle_emitters {
		data.extend_from_slice(&words_to_bytes(&particle_emitter.data));
	}

	Ok(data)
}
