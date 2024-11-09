use std::error::Error;
use std::num::Wrapping;

use serde::Serialize;

use md5::{ Md5, Digest };

use tauri::{ AppHandle, State, Manager, Emitter };

use crate::{ DataState, BinSize, update_window_title };
use crate::data_view::{ DataView, bytes_to_words };
use crate::data_pack::{ DataPack, get_data_pack, save_data_pack };
use crate::sprite_pack::SpritePack;
use crate::file::set_file_modified;

#[derive(Clone, Serialize)]
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
	md5: [u8; 16]
}

#[derive(Clone, Serialize)]
pub enum CardType {
	TamaSmaCard,
	PromoTreasure,
	PromoItem,
	Unknown
}

#[derive(Clone)]
pub struct TamaSmaCard {
	pub header: CardHeader,
	pub data_pack: DataPack,
	pub sprite_pack: SpritePack
}

pub fn read_card(handle: &AppHandle, data: &DataView) -> Result<TamaSmaCard, Box<dyn Error>> {
	let header = read_card_header(data)?;
	let (data_pack, sprite_pack) = read_card_packs(handle, &data.chunk(0x1000, data.len() - 0x1000))?;
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

	let mut md5 = [0; 16];
	for (i, byte) in md5.iter_mut().enumerate() {
		*byte = data.get_u8(i+64);
	}

	Ok(CardHeader {
		sector_count,
		checksum,
		device_ids,
		vendor_id,
		product_id,
		card_type,
		card_id,
		year,
		month,
		day,
		revision,
		md5
	})
}

