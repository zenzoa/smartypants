// use super::EntityId;
use crate::data_view::DataView;

#[derive(Clone, serde::Serialize)]
pub struct ParticleEmitter {
	pub data: Vec<u16>
}

pub fn get_particle_emitters(data: &DataView) -> Vec<ParticleEmitter> {
	let mut particle_emitters: Vec<ParticleEmitter> = Vec::new();

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
