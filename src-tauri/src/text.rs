use std::error::Error;
use std::sync::Mutex;
use std::path::PathBuf;

use tauri::{ AppHandle, Manager, State, Emitter, path::BaseDirectory };
use tauri::menu::MenuItemKind;

use image::ImageReader;
use image::{ RgbaImage, GenericImageView };

use rfd::{ MessageButtons, MessageDialog, MessageDialogResult };

use crate::{ DataState, BinType, show_error_message };
use crate::import::import_encoding_from;

#[derive(Clone, serde::Serialize)]
pub struct Text {
	pub data: Vec<u16>,
	pub string: String
}

impl Text {
	pub fn from_data(char_codes: &[CharEncoding], new_data: &[u16]) -> Text {
		Text {
			data: new_data.to_vec(),
			string: encode_string(char_codes, new_data)
		}
	}

	pub fn from_string(char_codes: &[CharEncoding], new_string: &str) -> Text {
		Text {
			data: decode_string(char_codes, new_string),
			string: new_string.to_string()
		}
	}

	pub fn set_string(&mut self, char_codes: &[CharEncoding], new_string: &str) {
		self.data = decode_string(char_codes, new_string);
		self.string = new_string.to_string();
	}

	pub fn update_string(&mut self, char_codes: &[CharEncoding]) {
		let mut new_string = String::new();
		for word in &self.data {
			if let Some(substring) = word_to_char_code(char_codes, *word) {
				new_string.push_str(&substring);
			}
		}
		self.string = new_string;
	}
}

pub struct FontState {
	pub char_codes: Mutex<Vec<CharEncoding>>,
	pub encoding_language: Mutex<EncodingLanguage>,
	pub small_font_images: Mutex<Vec<image::RgbaImage>>,
	pub large_font_images: Mutex<Vec<image::RgbaImage>>
}

