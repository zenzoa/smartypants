use std::fs;
use std::error::Error;
use std::path::PathBuf;
use std::sync::Mutex;

use tauri::{ AppHandle, Manager, State, Emitter };
use tauri::async_runtime::spawn;

use rfd::{ FileDialog, MessageButtons, MessageDialog, MessageDialogResult };

use crate::{ DataState, ImageState, BinType, BinSize, show_spinner, hide_spinner, show_error_message, update_window_title, set_lock_colors, update_card_size_menu };
use crate::data_view::DataView;
use crate::data_pack::DataPack;
use crate::sprite_pack::SpritePack;
use crate::text::{ Text, FontState, CharEncoding, EncodingLanguage };
use crate::smacard::{ CardHeader, read_card, save_card };
use crate::firmware::{ read_firmware, save_firmware };

#[derive(Default)]
pub struct FileState {
	pub is_modified: Mutex<bool>,
	pub file_path: Mutex<Option<PathBuf>>,
	pub base_path: Mutex<Option<PathBuf>>
}

#[derive(Clone, serde::Serialize)]
struct FrontendData {
	encoding_language: EncodingLanguage,
	char_codes: Vec<CharEncoding>,
	bin_type: Option<BinType>,
	card_header: Option<CardHeader>,
	data_pack: Option<DataPack>,
	sprite_pack: Option<SpritePack>,
	menu_strings: Option<Vec<Text>>,
	use_patch_header: bool
}

#[tauri::command]
pub fn open_bin(handle: AppHandle) {
	if continue_if_modified(&handle) {
		spawn(async move {
			let file_state: State<FileState> = handle.state();
			let data_state: State<DataState> = handle.state();
			let image_state: State<ImageState> = handle.state();
			let font_state: State<FontState> = handle.state();

			let mut file_dialog = FileDialog::new()
				.add_filter("firmware dump", &["bin"]);
			if let Some(base_path) = file_state.base_path.lock().unwrap().as_ref() {
				file_dialog = file_dialog.set_directory(base_path);
			}

			if let Some(path) = file_dialog.pick_file() {
				show_spinner(&handle);

				let raw_data = fs::read(&path).unwrap();
				*data_state.original_data.lock().unwrap() = Some(raw_data.clone());
				let data = DataView::new(&raw_data);

				let bin_type = if data.len() == 16777216 {
					BinType::Firmware
				} else {
					BinType::SmaCard
				};
				*data_state.bin_type.lock().unwrap() = Some(bin_type.clone());

				match bin_type {
					BinType::SmaCard => {
						match read_card(&font_state, &data) {
							Ok(card) => {
								*data_state.card_header.lock().unwrap() = Some(card.header.clone());

								*data_state.use_patch_header.lock().unwrap() = false;

								*data_state.data_pack.lock().unwrap() = Some(card.data_pack.clone());

								let mut sprite_pack = card.sprite_pack;
								match sprite_pack.get_image_data() {
									Ok(image_data) => *image_state.images.lock().unwrap() = image_data,
									Err(why) => show_error_message(why)
								}
								*data_state.sprite_pack.lock().unwrap() = Some(sprite_pack);

								*data_state.bin_size.lock().unwrap() = Some(if data.len() <= 0x20000 {
									BinSize::Card128KB
								} else if data.len() <= 0x100000 {
									BinSize::Card1MB
								} else if data.len() <= 0x200000 {
									BinSize::Card2MB
								} else {
									BinSize::TooBig
								});
								update_card_size_menu(&handle);

								send_data_to_frontend(&handle);
							},
							Err(why) => show_error_message(why)
						}
					},

					BinType::Firmware => {
						match read_firmware(&handle, &data) {
							Ok(mut firmware) => {
								*data_state.use_patch_header.lock().unwrap() = firmware.use_patch_header;

								*data_state.data_pack.lock().unwrap() = Some(firmware.data_pack.clone());

								if let Ok(image_data) = firmware.sprite_pack.get_image_data() {
									*image_state.images.lock().unwrap() = image_data;
								}

								*data_state.sprite_pack.lock().unwrap() = Some(firmware.sprite_pack.clone());

								if let Some(small_font_images) = image_state.images.lock().unwrap().get(98) {
									font_state.small_font_images.lock().unwrap().clone_from(small_font_images);
								}
								if let Some(large_font_images) = image_state.images.lock().unwrap().get(99) {
									font_state.large_font_images.lock().unwrap().clone_from(large_font_images);
								}

								*data_state.menu_strings.lock().unwrap() = Some(firmware.menu_strings.clone());

								*data_state.bin_size.lock().unwrap() = Some(BinSize::Firmware);

								set_lock_colors(&handle, Some(true));

								send_data_to_frontend(&handle);
							},
							Err(why) => show_error_message(why)
						}
					}
				}

				*file_state.is_modified.lock().unwrap() = false;
				*file_state.file_path.lock().unwrap() = Some(path.to_path_buf());
				*file_state.base_path.lock().unwrap() = Some(path.parent().unwrap().to_path_buf());

				hide_spinner(&handle);

				update_window_title(&handle);
			}
		});
	}
}

