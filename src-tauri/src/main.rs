// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::error::Error;
use std::path::PathBuf;
use std::sync::Mutex;
use std::io::Cursor;

use tauri::{ Builder, AppHandle, Manager, State };
use tauri::menu::{ Menu, Submenu, MenuItem, PredefinedMenuItem, MenuId };

use rfd::{ MessageLevel, MessageButtons, MessageDialog };

use regex::Regex;

mod smacard;
mod firmware;
mod data_view;
mod data_pack;
mod sprite_pack;
mod file;
mod export;
mod import;

use data_view::DataView;
use data_pack::DataPack;
use sprite_pack::SpritePack;
use file::{ open_bin, save_bin, save_bin_as, continue_if_modified };
use export::{ export_data, export_images, export_image_spritesheet };
use import::{ import_strings, import_menu_strings, import_image_spritesheet };

pub struct DataState {
	pub is_modified: Mutex<bool>,
	pub bin_type: Mutex<Option<BinType>>,
	pub file_path: Mutex<Option<PathBuf>>,
	pub base_path: Mutex<Option<PathBuf>>,
	pub data_pack: Mutex<Option<DataPack>>,
	pub sprite_pack: Mutex<Option<SpritePack>>,
	pub menu_strings: Mutex<Option<Vec<String>>>,
	pub original_data: Mutex<Option<Vec<u8>>>
}

impl DataState {
	pub fn new() -> DataState {
		DataState{
			is_modified: Mutex::new(false),
			bin_type: Mutex::new(None),
			file_path: Mutex::new(None),
			base_path: Mutex::new(None),
			data_pack: Mutex::new(None),
			sprite_pack: Mutex::new(None),
			menu_strings: Mutex::new(None),
			original_data: Mutex::new(None)
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

pub enum BinType {
	SmaCard,
	Firmware
}

fn main() {
	Builder::default()
		.invoke_handler(tauri::generate_handler![
			open_bin,
			save_bin,
			save_bin_as,
			export_data,
			export_images,
			import_image_spritesheet,
			export_image_spritesheet,
			import_strings,
			import_menu_strings,
			try_quit,
			sprite_pack::image_def::update_image_def
		])

		.manage(DataState::new())
		.manage(ImageState::new())

		.menu(|handle| {
			Menu::with_id_and_items(handle, "main", &[
				&Submenu::with_id_and_items(handle, "file", "File", true, &[
					&MenuItem::with_id(handle, "open", "Open", true, Some("CmdOrCtrl+O"))?,
					&PredefinedMenuItem::separator(handle)?,
					&MenuItem::with_id(handle, "save", "Save", true, Some("CmdOrCtrl+S"))?,
					&MenuItem::with_id(handle, "save_as", "Save As...", true, Some("CmdOrCtrl+Shift+S"))?,
					&PredefinedMenuItem::separator(handle)?,

					&Submenu::with_id_and_items(handle, "export", "Export", true, &[
						&MenuItem::with_id(handle, "export_data", "Export Data", true, None::<&str>)?,
						&MenuItem::with_id(handle, "export_images", "Export Images", true, None::<&str>)?,
					])?,

					&PredefinedMenuItem::separator(handle)?,
					&MenuItem::with_id(handle, "quit", "Quit", true, Some("CmdOrCtrl+Q"))?,
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
					"save" => save_bin(handle),
					"save_as" => save_bin_as(handle),
					"export_data" => export_data(handle),
					"export_images" => export_images(handle),
					"quit" => try_quit(handle),
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
				let img = match image_state.images.lock().unwrap().get(image_id) {
					Some(image) => {
						match image.get(subimage_index) {
							Some(subimage) => {
								subimage.clone()
							},
							None => {
								return Err(format!("subimage {}-{} not found", image_id, subimage_index).into());
							}
						}
					},
					None => {
						return Err(format!("image {} not found", image_id).into());
					}
				};

				let mut img_data = Cursor::new(Vec::new());
				let _ = img.write_to(&mut img_data, image::ImageFormat::Png)?;

				Ok(http::Response::builder()
					.header("Content-Type", "image/png")
					.body(img_data.into_inner())?)
			};

			match do_the_thing() {
				Ok(response) => response,
				Err(_why) => {
					// println!("ERROR: {}", why);
					not_found
				}
			}
		})

		.run(tauri::generate_context!())

		.expect("error while running tauri application");
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
	if continue_if_modified(&handle) {
		handle.exit(0);
	}
}

pub fn update_window_title(handle: &AppHandle) {
	let window = handle.get_webview_window("main").unwrap();
	let data_state: State<DataState> = handle.state();
	let file_path_opt = data_state.file_path.lock().unwrap();

	let modified_indicator = if *data_state.is_modified.lock().unwrap() { "*" } else { "" };

	let file_name = match file_path_opt.as_ref() {
		Some(file_path) => match file_path.file_name() {
			Some(file_name) => Some(file_name.to_string_lossy()),
			None => None
		},
		None => None
	};

	match file_name {
		Some(file_name) => window.set_title(&format!("Smarty Pants - {}{}", file_name, modified_indicator)).unwrap(),
		None => window.set_title("Smarty Pants").unwrap()
	}
}
