class EditTamaStringDialog extends EditDialog {
	static open(i, tamaString) {
		document.getElementById('edit-dialog-title').innerText = `Edit Dialog String ${i}`

		EditDialog.addDropdown('Expression', 'expression', tamaString.expression, [
			{ title: 'Other/Image ID', value: 999 },
			{ title: '-', value: 0 },
			{ title: 'Joyful', value: 542 },
			{ title: 'Cry', value: 543 },
			{ title: 'Frown', value: 544 },
			{ title: 'Blush', value: 545 },
			{ title: 'Eyes Closed', value: 546 },
			{ title: 'Bored', value: 547 },
			{ title: 'Smiling', value: 548 },
			{ title: 'Neutral', value: 549 },
			{ title: 'Blank', value: 550 },
			{ title: 'Q&A', value: 373 },
			{ title: 'Q&A w/ Image', value: 570 },
		])
		EditDialog.addIntInput('Raw Expression', 'raw-expression', tamaString.expression, 0, U16_MAX)
		document.getElementById('edit-expression').addEventListener('change', () => {
			const newExpression = EditDialog.getDropdownValue('expression')
			document.getElementById('edit-raw-expression').value = newExpression
		})

		EditDialog.addIntInput('Field 1', 'field1', tamaString.field1, 0, U16_MAX)
		EditDialog.addIntInput('Field 2', 'field2', tamaString.field2, 0, U16_MAX)
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
		if (EditDialog.checkStrValue('value')) {
			const newTamastring = {
				id: tamaString.id,
				expression: EditDialog.getIntValue('raw-expression'),
				field1: EditDialog.getIntValue('field1'),
				field2: EditDialog.getIntValue('field2'),
				value: EditDialog.getStrValue('value')
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