fn send_data_to_frontend(handle: &AppHandle) {
	let data_state: State<DataState> = handle.state();
	let font_state: State<FontState> = handle.state();

	let frontend_data = FrontendData {
		encoding_language: font_state.encoding_language.lock().unwrap().clone(),
		char_codes: font_state.char_codes.lock().unwrap().clone(),
		bin_type: data_state.bin_type.lock().unwrap().clone(),
		card_header: data_state.card_header.lock().unwrap().clone(),
		data_pack: data_state.data_pack.lock().unwrap().clone(),
		sprite_pack: data_state.sprite_pack.lock().unwrap().clone(),
		menu_strings: data_state.menu_strings.lock().unwrap().clone(),
		use_patch_header: *data_state.use_patch_header.lock().unwrap()
	};
	handle.emit("update_data", frontend_data).unwrap();
}

#[tauri::command]
pub fn save_bin(handle: AppHandle) {
	let file_state: State<FileState> = handle.state();
	let data_state: State<DataState> = handle.state();
	let no_data = data_state.data_pack.lock().unwrap().is_none();
	if no_data {
		show_error_message("No data to save".into());

	} else {
		let file_path_opt = file_state.file_path.lock().unwrap().clone();
		match file_path_opt {
			Some(file_path) => {
				show_spinner(&handle);
				if let Err(why) = save(&handle, &file_path) {
					show_error_message(why);
					update_window_title(&handle);
				}
				hide_spinner(&handle);
			},
			None => save_bin_as(handle)
		}
	}
}

#[tauri::command]
pub fn save_bin_as(handle: AppHandle) {
	spawn(async move {
		let file_state: State<FileState> = handle.state();
		let data_state: State<DataState> = handle.state();
		let no_data = data_state.data_pack.lock().unwrap().is_none();
		if no_data {
			show_error_message("No data to save".into());

		} else {
			let mut file_dialog = FileDialog::new()
				.add_filter("tama smart data dump", &["bin"]);

			if let Some(base_path) = file_state.base_path.lock().unwrap().as_ref() {
				file_dialog = file_dialog.set_directory(base_path);
			}

			if let Some(file_path) = file_state.file_path.lock().unwrap().as_ref() {
				if let Some(file_stem) = file_path.file_stem() {
					file_dialog = file_dialog.set_file_name(format!("{}-copy.bin", file_stem.to_string_lossy()));
				}
			}

			if let Some(path) = file_dialog.save_file() {
				show_spinner(&handle);
				match save(&handle, &path) {
					Ok(()) => {
						*file_state.file_path.lock().unwrap() = Some(path.to_path_buf());
						*file_state.base_path.lock().unwrap() = Some(path.parent().unwrap().to_path_buf());
						update_window_title(&handle);
					},
					Err(why) => show_error_message(why)
				}
				hide_spinner(&handle);
			}
		}
	});
}

pub fn save(handle: &AppHandle, path: &PathBuf) -> Result<(), Box<dyn Error>> {
	let data_state: State<DataState> = handle.state();

	let bin_type_base = data_state.bin_type.lock().unwrap();
	let bin_type = bin_type_base.as_ref().ok_or("Invalid bin type")?;
	match bin_type {
		BinType::Firmware => {
			let original_data_base = data_state.original_data.lock().unwrap();
			let original_data = original_data_base.as_ref().ok_or("No original data found for current file")?;
			let new_data = save_firmware(handle, original_data)?;
			if original_data.len() == new_data.len() {
				fs::write(path, &new_data)?;
				set_file_modified(handle, false);
			} else {
				return Err(format!("New data is {} bytes, but original is {} bytes", new_data.len(), original_data.len()).into());
			}
		},

		BinType::SmaCard => {
			let new_data = save_card(handle)?;
			fs::write(path, new_data)?;
			set_file_modified(handle, false);
		}
	}

	Ok(())
}

pub fn set_file_modified(handle: &AppHandle, value: bool) {
	let file_state: State<FileState> = handle.state();
	*file_state.is_modified.lock().unwrap() = value;
	update_window_title(handle);
}

pub fn continue_if_modified(handle: &AppHandle) -> bool {
	let file_state: State<FileState> = handle.state();
	if *file_state.is_modified.lock().unwrap() {
		let dialog_result = MessageDialog::new()
			.set_title("File modified")
			.set_description("Do you want to continue anyway and lose any unsaved work?")
			.set_buttons(MessageButtons::YesNo)
			.show();
		matches!(dialog_result, MessageDialogResult::Yes)
	} else {
		true
	}
}
