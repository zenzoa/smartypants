class EditSceneLayerDialog extends EditDialog {
	static open(sceneIndex, layerIndex, layer) {
		document.getElementById('edit-dialog-title').innerText = `Edit Scene ${sceneIndex}: Layer ${layerIndex}`

		EditDialog.addIntInput('X Pos', 'x', layer.x, -128, 128)
		EditDialog.addIntInput('Y Pos', 'y', layer.y, -128, 128)
		EditDialog.addIdInput('Image ID', 'image-id', layer.image_id)
		EditDialog.addIntInput('Subimage Index', 'subimage-index', layer.subimage_index, 0, U16_MAX)
		EditDialog.addIntInput('Unknown 1', 'unknown1', layer.unknown1, -32767, 32767)
		EditDialog.addIntInput('Unknown 2', 'unknown2', layer.unknown2, 0, U16_MAX)
		EditDialog.addIntInput('Unknown 3', 'unknown3', layer.unknown3, 0, U16_MAX)
		EditDialog.addIntInput('Unknown 4', 'unknown4', layer.unknown4, 0, U16_MAX)
		EditDialog.addIntInput('Unknown 5', 'unknown5', layer.unknown5, 0, U16_MAX)
		EditDialog.addIntInput('Unknown 6', 'unknown6', layer.unknown6, 0, U16_MAX)
		EditDialog.addIntInput('Unknown 7', 'unknown7', layer.unknown7, 0, U16_MAX)
		EditDialog.addIntInput('Unknown 8', 'unknown8', layer.unknown8, 0, U16_MAX)
		EditDialog.addCheckbox('Flag 1', 'flag1', layer.flag1)
		EditDialog.addCheckbox('Flag 2', 'flag2', layer.flag2)
		EditDialog.addCheckbox('Flag 3', 'flag3', layer.flag3)
		EditDialog.addCheckbox('Flag 4', 'flag4', layer.flag4)

		document.getElementById('edit-dialog-actions').append(
			button({ id: 'edit-cancel-button', className: 'text', title: 'Cancel', onclick: EditSceneLayerDialog.close }, 'Cancel'),
		)

		document.getElementById('edit-dialog-actions').append(
			button({ id: 'edit-ok-button', className: 'text', title: 'Ok', onclick: () => EditSceneLayerDialog.submit(sceneIndex, layerIndex, layer) }, 'Ok')
		)

		document.getElementById('edit-dialog').classList.add('open')
	}

	static submit(sceneIndex, layerIndex, layer) {
		if (EditDialog.checkIntValue('x') &&
			EditDialog.checkIntValue('y') &&
			EditDialog.checkIdValue('image-id') &&
			EditDialog.checkIntValue('subimage-index') &&
			EditDialog.checkIntValue('unknown1') &&
			EditDialog.checkIntValue('unknown2') &&
			EditDialog.checkIntValue('unknown3') &&
			EditDialog.checkIntValue('unknown4') &&
			EditDialog.checkIntValue('unknown5') &&
			EditDialog.checkIntValue('unknown6') &&
			EditDialog.checkIntValue('unknown7') &&
			EditDialog.checkIntValue('unknown8')
		) {
			const newLayer = {
				x: EditDialog.getIntValue('x'),
				y: EditDialog.getIntValue('y'),
				image_id: EditDialog.getIdValue('image-id'),
				subimage_index: EditDialog.getIntValue('subimage-index'),
				unknown1: EditDialog.getIntValue('unknown1'),
				unknown2: EditDialog.getIntValue('unknown2'),
				unknown3: EditDialog.getIntValue('unknown3'),
				unknown4: EditDialog.getIntValue('unknown4'),
				unknown5: EditDialog.getIntValue('unknown5'),
				unknown6: EditDialog.getIntValue('unknown6'),
				unknown7: EditDialog.getIntValue('unknown7'),
				unknown8: EditDialog.getIntValue('unknown8'),
				flag1: EditDialog.getCheckboxValue('flag1'),
				flag2: EditDialog.getCheckboxValue('flag2'),
				flag3: EditDialog.getCheckboxValue('flag3'),
				flag4: EditDialog.getCheckboxValue('flag4')
			}

			tauri_invoke('update_scene_layer', { sceneIndex, layerIndex, newLayer }).then(result => {
				if (result != null) {
					cardData.data_pack.scenes[sceneIndex].layers[layerIndex] = result
					sections.scenes = setupScenes()
					viewScenes()
					const sceneEl = document.getElementById(`scene-${sceneIndex}`)
					if (sceneEl != null) {
						sceneEl.scrollIntoView()
					}
				}
			})

			EditSceneLayerDialog.close()
		}
	}
}
