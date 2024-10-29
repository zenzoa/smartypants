class EditFrameDialog extends EditDialog {
	static open(groupIndex, frameIndex, frame) {
		document.getElementById('edit-dialog-title').innerText = `Edit Frame ${groupIndex}-${frameIndex}: ${frameNames[frameIndex]}`

		frame.Explicit.forEach((layer, layerIndex) => {
			EditDialog.addSectionTitle(`Layer ${layerIndex}: ${layer.layer_type}`)
			EditDialog.addIntInput('X Offset', `x-${layerIndex}`, layer.x, -128, 128)
			EditDialog.addIntInput('Y Offset', `y-${layerIndex}`, layer.y, -128, 128)
			EditDialog.addIntInput('Image ID', `image-id-${layerIndex}`, layer.image_id.entity_id, 0, U16_MAX)
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
			if (!document.getElementById(`edit-x-${layerIndex}`).checkValidity() ||
				!document.getElementById(`edit-y-${layerIndex}`).checkValidity() ||
				!document.getElementById(`edit-image-id-${layerIndex}`).checkValidity() ||
				!document.getElementById(`edit-subimage-index-${layerIndex}`).checkValidity() ||
				!document.getElementById(`edit-unknown1-${layerIndex}`).checkValidity() ||
				!document.getElementById(`edit-unknown2-${layerIndex}`).checkValidity() ||
				!document.getElementById(`edit-unknown3-${layerIndex}`).checkValidity()
			) {
				invalid = true
			}
		})
		if (!invalid) {
			const newFrame = {
				Explicit: frame.Explicit.map((layer, layerIndex) => {
					return {
						layer_type: layer.layer_type,
						x: parseInt(document.getElementById(`edit-x-${layerIndex}`).value),
						y: parseInt(document.getElementById(`edit-y-${layerIndex}`).value),
						image_id: {
							card_id: layer.image_id.card_id,
							entity_id: parseInt(document.getElementById(`edit-image-id-${layerIndex}`).value)
						},
						subimage_index: parseInt(document.getElementById(`edit-subimage-index-${layerIndex}`).value),
						unknown1: parseInt(document.getElementById(`edit-unknown1-${layerIndex}`).value),
						unknown2: parseInt(document.getElementById(`edit-unknown2-${layerIndex}`).value),
						unknown3: parseInt(document.getElementById(`edit-unknown3-${layerIndex}`).value)
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
