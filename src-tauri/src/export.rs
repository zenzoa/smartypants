use std::error::Error;
use std::path::PathBuf;
use std::fs::File;
use std::io::prelude::*;
use crate::DataState;
use crate::ImageState;

pub fn export_data_to(data_state: &DataState, path: &PathBuf) -> Result<(), Box<dyn Error>> {
	if let Some(data_pack) = data_state.data_pack.lock().unwrap().clone() {
		let serialized = serde_json::to_string(&data_pack)?;
		let mut file = File::create(path)?;
		file.write_all(serialized.as_bytes())?;
		Ok(())
	} else {
		Err("No data to export".into())
	}
}

pub fn export_images_to(image_state: &ImageState, path: &PathBuf) -> Result<(), Box<dyn Error>> {
	let base_name = path.file_stem().unwrap().to_string_lossy();

	let image_count = image_state.images.lock().unwrap().len();
	for i in 0..image_count {
		let subimage_count = image_state.images.lock().unwrap().get(i).ok_or("")?.len();
		for j in 0..subimage_count {
			let img_path = path.with_file_name(&format!("{}-{}-{}", base_name, i, j)).with_extension("png");
			image_state.images.lock().unwrap()
				.get(i).ok_or("image not found")?
				.get(j).ok_or("subimage not found")?
				.save(img_path)?;
		}
	}

	Ok(())
}