pub fn read_card_packs(handle: &AppHandle, data: &DataView) -> Result<(DataPack, SpritePack), Box<dyn Error>> {
	if data.len() < 68 {
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
				0 => data_pack_opt = Some(get_data_pack(handle, &pack_data)?),
				1 => sprite_pack_opt = Some(SpritePack::from_data(&pack_data)?),
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

pub fn save_card(handle: &AppHandle) -> Result<Vec<u8>, Box<dyn Error>> {
	let data_state: State<DataState> = handle.state();

	let data_pack_offset = 68;
	let data_pack_opt = data_state.data_pack.lock().unwrap();
	let data_pack = data_pack_opt.as_ref().ok_or("Unable to save Sma Card: missing data pack")?;
	let mut data_pack_data = save_data_pack(data_pack, data_pack_offset)?;
	data_pack_data.extend_from_slice(&[0, 0]);

	let mut sprite_pack_opt = data_state.sprite_pack.lock().unwrap();
	let sprite_pack = sprite_pack_opt.as_mut().ok_or("Unable to save Sma Card: missing sprite pack")?;
	let sprite_pack_data = sprite_pack.as_bytes()?;

	let mut sprite_pack_offset = data_pack_offset + data_pack_data.len();
	while sprite_pack_offset % 32 != 0 {
		sprite_pack_offset += 1;
	}
	let padded_data_pack_size = sprite_pack_offset - data_pack_offset;
	data_pack_data.resize(padded_data_pack_size, 0);

	let mut pack_summary = 0x3232_u16.to_le_bytes().to_vec();
	pack_summary.extend_from_slice(&4_u16.to_le_bytes());

	pack_summary.extend_from_slice(&[0, 0, 0, 0]);
	pack_summary.extend_from_slice(&(data_pack_offset as u32).to_le_bytes());
	pack_summary.extend_from_slice(&(data_pack_data.len() as u32).to_le_bytes());
	pack_summary.extend_from_slice(&(data_pack_data.len() as u32 - 2).to_le_bytes());

	pack_summary.extend_from_slice(&[0, 0, 0, 0]);
	pack_summary.extend_from_slice(&(sprite_pack_offset as u32).to_le_bytes());
	pack_summary.extend_from_slice(&(sprite_pack_data.len() as u32).to_le_bytes());
	pack_summary.extend_from_slice(&(sprite_pack_data.len() as u32).to_le_bytes());

	pack_summary.extend_from_slice(&[0; 32]);

	let pack_data = [pack_summary, data_pack_data, sprite_pack_data].concat();

	let mut header_opt = data_state.card_header.lock().unwrap();
	let header = header_opt.as_mut().ok_or("Unable to save Sma Card: missing header")?;
	header.sector_count = match data_state.bin_size.lock().unwrap().as_ref().ok_or("Undefined card size")? {
		BinSize::Card128KB => 31,
		BinSize::Card1MB => 255,
		BinSize::Card2MB => 511,
		_ => return Err("Invalid TamaSma card size".into())
	};
	let header_data = save_card_header(header)?;

	let mut data = [header_data, pack_data].concat();

	let padding_len = match data_state.bin_size.lock().unwrap().as_ref().ok_or("Undefined card size")? {
		BinSize::Card128KB => 0x20000 - 16,
		BinSize::Card1MB => 0x100000 - 16,
		BinSize::Card2MB => 0x200000 - 16,
		_ => return Err("Invalid TamaSma card size".into())
	};
	if padding_len < data.len() {
		return Err("Data is too large to fit onto Sma Card".into());
	}
	let padding = vec![0_u8; padding_len - data.len()];
	data.extend_from_slice(&padding);

	data.extend_from_slice(&[50, 132, 171, 86, 34, 17, 220, 254, 142, 107, 85, 255, 181, 16, 127, 51]);

	let checksum = calc_checksum(&data[1000..]);
	data.splice(2..4, checksum.to_le_bytes());

	let mut hasher = Md5::new();
	hasher.update(&data[0..64]);
	let md5_result = hasher.finalize();
	data.splice(64..80, md5_result);

	Ok(data)
}

pub fn save_card_header(header: &CardHeader) -> Result<Vec<u8>, Box<dyn Error>> {
	let mut data: Vec<u8> = Vec::new();

	data.extend_from_slice(&header.sector_count.to_le_bytes());
	data.extend_from_slice(&header.checksum.to_le_bytes());
	data.extend_from_slice(&header.device_ids[0].to_le_bytes());
	data.extend_from_slice(&header.device_ids[1].to_le_bytes());
	data.extend_from_slice(&header.device_ids[2].to_le_bytes());
	data.extend_from_slice(header.vendor_id.as_bytes());
	data.extend_from_slice(header.product_id.as_bytes());

	let card_type = match header.card_type {
		CardType::TamaSmaCard => 0_u16,
		CardType::PromoTreasure => 1_u16,
		CardType::PromoItem => 2_u16,
		_ => { return Err("Invalid card type".into()); }
	};
	data.extend_from_slice(&card_type.to_le_bytes());

	data.extend_from_slice(&header.card_id.to_le_bytes());

	data.extend_from_slice(&[0, 0]);

	data.extend_from_slice(&header.year.to_le_bytes());
	data.extend_from_slice(&header.month.to_le_bytes());
	data.extend_from_slice(&header.day.to_le_bytes());
	data.extend_from_slice(&header.revision.to_le_bytes());

	data.extend_from_slice(&[0, 0]);

	data.extend_from_slice(&header.md5);

	let padding_len = 0x1000 - data.len();
	let padding = vec![0_u8; padding_len];
	data.extend_from_slice(&padding);

	Ok(data)
}

fn calc_checksum(data: &[u8]) -> u16 {
	let words = bytes_to_words(data);
	let mut checksum = Wrapping(0_u16);
	for word in words {
		checksum += Wrapping(word);
	}
	checksum.0
}

#[tauri::command]
pub fn clear_device_ids(handle: AppHandle) {
	let data_state: State<DataState> = handle.state();
	let mut header_opt = data_state.card_header.lock().unwrap();
	if let Some(header) = header_opt.as_mut() {
		header.device_ids = [0, 0, 0];
		set_file_modified(&handle, true);
		update_window_title(&handle);
		handle.emit("update_card_header", header.clone()).unwrap();
	}
}
