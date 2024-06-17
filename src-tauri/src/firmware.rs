use std::error::Error;

use crate::DataView;
use crate::data_pack::{ DataPack, get_data_pack };
use crate::sprite_pack::{ SpritePack, get_sprite_pack };

#[derive(Clone, serde::Serialize)]
pub struct Firmware {
	pub data_pack: DataPack,
	pub sprite_pack: SpritePack
}

pub fn read_firmware(data: &DataView) -> Result<Firmware, Box<dyn Error>> {
	let data_pack = get_data_pack(&data.chunk(0x6CE000, 0x730000 - 0x6CE000))?;
	let sprite_pack = get_sprite_pack(&data.chunk(0x730000, data.len() - 0x730000))?;
	Ok(Firmware { data_pack, sprite_pack })
}
