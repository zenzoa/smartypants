use std::fs;
use std::error::Error;
use std::sync::Mutex;

use tauri::{ AppHandle, Manager, State, Emitter };
use tauri::menu::{ MenuItemKind, CheckMenuItem };
use tauri::path::BaseDirectory;

#[derive(serde::Serialize)]
pub struct ConfigState {
	pub theme: Mutex<String>,
	pub show_toolbar: Mutex<bool>
}

impl Default for ConfigState {
	fn default() -> Self {
		ConfigState {
			theme: Mutex::new(String::new()),
			show_toolbar: Mutex::new(true)
		}
	}
}

#[derive(serde::Serialize)]
struct IniProp {
	key: String,
	value: String
}

#[tauri::command]
pub fn load_config(handle: AppHandle) {
	if let Ok(config_path) = handle.path().resolve("resources/config.ini", BaseDirectory::Resource) {
		if let Ok(config_contents) = fs::read_to_string(config_path) {
			let props = read_ini_file(&config_contents);
			for prop in props {
				match prop.key.as_str() {
					"theme" => {
						set_theme(&handle, &prop.value).unwrap();
					},
					"show_toolbar" => {
						set_toolbar_visibility(&handle, Some(prop.value == "true"));
					},
					_ => ()
				}
			}
		}
	}
}

pub fn save_config(handle: &AppHandle) -> Result<(), Box<dyn Error>> {
	let config_state: State<ConfigState> = handle.state();
	let config_path = handle.path().resolve("resources/config.ini", BaseDirectory::Resource)?;
	fs::write(config_path, format!(
		"theme = {}\nshow_toolbar = {}",
		config_state.theme.lock().unwrap(),
		if *config_state.show_toolbar.lock().unwrap() { "true" } else { "false" }
	))?;
	Ok(())
}

pub fn get_themes(handle: &AppHandle) -> Result<(), Box<dyn Error>> {
	let theme_dir = handle.path().resolve("resources/themes", BaseDirectory::Resource)?;
	if theme_dir.is_dir() {
		for entry in fs::read_dir(theme_dir)? {
			let entry = entry?;
			if let Some(theme) = entry.path().file_stem() {
				if let Some(theme) = theme.to_str() {
					if let Some(menu) = handle.menu() {
						if let Some(MenuItemKind::Submenu(view_menu)) = menu.get("view") {
							if let Some(MenuItemKind::Submenu(theme_menu)) = view_menu.get("theme") {
								theme_menu.append(
									&CheckMenuItem::with_id(handle, format!("theme_{}", theme), theme, true, false, None::<&str>)?
								)?;
							}
						}
					}
				}
			}
		}
	}
	Ok(())
}

pub fn set_theme(handle: &AppHandle, new_theme: &str) -> Result<(), Box<dyn Error>> {
	let theme_dir = handle.path().resolve("resources/themes", BaseDirectory::Resource)?;
	let theme_path = theme_dir.join(format!("{}.ini", new_theme));
	let theme_contents = fs::read_to_string(theme_path)?;
	let props = read_ini_file(&theme_contents);

	let config_state: State<ConfigState> = handle.state();
	*config_state.theme.lock().unwrap() = new_theme.to_string();

	handle.emit("update_theme", &props).unwrap();

	if let Some(menu) = handle.menu() {
		if let Some(MenuItemKind::Submenu(view_menu)) = menu.get("view") {
			if let Some(MenuItemKind::Submenu(theme_menu)) = view_menu.get("theme") {
				for menu_item in theme_menu.items()? {
					if let Some(check_menu_item) = menu_item.as_check_menuitem() {
						if check_menu_item.text()? == new_theme {
							check_menu_item.set_checked(true)?;
						} else {
							check_menu_item.set_checked(false)?;

						}
					}
				}
			}
		}
	}

	save_config(handle).unwrap();

	Ok(())
}

pub fn set_toolbar_visibility(handle: &AppHandle, new_value: Option<bool>) {
	let config_state: State<ConfigState> = handle.state();
	let new_value = if let Some(val) = new_value { val } else {
		!*config_state.show_toolbar.lock().unwrap()
	};
	*config_state.show_toolbar.lock().unwrap() = new_value;

	if let Some(menu) = handle.menu() {
		if let Some(MenuItemKind::Submenu(view_menu)) = menu.get("view") {
			if let Some(MenuItemKind::Check(show_toolbar_menu_item)) = view_menu.get("show_toolbar") {
				show_toolbar_menu_item.set_checked(new_value).unwrap();
			}
		}
	}

	handle.emit("update_toolbar_visibility", new_value).unwrap();

	save_config(handle).unwrap();
}

fn read_ini_file(contents: &str) -> Vec<IniProp> {
	let mut props = Vec::new();
	let lines = contents.split('\n').collect::<Vec<&str>>();
	for line in lines {
		if !line.starts_with(';') && !line.starts_with('[') {
			if let Some((key, value)) = line.split_once('=') {
				props.push(IniProp{
					key: key.trim().to_string(),
					value: value.trim().replace('"', "").to_string()
				});
			}
		}
	}
	props
}