impl Default for FontState {
	fn default() -> FontState {
		FontState{
			char_codes: Mutex::new(get_default_char_codes()),
			encoding_language: Mutex::new(EncodingLanguage::Japanese),
			small_font_images: Mutex::new(Vec::new()),
			large_font_images: Mutex::new(Vec::new())
		}
	}
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct CharEncoding {
	data: u16,
	text: Vec<String>
}

#[derive(Clone, PartialEq, serde::Serialize)]
pub enum EncodingLanguage {
	Custom,
	Japanese,
	English,
	Latin
}

pub fn word_to_char_code(char_codes: &[CharEncoding], word: u16) -> Option<String> {
	if word <= 256 {
		Some(char_codes[word as usize].text[0].clone())
	} else {
		for char_code in char_codes.iter() {
			if word == char_code.data {
				return Some(char_code.text[0].clone())
			}
		}
		None
	}
}

pub fn char_code_to_word(char_codes: &[CharEncoding], text: &str) -> Option<u16> {
	for char_code in char_codes.iter() {
		if char_code.text.contains(&text.to_string()) {
			return Some(char_code.data);
		}
	}
	None
}

#[tauri::command]
pub fn update_char_codes(handle: AppHandle, new_char_codes: Vec<CharEncoding>) -> (Vec<CharEncoding>, Vec<u16>) {
	let font_state: State<FontState> = handle.state();
	let mut char_codes = font_state.char_codes.lock().unwrap();

	let mut problem_codes = Vec::new();
	let mut has_duplicate = false;
	let mut has_invalid = false;

	for new_char_code1 in &new_char_codes {
		for text in new_char_code1.text.iter() {
			if text.chars().count() > 1 && !(text.starts_with('{') && text.ends_with('}')) {
				problem_codes.push(new_char_code1.data);
				has_invalid = true;
			}

			for new_char_code2 in &new_char_codes {
				if new_char_code1.data != new_char_code2.data && !text.is_empty() && new_char_code2.text.contains(text) {
					problem_codes.push(new_char_code2.data);
					has_duplicate = true;
				}
			}
		}
	}

	if problem_codes.is_empty() {
		for new_char_code in new_char_codes {
			for char_code in char_codes.iter_mut() {
				if char_code.data == new_char_code.data {
					char_code.text.clone_from(&new_char_code.text);
				}
			}
		}
		*font_state.encoding_language.lock().unwrap() = EncodingLanguage::Custom;
		re_decode_strings(&handle, &char_codes);
		refresh_encoding_menu(&handle);
	}

	problem_codes.sort();
	problem_codes.dedup();

	if has_duplicate {
		show_error_message("Duplicate value: each tama character must have a unique unicode representation.".into());
	}

	if has_invalid {
		show_error_message("Invalid value: each tama character must be represented by a single unicode character, or a sequence of characters surrounded by { and }.".into());
	}

	(char_codes.to_vec(), problem_codes)
}

#[tauri::command]
pub fn decode_string_js(handle: AppHandle, string: &str) -> Vec<u16> {
	let font_state: State<FontState> = handle.state();
	let char_codes = &font_state.char_codes.lock().unwrap();
	decode_string(char_codes, string)
}

pub fn decode_string(char_codes: &[CharEncoding], string: &str) -> Vec<u16> {
	let mut data: Vec<u16> = Vec::new();

	let mut var_name = String::new();
	for ch in string.chars() {
		match ch {
			'{' | '<' => {
				var_name.push(ch);
			},
			'}' | '>' => {
				var_name.push(ch);
				if let Some(word) = char_code_to_word(char_codes, &var_name.to_lowercase()) {
					data.push(word);
				}
				var_name = String::new();
			},
			_ => {
				if var_name.is_empty() {
					if let Some(word) = char_code_to_word(char_codes, &ch.to_string()) {
						data.push(word);
					}
				} else {
					var_name.push(ch);
				}
			}
		}
	}
	data
}

pub fn encode_string(char_codes: &[CharEncoding], data: &[u16]) -> String {
	let mut new_string = String::new();
	for word in data {
		if let Some(substring) = word_to_char_code(char_codes, *word) {
			new_string.push_str(&substring);
		}
	}
	new_string
}

pub fn get_char_image_small(font_state: &FontState, char_index: usize) -> Option<RgbaImage> {
	let small_font_images = font_state.small_font_images.lock().unwrap();
	small_font_images.get(char_index).cloned()
}

pub fn get_char_image_large(font_state: &FontState, char_index: usize) -> Option<RgbaImage> {
	let large_font_images = font_state.large_font_images.lock().unwrap();
	large_font_images.get(char_index).cloned()
}

#[tauri::command]
pub fn validate_string(handle: AppHandle, string: &str, max_length: usize) -> (bool, String) {
	let font_state: State<FontState> = handle.state();
	let char_codes = &font_state.char_codes.lock().unwrap();

	let words = decode_string(char_codes, string);
	let string2 = encode_string(char_codes, &words);
	let words2 = decode_string(char_codes, &string2);
	(words == words2 && words2.len() <= max_length, string2)
}

pub fn load_font(path: &PathBuf) -> Result<Vec<RgbaImage>, Box<dyn Error>> {
	let image = ImageReader::open(path)?.decode()?;
	if image.width() != 4096 || image.height() != 16 {
		return Err("Font image is not the correct size, 4096x16".into());
	}
	let mut subimages: Vec<RgbaImage> = Vec::new();
	for i in 0..256 {
		let subimage = image.view(i*16, 0, 16, 16).to_image();
		subimages.push(subimage);
	}
	Ok(subimages)
}

#[tauri::command]
pub fn set_to_preset_encoding(handle: AppHandle, name: &str) {
	let data_state: State<DataState> = handle.state();
	let font_state: State<FontState> = handle.state();

	let do_the_thing = || {
		if let Ok(encoding_path) = handle.path().resolve(format!("resources/encoding_{}.json", name), BaseDirectory::Resource) {
			match import_encoding_from(&handle, &font_state, &encoding_path, false) {
				Ok(()) => {
					*font_state.encoding_language.lock().unwrap() = match name {
						"jp" => EncodingLanguage::Japanese,
						"en" => EncodingLanguage::English,
						"latin" => EncodingLanguage::Latin,
						_ => EncodingLanguage::Custom
					};

					if let Some(BinType::SmaCard) = *data_state.bin_type.lock().unwrap() {
						if let Ok(small_font_path) = handle.path().resolve(format!("resources/font_small_{}.png", name), BaseDirectory::Resource) {
							if let Ok(small_font) = load_font(&small_font_path) {
								*font_state.small_font_images.lock().unwrap() = small_font;
							}
						}
						if let Ok(large_font_path) = handle.path().resolve(format!("resources/font_large_{}.png", name), BaseDirectory::Resource) {
							if let Ok(large_font) = load_font(&large_font_path) {
								*font_state.large_font_images.lock().unwrap() = large_font;
							}
						}
					}

					let char_codes = &font_state.char_codes.lock().unwrap();
					re_decode_strings(&handle, char_codes);
				},

				Err(why) => show_error_message(why)
			}
		}
	};

	if *font_state.encoding_language.lock().unwrap() == EncodingLanguage::Custom {
		let dialog_result = MessageDialog::new()
			.set_title("Change Text Encoding")
			.set_description("This will overwrite your existing text encoding. Are you sure you want to continue?")
			.set_buttons(MessageButtons::YesNo)
			.show();
		if dialog_result == MessageDialogResult::Yes{
			do_the_thing();
		}
	} else {
		do_the_thing();
	}

	refresh_encoding_menu(&handle);
}

pub fn refresh_encoding_menu(handle: &AppHandle) {
	let font_state: State<FontState> = handle.state();
	let encoding_language = font_state.encoding_language.lock().unwrap();

	if let Some(menu) = handle.menu() {
		if let Some(MenuItemKind::Submenu(config_menu)) = menu.get("config") {
			if let Some(MenuItemKind::Submenu(text_encoding_menu)) = config_menu.get("text_encoding") {
				if let Some(MenuItemKind::Check(menu_item_jp)) = text_encoding_menu.get("encoding_jp") {
					menu_item_jp.set_checked(*encoding_language == EncodingLanguage::Japanese).unwrap();
				}
				if let Some(MenuItemKind::Check(menu_item_en)) = text_encoding_menu.get("encoding_en") {
					menu_item_en.set_checked(*encoding_language == EncodingLanguage::English).unwrap();
				}
				if let Some(MenuItemKind::Check(menu_item_latin)) = text_encoding_menu.get("encoding_latin") {
					menu_item_latin.set_checked(*encoding_language == EncodingLanguage::Latin).unwrap();
				}
				if let Some(MenuItemKind::Check(menu_item_custom)) = text_encoding_menu.get("encoding_custom") {
					menu_item_custom.set_checked(*encoding_language == EncodingLanguage::Custom).unwrap();
				}
			}
		}
	}

	handle.emit("update_encoding_language", encoding_language.clone()).unwrap();
}

pub fn re_decode_strings(handle: &AppHandle, char_codes: &[CharEncoding]) {
	let data_state: State<DataState> = handle.state();

	let mut menu_strings_opt = data_state.menu_strings.lock().unwrap();
	if let Some(menu_strings) = menu_strings_opt.as_mut() {
		for menu_string in menu_strings.iter_mut() {
			menu_string.update_string(char_codes);
		}
		handle.emit("update_menu_strings", (&menu_strings, false)).unwrap();
	}

	let mut data_pack_opt = data_state.data_pack.lock().unwrap();
	if let Some(data_pack) = data_pack_opt.as_mut() {
		for tamastring in data_pack.tamastrings.iter_mut() {
			tamastring.value.update_string(char_codes);
		}
		handle.emit("update_tamastrings", (&data_pack.tamastrings, false)).unwrap();

		for item in data_pack.items.iter_mut() {
			item.name.update_string(char_codes);
		}
		handle.emit("update_items", (&data_pack.items, false)).unwrap();

		for character in data_pack.characters.iter_mut() {
			character.name.update_string(char_codes);
			character.pronoun.update_string(char_codes);
			character.statement.update_string(char_codes);
			character.question1.update_string(char_codes);
			character.question2.update_string(char_codes);
		}
		handle.emit("update_characters", (&data_pack.characters, false)).unwrap();
	}

	handle.emit("refresh_tab", ()).unwrap();
}

#[tauri::command]
pub fn get_default_char_codes() -> Vec<CharEncoding> {
	vec![
		CharEncoding { data: 0u16, text: vec![String::from("█")] },
		CharEncoding { data: 1u16, text: vec![String::from(" "), String::from(" ")] },
		CharEncoding { data: 2u16, text: vec![String::from("０"), String::from("0")] },
		CharEncoding { data: 3u16, text: vec![String::from("１"), String::from("1")] },
		CharEncoding { data: 4u16, text: vec![String::from("２"), String::from("2")] },
		CharEncoding { data: 5u16, text: vec![String::from("３"), String::from("3")] },
		CharEncoding { data: 6u16, text: vec![String::from("４"), String::from("4")] },
		CharEncoding { data: 7u16, text: vec![String::from("５"), String::from("5")] },
		CharEncoding { data: 8u16, text: vec![String::from("６"), String::from("6")] },
		CharEncoding { data: 9u16, text: vec![String::from("７"), String::from("7")] },
		CharEncoding { data: 10u16, text: vec![String::from("８"), String::from("8")] },
		CharEncoding { data: 11u16, text: vec![String::from("９"), String::from("9")] },
		CharEncoding { data: 12u16, text: vec![String::from("＋"), String::from("+")] },
		CharEncoding { data: 13u16, text: vec![String::from("－"), String::from("-")] },
		CharEncoding { data: 14u16, text: vec![String::from("↵")] },
		CharEncoding { data: 15u16, text: vec![String::from("あ")] },
		CharEncoding { data: 16u16, text: vec![String::from("い")] },
		CharEncoding { data: 17u16, text: vec![String::from("う")] },
		CharEncoding { data: 18u16, text: vec![String::from("え")] },
		CharEncoding { data: 19u16, text: vec![String::from("お")] },
		CharEncoding { data: 20u16, text: vec![String::from("か")] },
		CharEncoding { data: 21u16, text: vec![String::from("き")] },
		CharEncoding { data: 22u16, text: vec![String::from("く")] },
		CharEncoding { data: 23u16, text: vec![String::from("け")] },
		CharEncoding { data: 24u16, text: vec![String::from("こ")] },
		CharEncoding { data: 25u16, text: vec![String::from("さ")] },
		CharEncoding { data: 26u16, text: vec![String::from("し")] },
		CharEncoding { data: 27u16, text: vec![String::from("す")] },
		CharEncoding { data: 28u16, text: vec![String::from("せ")] },
		CharEncoding { data: 29u16, text: vec![String::from("そ")] },
		CharEncoding { data: 30u16, text: vec![String::from("た")] },
		CharEncoding { data: 31u16, text: vec![String::from("ち")] },
		CharEncoding { data: 32u16, text: vec![String::from("つ")] },
		CharEncoding { data: 33u16, text: vec![String::from("て")] },
		CharEncoding { data: 34u16, text: vec![String::from("と")] },
		CharEncoding { data: 35u16, text: vec![String::from("な")] },
		CharEncoding { data: 36u16, text: vec![String::from("に")] },
		CharEncoding { data: 37u16, text: vec![String::from("ぬ")] },
		CharEncoding { data: 38u16, text: vec![String::from("ね")] },
		CharEncoding { data: 39u16, text: vec![String::from("の")] },
		CharEncoding { data: 40u16, text: vec![String::from("は")] },
		CharEncoding { data: 41u16, text: vec![String::from("ひ")] },
		CharEncoding { data: 42u16, text: vec![String::from("ふ")] },
		CharEncoding { data: 43u16, text: vec![String::from("へ")] },
		CharEncoding { data: 44u16, text: vec![String::from("ほ")] },
		CharEncoding { data: 45u16, text: vec![String::from("ま")] },
		CharEncoding { data: 46u16, text: vec![String::from("み")] },
		CharEncoding { data: 47u16, text: vec![String::from("む")] },
		CharEncoding { data: 48u16, text: vec![String::from("め")] },
		CharEncoding { data: 49u16, text: vec![String::from("も")] },
		CharEncoding { data: 50u16, text: vec![String::from("や")] },
		CharEncoding { data: 51u16, text: vec![String::from("ゆ")] },
		CharEncoding { data: 52u16, text: vec![String::from("よ")] },
		CharEncoding { data: 53u16, text: vec![String::from("ら")] },
		CharEncoding { data: 54u16, text: vec![String::from("り")] },
		CharEncoding { data: 55u16, text: vec![String::from("る")] },
		CharEncoding { data: 56u16, text: vec![String::from("れ")] },
		CharEncoding { data: 57u16, text: vec![String::from("ろ")] },
		CharEncoding { data: 58u16, text: vec![String::from("わ")] },
		CharEncoding { data: 59u16, text: vec![String::from("を")] },
		CharEncoding { data: 60u16, text: vec![String::from("ん")] },
		CharEncoding { data: 61u16, text: vec![String::from("ぁ")] },
		CharEncoding { data: 62u16, text: vec![String::from("ぃ")] },
		CharEncoding { data: 63u16, text: vec![String::from("ぅ")] },
		CharEncoding { data: 64u16, text: vec![String::from("ぇ")] },
		CharEncoding { data: 65u16, text: vec![String::from("ぉ")] },
		CharEncoding { data: 66u16, text: vec![String::from("っ")] },
		CharEncoding { data: 67u16, text: vec![String::from("ゃ")] },
		CharEncoding { data: 68u16, text: vec![String::from("ゅ")] },
		CharEncoding { data: 69u16, text: vec![String::from("ょ")] },
		CharEncoding { data: 70u16, text: vec![String::from("が")] },
		CharEncoding { data: 71u16, text: vec![String::from("ぎ")] },
		CharEncoding { data: 72u16, text: vec![String::from("ぐ")] },
		CharEncoding { data: 73u16, text: vec![String::from("げ")] },
		CharEncoding { data: 74u16, text: vec![String::from("ご")] },
		CharEncoding { data: 75u16, text: vec![String::from("ざ")] },
		CharEncoding { data: 76u16, text: vec![String::from("じ")] },
		CharEncoding { data: 77u16, text: vec![String::from("ず")] },
		CharEncoding { data: 78u16, text: vec![String::from("ぜ")] },
		CharEncoding { data: 79u16, text: vec![String::from("ぞ")] },
		CharEncoding { data: 80u16, text: vec![String::from("だ")] },
		CharEncoding { data: 81u16, text: vec![String::from("ぢ")] },
		CharEncoding { data: 82u16, text: vec![String::from("づ")] },
		CharEncoding { data: 83u16, text: vec![String::from("で")] },
		CharEncoding { data: 84u16, text: vec![String::from("ど")] },
		CharEncoding { data: 85u16, text: vec![String::from("ば")] },
		CharEncoding { data: 86u16, text: vec![String::from("び")] },
		CharEncoding { data: 87u16, text: vec![String::from("ぶ")] },
		CharEncoding { data: 88u16, text: vec![String::from("べ")] },
		CharEncoding { data: 89u16, text: vec![String::from("ぼ")] },
		CharEncoding { data: 90u16, text: vec![String::from("ぱ")] },
		CharEncoding { data: 91u16, text: vec![String::from("ぴ")] },
		CharEncoding { data: 92u16, text: vec![String::from("ぷ")] },
		CharEncoding { data: 93u16, text: vec![String::from("ぺ")] },
		CharEncoding { data: 94u16, text: vec![String::from("ぽ")] },
		CharEncoding { data: 95u16, text: vec![String::from("ア")] },
		CharEncoding { data: 96u16, text: vec![String::from("イ")] },
		CharEncoding { data: 97u16, text: vec![String::from("ウ")] },
		CharEncoding { data: 98u16, text: vec![String::from("エ")] },
		CharEncoding { data: 99u16, text: vec![String::from("オ")] },
		CharEncoding { data: 100u16, text: vec![String::from("カ")] },
		CharEncoding { data: 101u16, text: vec![String::from("キ")] },
		CharEncoding { data: 102u16, text: vec![String::from("ク")] },
		CharEncoding { data: 103u16, text: vec![String::from("ケ")] },
		CharEncoding { data: 104u16, text: vec![String::from("コ")] },
		CharEncoding { data: 105u16, text: vec![String::from("サ")] },
		CharEncoding { data: 106u16, text: vec![String::from("シ")] },
		CharEncoding { data: 107u16, text: vec![String::from("ス")] },
		CharEncoding { data: 108u16, text: vec![String::from("セ")] },
		CharEncoding { data: 109u16, text: vec![String::from("ソ")] },
		CharEncoding { data: 110u16, text: vec![String::from("タ")] },
		CharEncoding { data: 111u16, text: vec![String::from("チ")] },
		CharEncoding { data: 112u16, text: vec![String::from("ツ")] },
		CharEncoding { data: 113u16, text: vec![String::from("テ")] },
		CharEncoding { data: 114u16, text: vec![String::from("ト")] },
		CharEncoding { data: 115u16, text: vec![String::from("ナ")] },
		CharEncoding { data: 116u16, text: vec![String::from("ニ")] },
		CharEncoding { data: 117u16, text: vec![String::from("ヌ")] },
		CharEncoding { data: 118u16, text: vec![String::from("ネ")] },
		CharEncoding { data: 119u16, text: vec![String::from("ノ")] },
		CharEncoding { data: 120u16, text: vec![String::from("ハ")] },
		CharEncoding { data: 121u16, text: vec![String::from("ヒ")] },
		CharEncoding { data: 122u16, text: vec![String::from("フ")] },
		CharEncoding { data: 123u16, text: vec![String::from("ヘ")] },
		CharEncoding { data: 124u16, text: vec![String::from("ホ")] },
		CharEncoding { data: 125u16, text: vec![String::from("マ")] },
		CharEncoding { data: 126u16, text: vec![String::from("ミ")] },
		CharEncoding { data: 127u16, text: vec![String::from("ム")] },
		CharEncoding { data: 128u16, text: vec![String::from("メ")] },
		CharEncoding { data: 129u16, text: vec![String::from("モ")] },
		CharEncoding { data: 130u16, text: vec![String::from("ヤ")] },
		CharEncoding { data: 131u16, text: vec![String::from("ユ")] },
		CharEncoding { data: 132u16, text: vec![String::from("ヨ")] },
		CharEncoding { data: 133u16, text: vec![String::from("ラ")] },
		CharEncoding { data: 134u16, text: vec![String::from("リ")] },
		CharEncoding { data: 135u16, text: vec![String::from("ル")] },
		CharEncoding { data: 136u16, text: vec![String::from("レ")] },
		CharEncoding { data: 137u16, text: vec![String::from("ロ")] },
		CharEncoding { data: 138u16, text: vec![String::from("ワ")] },
		CharEncoding { data: 139u16, text: vec![String::from("ヲ")] },
		CharEncoding { data: 140u16, text: vec![String::from("ン")] },
		CharEncoding { data: 141u16, text: vec![String::from("ァ")] },
		CharEncoding { data: 142u16, text: vec![String::from("ィ")] },
		CharEncoding { data: 143u16, text: vec![String::from("ゥ")] },
		CharEncoding { data: 144u16, text: vec![String::from("ェ")] },
		CharEncoding { data: 145u16, text: vec![String::from("ォ")] },
		CharEncoding { data: 146u16, text: vec![String::from("ッ")] },
		CharEncoding { data: 147u16, text: vec![String::from("ャ")] },
		CharEncoding { data: 148u16, text: vec![String::from("ュ")] },
		CharEncoding { data: 149u16, text: vec![String::from("ョ")] },
		CharEncoding { data: 150u16, text: vec![String::from("ガ")] },
		CharEncoding { data: 151u16, text: vec![String::from("ギ")] },
		CharEncoding { data: 152u16, text: vec![String::from("グ")] },
		CharEncoding { data: 153u16, text: vec![String::from("ゲ")] },
		CharEncoding { data: 154u16, text: vec![String::from("ゴ")] },
		CharEncoding { data: 155u16, text: vec![String::from("ザ")] },
		CharEncoding { data: 156u16, text: vec![String::from("ジ")] },
		CharEncoding { data: 157u16, text: vec![String::from("ズ")] },
		CharEncoding { data: 158u16, text: vec![String::from("ゼ")] },
		CharEncoding { data: 159u16, text: vec![String::from("ゾ")] },
		CharEncoding { data: 160u16, text: vec![String::from("ダ")] },
		CharEncoding { data: 161u16, text: vec![String::from("ヂ")] },
		CharEncoding { data: 162u16, text: vec![String::from("ヅ")] },
		CharEncoding { data: 163u16, text: vec![String::from("デ")] },
		CharEncoding { data: 164u16, text: vec![String::from("ド")] },
		CharEncoding { data: 165u16, text: vec![String::from("バ")] },
		CharEncoding { data: 166u16, text: vec![String::from("ビ")] },
		CharEncoding { data: 167u16, text: vec![String::from("ブ")] },
		CharEncoding { data: 168u16, text: vec![String::from("ベ")] },
		CharEncoding { data: 169u16, text: vec![String::from("ボ")] },
		CharEncoding { data: 170u16, text: vec![String::from("パ")] },
		CharEncoding { data: 171u16, text: vec![String::from("ピ")] },
		CharEncoding { data: 172u16, text: vec![String::from("プ")] },
		CharEncoding { data: 173u16, text: vec![String::from("ペ")] },
		CharEncoding { data: 174u16, text: vec![String::from("ポ")] },
		CharEncoding { data: 175u16, text: vec![String::from("ヴ")] },
		CharEncoding { data: 176u16, text: vec![String::from("ー"), String::from("—"), String::from("–")] },
		CharEncoding { data: 177u16, text: vec![String::from("～"), String::from("~")] },
		CharEncoding { data: 178u16, text: vec![String::from("…")] },
		CharEncoding { data: 179u16, text: vec![String::from("、"), String::from(",")] },
		CharEncoding { data: 180u16, text: vec![String::from("。")] },
		CharEncoding { data: 181u16, text: vec![String::from("（"), String::from("(")] },
		CharEncoding { data: 182u16, text: vec![String::from("）"), String::from(")")] },
		CharEncoding { data: 183u16, text: vec![String::from("「"), String::from("“")] },
		CharEncoding { data: 184u16, text: vec![String::from("」"), String::from("”")] },
		CharEncoding { data: 185u16, text: vec![String::from("．"), String::from(".")] },
		CharEncoding { data: 186u16, text: vec![String::from("•")] },
		CharEncoding { data: 187u16, text: vec![String::from("！"), String::from("!")] },
		CharEncoding { data: 188u16, text: vec![String::from("？"), String::from("?")] },
		CharEncoding { data: 189u16, text: vec![String::from("＆"), String::from("&")] },
		CharEncoding { data: 190u16, text: vec![String::from("〇"), String::from("○")] },
		CharEncoding { data: 191u16, text: vec![String::from("✕")] },
		CharEncoding { data: 192u16, text: vec![String::from("♥")] },
		CharEncoding { data: 193u16, text: vec![String::from("☼")] },
		CharEncoding { data: 194u16, text: vec![String::from("★"), String::from("*")] },
		CharEncoding { data: 195u16, text: vec![String::from("🌀")] },
		CharEncoding { data: 196u16, text: vec![String::from("♪")] },
		CharEncoding { data: 197u16, text: vec![String::from("💢")] },
		CharEncoding { data: 198u16, text: vec![String::from("⤴")] },
		CharEncoding { data: 199u16, text: vec![String::from("⤵")] },
		CharEncoding { data: 200u16, text: vec![String::from("→")] },
		CharEncoding { data: 201u16, text: vec![String::from("←")] },
		CharEncoding { data: 202u16, text: vec![String::from("＄"), String::from("$")] },
		CharEncoding { data: 203u16, text: vec![String::from("％"), String::from("%")] },
		CharEncoding { data: 204u16, text: vec![String::from("Ａ"), String::from("A"), String::from("a")] },
		CharEncoding { data: 205u16, text: vec![String::from("Ｂ"), String::from("B"), String::from("b")] },
		CharEncoding { data: 206u16, text: vec![String::from("Ｃ"), String::from("C"), String::from("c")] },
		CharEncoding { data: 207u16, text: vec![String::from("Ｄ"), String::from("D"), String::from("d")] },
		CharEncoding { data: 208u16, text: vec![String::from("Ｅ"), String::from("E"), String::from("e")] },
		CharEncoding { data: 209u16, text: vec![String::from("Ｆ"), String::from("F"), String::from("f")] },
		CharEncoding { data: 210u16, text: vec![String::from("Ｇ"), String::from("G"), String::from("g")] },
		CharEncoding { data: 211u16, text: vec![String::from("Ｈ"), String::from("H"), String::from("h")] },
		CharEncoding { data: 212u16, text: vec![String::from("Ｉ"), String::from("I"), String::from("i")] },
		CharEncoding { data: 213u16, text: vec![String::from("Ｊ"), String::from("J"), String::from("j")] },
		CharEncoding { data: 214u16, text: vec![String::from("Ｋ"), String::from("K"), String::from("k")] },
		CharEncoding { data: 215u16, text: vec![String::from("Ｌ"), String::from("L"), String::from("l")] },
		CharEncoding { data: 216u16, text: vec![String::from("Ｍ"), String::from("M"), String::from("m")] },
		CharEncoding { data: 217u16, text: vec![String::from("Ｎ"), String::from("N"), String::from("n")] },
		CharEncoding { data: 218u16, text: vec![String::from("Ｏ"), String::from("O"), String::from("o")] },
		CharEncoding { data: 219u16, text: vec![String::from("Ｐ"), String::from("P"), String::from("p")] },
		CharEncoding { data: 220u16, text: vec![String::from("Ｑ"), String::from("Q"), String::from("q")] },
		CharEncoding { data: 221u16, text: vec![String::from("Ｒ"), String::from("R"), String::from("r")] },
		CharEncoding { data: 222u16, text: vec![String::from("Ｓ"), String::from("S"), String::from("s")] },
		CharEncoding { data: 223u16, text: vec![String::from("Ｔ"), String::from("T"), String::from("t")] },
		CharEncoding { data: 224u16, text: vec![String::from("Ｕ"), String::from("U"), String::from("u")] },
		CharEncoding { data: 225u16, text: vec![String::from("Ｖ"), String::from("V"), String::from("v")] },
		CharEncoding { data: 226u16, text: vec![String::from("Ｗ"), String::from("W"), String::from("w")] },
		CharEncoding { data: 227u16, text: vec![String::from("Ｘ"), String::from("X"), String::from("x")] },
		CharEncoding { data: 228u16, text: vec![String::from("Ｙ"), String::from("Y"), String::from("y")] },
		CharEncoding { data: 229u16, text: vec![String::from("Ｚ"), String::from("Z"), String::from("z")] },
		CharEncoding { data: 230u16, text: vec![String::from("¡")] },
		CharEncoding { data: 231u16, text: vec![String::from("_")] },
		CharEncoding { data: 232u16, text: vec![String::from("†")] },
		CharEncoding { data: 233u16, text: vec![String::from("😄")] },
		CharEncoding { data: 234u16, text: vec![String::from("😣")] },
		CharEncoding { data: 235u16, text: vec![String::from("😤")] },
		CharEncoding { data: 236u16, text: vec![String::from("😑")] },
		CharEncoding { data: 237u16, text: vec![String::from("😵")] },
		CharEncoding { data: 238u16, text: vec![String::from("😢")] },
		CharEncoding { data: 239u16, text: vec![String::from("🐱")] },
		CharEncoding { data: 240u16, text: vec![String::from("⏱")] },
		CharEncoding { data: 241u16, text: vec![String::from("🎂")] },
		CharEncoding { data: 242u16, text: vec![String::from("🎁")] },
		CharEncoding { data: 243u16, text: vec![String::from("📱")] },
		CharEncoding { data: 244u16, text: vec![String::from("🏢")] },
		CharEncoding { data: 245u16, text: vec![String::from("💼")] },
		CharEncoding { data: 246u16, text: vec![String::from("🍙")] },
		CharEncoding { data: 247u16, text: vec![String::from("🍰")] },
		CharEncoding { data: 248u16, text: vec![String::from("✨")] },
		CharEncoding { data: 249u16, text: vec![String::from("🟥")] },
		CharEncoding { data: 250u16, text: vec![String::from("")] },
		CharEncoding { data: 251u16, text: vec![String::from("")] },
		CharEncoding { data: 252u16, text: vec![String::from("")] },
		CharEncoding { data: 253u16, text: vec![String::from("")] },
		CharEncoding { data: 254u16, text: vec![String::from("")] },
		CharEncoding { data: 255u16, text: vec![String::from("")] },
		CharEncoding { data: 256u16, text: vec![String::from("")] },
		CharEncoding { data: 61440u16, text: vec![String::from("<br>")] },
		CharEncoding { data: 61441u16, text: vec![String::from("<hr>")] }, //new page
		CharEncoding { data: 61442u16, text: vec![String::from("{username}")] },
		CharEncoding { data: 61443u16, text: vec![String::from("{charname}")] },
		CharEncoding { data: 61444u16, text: vec![String::from("{statement}")] },
		CharEncoding { data: 61445u16, text: vec![String::from("{question1}")] },
		CharEncoding { data: 61446u16, text: vec![String::from("{question2}")] },
		CharEncoding { data: 61447u16, text: vec![String::from("{variable}")] },
		CharEncoding { data: 61448u16, text: vec![String::from("{pronoun}")] },
		CharEncoding { data: 61449u16, text: vec![String::from("{nickname}")] },
		CharEncoding { data: 61450u16, text: vec![String::from("{owner}")] }
	]
}
