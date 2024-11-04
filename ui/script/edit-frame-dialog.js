class EditFrameDialog extends EditDialog {
	static open(groupIndex, frameIndex, frame) {
		document.getElementById('edit-dialog-title').innerText = `Edit Frame ${groupIndex}-${frameIndex}: ${frameNames[frameIndex]}`

		frame.Explicit.forEach((layer, layerIndex) => {
			EditDialog.addSectionTitle(`Layer ${layerIndex}: ${layer.layer_type}`)
			EditDialog.addIntInput('X Offset', `x-${layerIndex}`, layer.x, -128, 128)
			EditDialog.addIntInput('Y Offset', `y-${layerIndex}`, layer.y, -128, 128)
			EditDialog.addIdInput('Image ID', `image-id-${layerIndex}`, layer.image_id)
			EditDialog.addIntInput('Subimage Index', `subimage-index-${layerIndex}`, layer.subimage_index, 0, U16_MAX)
			EditDialog.addIntInput('Unknown 1', `unknown1-${layerIndex}`, layer.unknown1, 0, U16_MAX)
			EditDialog.addIntInput('Unknown 2', `unknown2-${layerIndex}`, layer.unknown2, 0, U16_MAX)
			EditDialog.addIntInput('Unknown 3', `unknown3-${layerIndex}`, layer.unknown3, 0, U16_MAX)
		})

		document.getElementById('edit-dialog-actions').append(
			button({ id: 'edit-cancel-button', className: 'text', title: 'Cancel', onclick: EditFrameDialog.close }, 'Cancel'),
		)

		document.getElementById('edit-dialog-actions').append(
			button({ id: 'edit-ok-button', className: 'text', title: 'Ok', onclick: () => EditFrameDialog.submit(groupIndex, frameIndex, frame) }, 'Ok')
		)

		document.getElementById('edit-dialog').classList.add('open')
	}

	static submit(groupIndex, frameIndex, frame) {
		let invalid = false
		frame.Explicit.forEach((_, layerIndex) => {
			if (!(
				EditDialog.checkIntValue(`x-${layerIndex}`) &&
				EditDialog.checkIntValue(`y-${layerIndex}`) &&
				EditDialog.checkIdValue(`image-id-${layerIndex}`) &&
				EditDialog.checkIntValue(`subimage-index-${layerIndex}`) &&
				EditDialog.checkIntValue(`unknown1-${layerIndex}`) &&
				EditDialog.checkIntValue(`unknown2-${layerIndex}`) &&
				EditDialog.checkIntValue(`unknown3-${layerIndex}`)
			)) {
				invalid = true
			}
		})

		if (!invalid) {
			const newFrame = {
				Explicit: frame.Explicit.map((layer, layerIndex) => {
					return {
						layer_type: layer.layer_type,
						x: EditDialog.getIntValue(`x-${layerIndex}`),
						y: EditDialog.getIntValue(`y-${layerIndex}`),
						image_id: EditDialog.getIdValue(`image-id-${layerIndex}`),
						subimage_index: EditDialog.getIntValue(`subimage-index-${layerIndex}`),
						unknown1: EditDialog.getIntValue(`unknown1-${layerIndex}`),
						unknown2: EditDialog.getIntValue(`unknown2-${layerIndex}`),
						unknown3: EditDialog.getIntValue(`unknown3-${layerIndex}`)
					}
				})
			}

			tauri_invoke('update_frame', { groupIndex, frameIndex, newFrame }).then(result => {
				if (result != null) {
					cardData.data_pack.frame_groups[groupIndex].frames[frameIndex] = result
					sections.frames = setupFrames()
					viewFrames()
					const frameEl = document.getElementById(`frame-${groupIndex}-${frameIndex}`)
					if (frameEl != null) {
						frameEl.scrollIntoView()
					}
				}
			})

			EditFrameDialog.close()
		}
	}
}
