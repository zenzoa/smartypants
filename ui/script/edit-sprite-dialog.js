class EditSpriteDialog extends EditDialog {
	static open(i, imageDef) {
		document.getElementById('edit-dialog-title').innerText = `Edit Image ${i}`

		EditDialog.addIntInput('X-Offset', 'offset-x', imageDef.offset_x, -128, 128)
		EditDialog.addIntInput('Y-Offset', 'offset-y', imageDef.offset_y, -128, 128)
		if (cardData.lock_colors) {
			EditDialog.addIntInput('First Palette ID', 'first-palette-index', imageDef.first_palette_index, 0, U16_MAX)
		}

		document.getElementById('edit-dialog-actions').append(
			button({ id: 'edit-cancel-button', className: 'text', title: 'Cancel', onclick: EditSpriteDialog.close }, 'Cancel'),
		)

		document.getElementById('edit-dialog-actions').append(
			button({ id: 'edit-ok-button', className: 'text', title: 'Ok', onclick: () => EditSpriteDialog.submit(i) }, 'Ok')
		)

		document.getElementById('edit-dialog').classList.add('open')
	}

	static submit(i) {
		const offsetX = parseInt(document.getElementById('edit-offset-x').value)
		const offsetY = parseInt(document.getElementById('edit-offset-y').value)
		const firstPaletteIndex = cardData.lock_colors ? parseInt(document.getElementById('edit-first-palette-index').value) : null

		tauri_invoke('update_image_def', { index: i, offsetX, offsetY, firstPaletteIndex }).then(result => {
			if (result != null) {
				cardData.sprite_pack.image_defs[i] = result
				sections.frames = setupFrames()
				sections.sprites = setupSprites()
				viewSprites()
				const imageEl = document.getElementById(`image-${i}`)
				if (imageEl != null) {
					imageEl.scrollIntoView()
				}
			}
		})

		EditSpriteDialog.close()
	}
}
