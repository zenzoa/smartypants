// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::fs;
use std::error::Error;
use std::path::PathBuf;
use std::sync::Mutex;
use std::io::Cursor;
use tauri::{ Builder, AppHandle, Manager, State };
use tauri::menu::{ Menu, Submenu, MenuItem, PredefinedMenuItem, MenuId };
use tauri::async_runtime::spawn;
use rfd::{ MessageLevel, MessageButtons, MessageDialog, FileDialog };
use regex::Regex;

mod data_view;
mod data_pack;
mod sprite_pack;
mod export;

use data_view::DataView;
use data_pack::{ DataPack, get_data_pack };
use sprite_pack::{ SpritePack, get_sprite_pack, get_image_data };
use export::export_data_to;
use export::export_images_to;

pub struct DataState {
	pub base_path: Mutex<Option<PathBuf>>,
	pub data_pack: Mutex<Option<DataPack>>,
	pub sprite_pack: Mutex<Option<SpritePack>>
}

impl DataState {
	pub fn new() -> DataState {
		DataState{
			base_path: Mutex::new(None),
			data_pack: Mutex::new(None),
			sprite_pack: Mutex::new(None)
		}
	}
}

pub struct ImageState {
	pub images: Mutex<Vec<Vec<image::RgbaImage>>>
}

impl ImageState {
	pub fn new() -> ImageState {
		ImageState{ images: Mutex::new(vec![vec![]]) }
	}
}

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

fn main() {
	Builder::default()
		.invoke_handler(tauri::generate_handler![
			open_bin,
			export_data,
			export_images,
			try_quit
		])

		.manage(DataState::new())
		.manage(ImageState::new())

		.menu(|handle| {
			Menu::with_id_and_items(handle, "main", &[
				&Submenu::with_id_and_items(handle, "file", "File", true, &[
					&MenuItem::with_id(handle, "open", "Open", true, Some("CmdOrCtrl+O"))?,
					&PredefinedMenuItem::separator(handle)?,
					&MenuItem::with_id(handle, "quit", "Quit", true, Some("CmdOrCtrl+Q"))?,
				])?,

				&Submenu::with_id_and_items(handle, "export", "Export", true, &[
					&MenuItem::with_id(handle, "export_data", "Export Data", true, None::<&str>)?,
					&MenuItem::with_id(handle, "export_images", "Export Images", true, None::<&str>)?,
				])?,

				&Submenu::with_id_and_items(handle, "help", "Help", true, &[
					&MenuItem::with_id(handle, "about", "About", true, None::<&str>)?,
				])?,
			])
		})

		.setup(|app| {
			app.on_menu_event(|handle, event| {
				let MenuId(id) = event.id();
				let handle = handle.clone();

				match id.as_str() {
					"open" => open_bin(handle),
					"quit" => try_quit(handle),
					"export_data" => export_data(handle),
					"export_images" => export_images(handle),
					"about" => handle.emit("show_about_dialog", "").unwrap(),
					_ => {}
				}
			});
			Ok(())
		})

		.register_uri_scheme_protocol("getimage", |app, request| {
			let not_found = http::Response::builder().body(Vec::new()).unwrap();

			let do_the_thing = || -> Result<http::Response<Vec<u8>>, Box<dyn Error>> {
				let re = Regex::new(r"-(\d+)-(\d+)$").unwrap();
				let uri = request.uri().path();
				let caps = re.captures(uri).ok_or("no capture groups")?;

				let image_id_str = caps.get(1).ok_or("no image id")?.as_str();
				let image_id = usize::from_str_radix(image_id_str, 10)?;
				let subimage_index_str = caps.get(2).ok_or("no subimage index")?.as_str();
				let subimage_index = usize::from_str_radix(subimage_index_str, 10)?;

				let image_state: State<ImageState> = app.state();
				let img = image_state.images.lock().unwrap()
					.get(image_id).ok_or("image not found")?
					.get(subimage_index).ok_or("subimage not found")?
					.clone();

				let mut img_data = Cursor::new(Vec::new());
				let _ = img.write_to(&mut img_data, image::ImageFormat::Png)?;

				Ok(http::Response::builder()
					.header("Content-Type", "image/png")
					.body(img_data.into_inner())?)
			};

			match do_the_thing() {
				Ok(response) => response,
				Err(why) => {
					println!("ERROR: {}", why);
					not_found
				}
			}
		})

		.run(tauri::generate_context!())

		.expect("error while running tauri application");
}

