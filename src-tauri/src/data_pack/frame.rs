use std::error::Error;

use super::EntityId;
use crate::data_view::DataView;

#[derive(Clone, Debug, serde::Serialize)]
pub enum FrameLayerType {
	Unknown,
	Face,
	Npc,
	Body,
	HeadAccessory,
	FaceAccessory,
	BodyAccessory,
	DirtClouds,
	HandAccessory
}

#[derive(Default, Clone, Debug, serde::Serialize)]
pub struct FrameLayer {
	pub x: Option<i16>,
	pub y: Option<i16>,
	pub subimage_index: Option<u16>,
	pub layer_type: Option<FrameLayerType>,
	pub image_id: Option<EntityId>,
	pub unknown1: Option<u16>,
	pub unknown2: Option<u16>,
	pub unknown3: Option<u16>
}

#[derive(Clone, serde::Serialize)]
pub enum Frame {
	Implicit,
	Explicit(Vec<FrameLayer>)
}

#[derive(Clone, serde::Serialize)]
pub struct FrameGroup {
	pub frames: Vec<Frame>
}

pub fn get_frame_layers(data: &DataView) -> Vec<FrameLayer> {
	let mut frame_layers = Vec::new();

	let mut i = 0;
	while i + 2 <= data.len() {
		let mut layer = FrameLayer::default();

		let bitmask = data.get_u16(i);
		let mut flags = [false; 16];
		for (i, flag) in flags.iter_mut().enumerate() {
			*flag = bitmask & (1 << i) != 0;
		}

		if flags[0] {
			i += 2;
			if i < data.len() {
				layer.x = Some(data.get_i16(i));
			}
		}

		if flags[1] {
			i += 2;
			if i < data.len() {
				layer.y = Some(data.get_i16(i));
			}
		}

		if flags[2] {
			i += 2;
			if i < data.len() {
				layer.subimage_index = Some(data.get_u16(i));
			}
		}

		if flags[4] {
			i += 2;
			if i < data.len() {
				layer.unknown1 = Some(data.get_u16(i));
			}
		}

		if flags[5] {
			i += 2;
			if i < data.len() {
				layer.unknown2 = Some(data.get_u16(i));
			}
		}

		if flags[8] {
			i += 2;
			if i < data.len() {
				layer.unknown3 = Some(data.get_u16(i));
			}
		}

		if flags[9] {
			i += 2;
			if i < data.len() {
				layer.layer_type = Some(match data.get_u16(i) {
					1 => FrameLayerType::Face,
					2 => FrameLayerType::Npc,
					3 => FrameLayerType::Body,
					4 => FrameLayerType::HeadAccessory,
					6 => FrameLayerType::FaceAccessory,
					8 => FrameLayerType::BodyAccessory,
					9 => FrameLayerType::DirtClouds,
					10 => FrameLayerType::HandAccessory,
					_ => FrameLayerType::Unknown,
				});
			}
		}

		if flags[10] {
			i += 2;
			if i < data.len() {
				layer.image_id = Some(EntityId::new(data.get_u16(i)));
			}
		}

		frame_layers.push(layer);

		i += 2;
	}

	frame_layers
}

pub fn get_frame_groups(data: &DataView, layers: Vec<FrameLayer>) -> Vec<FrameGroup> {
	let mut all_frames = Vec::new();
	let mut i = 0;
	while i + 4 <= data.len() {
		let layer_index = data.get_u16(i) as usize;
		let frame = if layer_index == 0xffff {
			Frame::Implicit
		} else {
			let num_layers = data.get_u16(i + 2) as usize;
			Frame::Explicit(layers[layer_index..(layer_index + num_layers)].to_vec())
		};
		all_frames.push(frame);
		i += 4;
	}

	let mut frame_groups = Vec::new();
	for i in 0..(all_frames.len() / 53) {
		let frames = all_frames[(i*53)..(i*53+53)].to_vec();
		frame_groups.push(FrameGroup { frames })
	}

	frame_groups
}

pub fn save_frame_groups(frame_groups: &[FrameGroup]) -> Result<(Vec<u8>, Vec<u8>, Vec<u8>), Box<dyn Error>> {
	let mut frame_layer_offsets = Vec::new();
	let mut frame_layer_data = Vec::new();
	let mut frame_group_data = Vec::new();
	let mut frame_layer_index = 0_u16;

	for frame_group in frame_groups {
		for frame in &frame_group.frames {
			match frame {
				Frame::Implicit => {
					frame_group_data.extend_from_slice(&[0xFF, 0xFF, 0, 0]);
				},
				Frame::Explicit(frame_layers) => {
					frame_group_data.extend_from_slice(&frame_layer_index.to_le_bytes());
					frame_group_data.extend_from_slice(&(frame_layers.len() as u16).to_le_bytes());
					for frame_layer in frame_layers {
						frame_layer_offsets.extend_from_slice(&(frame_layer_data.len() as u32 / 2).to_le_bytes());
						let this_layer_data = save_frame_layer(frame_layer)?;
						frame_layer_data.extend_from_slice(&this_layer_data);
						frame_layer_index += 1;
					}
				}
			}
		}
	}

	frame_layer_offsets.extend_from_slice(&(frame_layer_data.len() as u32 / 2).to_le_bytes());

	Ok((frame_layer_offsets, frame_layer_data, frame_group_data))
}

fn save_frame_layer(frame_layer: &FrameLayer) -> Result<Vec<u8>, Box<dyn Error>> {
	let mut bitmask: u16 = 0;
	let mut data = vec![0, 0];

	if let Some(x) = frame_layer.x {
		bitmask |= 1 << 0;
		data.extend_from_slice(&x.to_le_bytes());
	}

	if let Some(y) = frame_layer.y {
		bitmask |= 1 << 1;
		data.extend_from_slice(&y.to_le_bytes());
	}

	if let Some(subimage_index) = frame_layer.subimage_index {
		bitmask |= 1 << 2;
		data.extend_from_slice(&subimage_index.to_le_bytes());
	}

	if let Some(unknown1) = frame_layer.unknown1 {
		bitmask |= 1 << 4;
		data.extend_from_slice(&unknown1.to_le_bytes());
	}

	if let Some(unknown2) = frame_layer.unknown2 {
		bitmask |= 1 << 5;
		data.extend_from_slice(&unknown2.to_le_bytes());
	}

	if let Some(unknown3) = frame_layer.unknown3 {
		bitmask |= 1 << 8;
		data.extend_from_slice(&unknown3.to_le_bytes());
	}

	if let Some(layer_type) = &frame_layer.layer_type {
		bitmask |= 1 << 9;
		let layer_id: u16 = match layer_type {
			FrameLayerType::Face => 1,
			FrameLayerType::Npc => 2,
			FrameLayerType::Body => 3,
			FrameLayerType::HeadAccessory => 4,
			FrameLayerType::FaceAccessory => 6,
			FrameLayerType::BodyAccessory => 8,
			FrameLayerType::DirtClouds => 9,
			FrameLayerType::HandAccessory => 10,
			FrameLayerType::Unknown => 0,
		};
		data.extend_from_slice(&layer_id.to_le_bytes());
	}

	if let Some(image_id) = &frame_layer.image_id {
		bitmask |= 1 << 10;
		data.extend_from_slice(&image_id.to_word().to_le_bytes());
	}

	data.splice(0..2, bitmask.to_le_bytes());

	Ok(data)
}
