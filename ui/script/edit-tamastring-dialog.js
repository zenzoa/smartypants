class EditTamaStringDialog extends EditDialog {
	static open(i, tamaString) {
		document.getElementById('edit-dialog-title').innerText = `Edit Dialog String ${i}`

		EditDialog.addIntInput('Unkown 1', 'unknown1', tamaString.unknown1, 0, U16_MAX)
		EditDialog.addIntInput('Unkown 2', 'unknown2', tamaString.unknown2, 0, U16_MAX)
		EditDialog.addIntInput('Unkown 3', 'unknown3', tamaString.unknown3, 0, U16_MAX)
		EditDialog.addBigStrInput('Value', 'value', tamaString.value.string)

		document.getElementById('edit-dialog-actions').append(
			button({ id: 'edit-cancel-button', className: 'text', title: 'Cancel', onclick: EditTamaStringDialog.close }, 'Cancel'),
		)

		document.getElementById('edit-dialog-actions').append(
			button({ id: 'edit-ok-button', className: 'text', title: 'Ok', onclick: () => EditTamaStringDialog.submit(i, tamaString) }, 'Ok')
		)

		document.getElementById('edit-dialog').classList.add('open')
	}

	static submit(i, tamaString) {
		if (!document.getElementById('edit-value').classList.contains('invalid')) {
			const newTamastring = {
				id: tamaString.id,
				unknown1: parseInt(document.getElementById('edit-unknown1').value),
				unknown2: parseInt(document.getElementById('edit-unknown2').value),
				unknown3: parseInt(document.getElementById('edit-unknown3').value),
				value: { data: [], string: document.getElementById('edit-value').value }
			}

			tauri_invoke('update_tamastring', { index: i, newTamastring }).then(result => {
				if (result != null) {
					cardData.data_pack.tamastrings[i] = result
					sections.tamaStrings = setupTamaStrings()
					viewTamaStrings()
				}
			})

			EditTamaStringDialog.close()
		}
	}
}
