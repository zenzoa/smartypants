// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::error::Error;
use std::path::PathBuf;
use std::sync::Mutex;
use std::io::Cursor;

use tauri::{ Builder, AppHandle, Manager, State };
use tauri::menu::{ Menu, Submenu, MenuItem, PredefinedMenuItem, CheckMenuItem, MenuId };
use tauri::path::BaseDirectory;

use rfd::{ MessageLevel, MessageButtons, MessageDialog };

use regex::Regex;

mod smacard;
mod firmware;
mod data_view;
mod data_pack;
mod sprite_pack;
mod text;
mod file;
mod export;
mod import;

use data_view::DataView;
use data_pack::DataPack;
use sprite_pack::SpritePack;
use text::{ Text, FontState, set_to_preset_encoding };
use file::{ open_bin, save_bin, save_bin_as, continue_if_modified };
use import::import_encoding;

#[derive(Default)]
pub struct DataState {
	pub is_modified: Mutex<bool>,
	pub bin_type: Mutex<Option<BinType>>,
	pub file_path: Mutex<Option<PathBuf>>,
	pub base_path: Mutex<Option<PathBuf>>,
	pub data_pack: Mutex<Option<DataPack>>,
	pub sprite_pack: Mutex<Option<SpritePack>>,
	pub menu_strings: Mutex<Option<Vec<Text>>>,
	pub use_patch_header: Mutex<bool>,
	pub original_data: Mutex<Option<Vec<u8>>>
}

#[derive(Default)]
pub struct ImageState {
	pub images: Mutex<Vec<Vec<image::RgbaImage>>>
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
			export::export_data,
			export::export_strings,
			export::export_images,
			export::export_image_spritesheet,
			export::export_encoding,
			import::import_strings,
			import::import_image_spritesheet,
			import::import_encoding,
			try_quit,
			firmware::set_patch_header,
			data_pack::item::update_item,
			data_pack::character::update_character,
			sprite_pack::image_def::update_image_def,
			text::validate_string,
			text::decode_string_js,
			text::get_default_char_codes,
			text::update_char_codes
		])

		.manage(DataState::default())
		.manage(ImageState::default())
		.manage(FontState::default())

		.menu(|handle| {
			Menu::with_id_and_items(handle, "main", &[
				&Submenu::with_id_and_items(handle, "file", "File", true, &[
					&MenuItem::with_id(handle, "open", "Open", true, Some("CmdOrCtrl+O"))?,
					&PredefinedMenuItem::separator(handle)?,
					&MenuItem::with_id(handle, "save", "Save", true, Some("CmdOrCtrl+S"))?,
					&MenuItem::with_id(handle, "save_as", "Save As...", true, Some("CmdOrCtrl+Shift+S"))?,
					&PredefinedMenuItem::separator(handle)?,

					&Submenu::with_id_and_items(handle, "import", "Import", true, &[
						&MenuItem::with_id(handle, "import_strings", "Import Strings", true, None::<&str>)?,
					])?,

					&Submenu::with_id_and_items(handle, "export", "Export", true, &[
						&MenuItem::with_id(handle, "export_data", "Export Data", true, None::<&str>)?,
						&MenuItem::with_id(handle, "export_strings", "Export Strings", true, None::<&str>)?,
						&MenuItem::with_id(handle, "export_images", "Export Images", true, None::<&str>)?,
					])?,

					&PredefinedMenuItem::separator(handle)?,
					&MenuItem::with_id(handle, "quit", "Quit", true, Some("CmdOrCtrl+Q"))?,
				])?,

				&Submenu::with_id_and_items(handle, "text", "Text", true, &[
					&Submenu::with_id_and_items(handle, "change_encoding", "Change Encoding", true, &[
						&CheckMenuItem::with_id(handle, "set_encoding_to_jp", "Japanese", true, true, None::<&str>)?,
						&CheckMenuItem::with_id(handle, "set_encoding_to_en", "English/Latin", true, false, None::<&str>)?,
						&MenuItem::with_id(handle, "import_encoding", "Import...", true, None::<&str>)?,
					])?,
					&MenuItem::with_id(handle, "edit_encoding", "Edit Encoding", true, None::<&str>)?,
				])?,

				&Submenu::with_id_and_items(handle, "help", "Help", true, &[
					&MenuItem::with_id(handle, "about", "About", true, None::<&str>)?,
				])?,
			])
		})

		.setup(|app| {
			let font_state: State<FontState> = app.state();

			if let Ok(small_font_path) = app.path().resolve("resources/font_small_jp.png", BaseDirectory::Resource) {
				if let Ok(small_font) = text::load_font(&small_font_path) {
					*font_state.small_font_images.lock().unwrap() = small_font;
				}
			}

			if let Ok(large_font_path) = app.path().resolve("resources/font_large_jp.png", BaseDirectory::Resource) {
				if let Ok(large_font) = text::load_font(&large_font_path) {
					*font_state.large_font_images.lock().unwrap() = large_font;
				}
			}

			app.on_menu_event(|handle, event| {
				let MenuId(id) = event.id();
				let handle = handle.clone();

				match id.as_str() {
					"open" => open_bin(handle),
					"save" => save_bin(handle),
					"save_as" => save_bin_as(handle),

					"import_strings" => import::import_strings(handle),

					"export_data" => export::export_data(handle),
					"export_strings" => export::export_strings(handle),
					"export_images" => export::export_images(handle),

					"quit" => try_quit(handle),

					"set_encoding_to_jp" => set_to_preset_encoding(handle, "jp"),
					"set_encoding_to_en" => set_to_preset_encoding(handle, "en"),
					"import_encoding" => import_encoding(handle),
					"edit_encoding" => handle.emit("show_encoding_dialog", "").unwrap(),

					"about" => handle.emit("show_about_dialog", "").unwrap(),
					_ => {}
				}
			});
			Ok(())
		})

		.register_uri_scheme_protocol("getimage", |app, request| {
			let not_found = http::Response::builder().body(Vec::new()).unwrap();

			let mut img_data = Cursor::new(Vec::new());

			let do_the_thing = || -> Result<http::Response<Vec<u8>>, Box<dyn Error>> {
				let re = Regex::new(r"-(\w+)-(\d+)$").unwrap();
				let uri = request.uri().path();
				let caps = re.captures(uri).ok_or("no capture groups")?;

				let image_id_str = caps.get(1).ok_or("no image id")?.as_str();
				let subimage_index_str = caps.get(2).ok_or("no subimage index")?.as_str();
				let subimage_index = subimage_index_str.parse::<usize>()?;

				match image_id_str {
					"smallfont" => {
						let font_state: State<FontState> = app.state();
						let img = text::get_char_image_small(&font_state, subimage_index).ok_or("no font image found for character index")?;
						img.write_to(&mut img_data, image::ImageFormat::Png)?;
					},
					"largefont" => {
						let font_state: State<FontState> = app.state();
						let img = text::get_char_image_large(&font_state, subimage_index).ok_or("no font image found for character index")?;
						img.write_to(&mut img_data, image::ImageFormat::Png)?;
					},
					_ => {
						let image_id = image_id_str.parse::<usize>()?;

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

						img.write_to(&mut img_data, image::ImageFormat::Png)?;
					}
				}

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
		Some(file_path) => file_path.file_name().map(|file_name| file_name.to_string_lossy()),
		None => None
	};


	match file_name {
		Some(file_name) => window.set_title(&format!("Smarty Pants - {}{}", file_name, modified_indicator)).unwrap(),
		None => window.set_title("Smarty Pants").unwrap()
	}
}
