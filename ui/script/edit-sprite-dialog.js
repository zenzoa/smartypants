class EditSpriteDialog extends EditDialog {
	static open(i, imageDef) {
		document.getElementById('edit-dialog-title').innerText = `Edit Image ${i}`

		EditDialog.addIntInput('First Palette ID', 'first-palette-index', imageDef.first_palette_index, 0, U16_MAX)

		document.getElementById('edit-dialog-actions').append(
			button({ id: 'edit-cancel-button', className: 'text', title: 'Cancel', onclick: EditSpriteDialog.close }, 'Cancel'),
		)

		document.getElementById('edit-dialog-actions').append(
			button({ id: 'edit-ok-button', className: 'text', title: 'Ok', onclick: () => EditSpriteDialog.submit(i) }, 'Ok')
		)

		document.getElementById('edit-dialog').classList.add('open')
	}

	static submit(i) {
		const firstPaletteIndex = document.getElementById('edit-first-palette-index').value

		tauri_invoke('update_image_def', { index: i, firstPaletteIndex }).then(result => {
			if (result != null) {
				cardData.menu_strings[i] = result
				sections.menuStrings = setupMenuStrings()
				viewMenuStrings()
			}
		})

		EditSpriteDialog.close()
	}
}
