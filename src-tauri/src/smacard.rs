use std::error::Error;

use crate::DataView;
use crate::data_pack::{ DataPack, get_data_pack };
use crate::sprite_pack::{ SpritePack, get_sprite_pack };

#[derive(Clone, serde::Serialize)]
pub struct CardHeader {
	sector_count: u16,
	checksum: u16,
	device_ids: [u32; 3],
	vendor_id: String,
	product_id: String,
	card_type: CardType,
	card_id: u16,
	year: u16,
	month: u16,
	day: u16,
	revision: u16,
	md5: String
}

#[derive(Clone, serde::Serialize)]
pub enum CardType {
	TamaSmaCard,
	PromoTreasure,
	PromoItem,
	Unknown
}

#[derive(Clone, serde::Serialize)]
pub struct TamaSmaCard {
	pub header: CardHeader,
	pub data_pack: DataPack,
	pub sprite_pack: SpritePack
}

pub fn read_card(data: &DataView) -> Result<TamaSmaCard, Box<dyn Error>> {
	let header = read_card_header(&data)?;
	let (data_pack, sprite_pack) = read_card_packs(&data.chunk(0x1000, data.len() - 0x1000))?;
	Ok(TamaSmaCard { header, data_pack, sprite_pack })
}

pub fn read_card_header(data: &DataView) -> Result<CardHeader, Box<dyn Error>> {
	if data.len() < 80 {
		return Err("Unable to read card data: too short for header".into());
	}

	let sector_count = data.get_u16(0);

	let checksum = data.get_u16(2);

	let device_ids = [
		data.get_u32(4),
		data.get_u32(8),
		data.get_u32(12)
	];

	let mut vendor_id = String::new();
	for i in 0..16 {
		vendor_id.push(data.get_u8(i+16).into());
	}

	let mut product_id = String::new();
	for i in 0..16 {
		product_id.push(data.get_u8(i+32).into());
	}

	let card_type = match data.get_u16(48) {
		0 => CardType::TamaSmaCard,
		1 => CardType::PromoTreasure,
		2 => CardType::PromoItem,
		_ => CardType::Unknown
	};

	let card_id = data.get_u16(50);

	let year = data.get_u16(54);
	let month = data.get_u16(56);
	let day = data.get_u16(58);
	let revision = data.get_u16(60);

	let mut md5 = String::new();
	for i in 0..16 {
		md5.push_str(&format!("{:02x}", data.get_u8(i+64)));
	}

	Ok(CardHeader { sector_count, checksum, device_ids, vendor_id, product_id, card_type, card_id, year, month, day, revision, md5 })
}

pub fn read_card_packs(data: &DataView) -> Result<(DataPack, SpritePack), Box<dyn Error>> {
	if data.len() < 66 {
		return Err("Unable to read card data: too short for pack info".into());
	}

	let pack_count = data.get_u16(2) as usize;
	if pack_count < 2 {
		return Err("Unable to read card data: too few packs".into());
	}

	let mut data_pack_opt: Option<DataPack> = None;
	let mut sprite_pack_opt: Option<SpritePack> = None;

	for i in 0..pack_count {
		let pack_offset = data.get_u32(i*16+8) as usize;
		let pack_size = data.get_u32(i*16+16) as usize;

		if pack_offset > 0 && pack_size > 0 {
			let pack_data = data.chunk(pack_offset, pack_size);
			match i {
				0 => {
					data_pack_opt = Some(get_data_pack(&pack_data)?);
				},
				1 => {
					sprite_pack_opt = Some(get_sprite_pack(&pack_data)?);
				},
				_ => {}
			}
		}
	}

	if let Some(data_pack) = data_pack_opt {
		if let Some(sprite_pack) = sprite_pack_opt {
			Ok((data_pack, sprite_pack))
		} else {
			Err("Unable to read card data: sprite pack not found".into())
		}
	} else {
		Err("Unable to read card data: data pack not found".into())
	}
}