#[tauri::command]
fn open_bin(handle: AppHandle) {
	spawn(async move {
		let data_state: State<DataState> = handle.state();
		let image_state: State<ImageState> = handle.state();

		let mut file_dialog = FileDialog::new()
			.add_filter("firmware dump", &["bin"]);
		if let Some(base_path) = data_state.base_path.lock().unwrap().clone() {
			file_dialog = file_dialog.set_directory(base_path);
		}

		if let Some(path) = file_dialog.pick_file() {
			show_spinner(&handle);
			let raw_data = fs::read(&path).unwrap();
			let data = DataView::new(&raw_data);
			match read_card(&data) {
				Ok(card) => {
					*data_state.base_path.lock().unwrap() = Some(path.parent().unwrap().to_path_buf());
					*data_state.data_pack.lock().unwrap() = Some(card.data_pack.clone());
					*data_state.sprite_pack.lock().unwrap() = Some(card.sprite_pack.clone());
					if let Ok(image_data) = get_image_data(&card.sprite_pack.clone()) {
						*image_state.images.lock().unwrap() = image_data;
					}
					handle.emit("show_card", card).unwrap();
				},
				Err(why) => show_error_message(why)
			}
			hide_spinner(&handle);
		}
	});
}

#[tauri::command]
fn export_data(handle: AppHandle) {
	spawn(async move {
		let data_state: State<DataState> = handle.state();

		let no_data = if let None = *data_state.data_pack.lock().unwrap() { true } else { false };
		if no_data {
			show_error_message("No data to export".into());

		} else {
			let mut file_dialog = FileDialog::new()
				.add_filter("JSON", &["json"]);

			if let Some(base_path) = data_state.base_path.lock().unwrap().clone() {
				file_dialog = file_dialog.set_directory(base_path);
			}

			let file_result = file_dialog.save_file();

			if let Some(path) = file_result {
				show_spinner(&handle);
				if let Err(why) = export_data_to(&data_state, &path) {
					show_error_message(why);
				}
				hide_spinner(&handle);
			}
		}
	});
}

#[tauri::command]
fn export_images(handle: AppHandle) {
	spawn(async move {
		let data_state: State<DataState> = handle.state();
		let image_state: State<ImageState> = handle.state();

		if image_state.images.lock().unwrap().len() == 0 || image_state.images.lock().unwrap()[0].len() == 0 {
			show_error_message("No images to export".into());

		} else {
			let mut file_dialog = FileDialog::new()
				.add_filter("PNG image", &["png"]);
			if let Some(base_path) = data_state.base_path.lock().unwrap().clone() {
				file_dialog = file_dialog.set_directory(base_path);
			}
			let file_result = file_dialog.save_file();

			if let Some(path) = file_result {
				show_spinner(&handle);
				if let Err(why) = export_images_to(&image_state, &path) {
					show_error_message(why);
				}
				hide_spinner(&handle);
			}
		}

	});
}

fn read_card(data: &DataView) -> Result<TamaSmaCard, Box<dyn Error>> {
	let header = read_card_header(&data)?;
	let (data_pack, sprite_pack) = read_card_packs(&data.chunk(0x1000, data.len() - 0x1000))?;
	Ok(TamaSmaCard { header, data_pack, sprite_pack })
}

fn read_card_header(data: &DataView) -> Result<CardHeader, Box<dyn Error>> {
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

fn read_card_packs(data: &DataView) -> Result<(DataPack, SpritePack), Box<dyn Error>> {
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
		Err("Unable to read card data: sata pack not found".into())
	}
}

pub fn show_error_message(why: Box<dyn Error>) {
	println!("ERROR: {}", why);
	let _ = MessageDialog::new()
		.set_level(MessageLevel::Error)
		.set_title("Error")
		.set_description(format!("{}", why))
		.set_buttons(MessageButtons::Ok)
		.show();
}

pub fn show_spinner(handle: &AppHandle) {
	handle.emit("show_spinner", ()).unwrap();
}

pub fn hide_spinner(handle: &AppHandle) {
	handle.emit("hide_spinner", ()).unwrap();
}

#[tauri::command]
fn try_quit(handle: AppHandle) {
	handle.exit(0);
}
