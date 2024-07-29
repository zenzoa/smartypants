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
	pub prop_bitmask: u16,
	pub x: Option<i16>,
	pub y: Option<i16>,
	pub subimage_index: Option<u16>,
	pub layer_type: Option<FrameLayerType>,
	pub image_id: Option<EntityId>,
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

		if bitmask & 0x1 > 0 {
			i += 2;
			if i < data.len() {
				layer.x = Some(data.get_i16(i));
			}
		}

		if bitmask & 0x2 > 0 {
			i += 2;
			if i < data.len() {
				layer.y = Some(data.get_i16(i));
			}
		}

		if bitmask & 0x4 > 0 {
			i += 2;
			if i < data.len() {
				layer.subimage_index = Some(data.get_u16(i));
			}
		}

		if bitmask & 0x8 > 0 { i += 2; }
		if bitmask & 0x10 > 0 { i += 2; }
		if bitmask & 0x20 > 0 { i += 2; }
		if bitmask & 0x40 > 0 { i += 2; }
		if bitmask & 0x80 > 0 { i += 2; }
		if bitmask & 0x100 > 0 { i += 2; }

		if bitmask & 0x200 > 0 {
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

		if bitmask & 0x400 > 0 {
			i += 2;
			if i < data.len() {
				layer.image_id = Some(EntityId::new(data.get_u16(i)));
				if layer.subimage_index.is_none() {
					layer.subimage_index = Some(0);
				}
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
