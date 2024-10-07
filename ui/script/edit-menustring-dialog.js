class EditMenuStringDialog extends EditDialog {
	static open(i, menuString) {
		document.getElementById('edit-dialog-title').innerText = `Edit Menu String ${i}`

		EditDialog.addBigStrInput('Value', 'value', menuString.string)

		document.getElementById('edit-dialog-actions').append(
			button({ id: 'edit-cancel-button', className: 'text', title: 'Cancel', onclick: EditMenuStringDialog.close }, 'Cancel'),
		)

		document.getElementById('edit-dialog-actions').append(
			button({ id: 'edit-ok-button', className: 'text', title: 'Ok', onclick: () => EditMenuStringDialog.submit(i, menuString) }, 'Ok')
		)

		document.getElementById('edit-dialog').classList.add('open')
	}

	static submit(i, menuString) {
		if (!document.getElementById('edit-value').classList.contains('invalid')) {
			const newMenuString = document.getElementById('edit-value').value

			tauri_invoke('update_menu_string', { index: i, newMenuString }).then(result => {
				if (result != null) {
					cardData.menu_strings[i] = result
					sections.menuStrings = setupMenuStrings()
					viewMenuStrings()
				}
			})

			EditMenuStringDialog.close()
		}
	}
}
