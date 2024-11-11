class EditCardIDDialog extends EditDialog {
	static open() {
		document.getElementById('edit-dialog-title').innerText = 'Edit Card ID'

		EditDialog.addIntInput('Card ID', 'card-id', cardData.card_header.card_id, 0, 256)

		document.getElementById('edit-dialog-actions').append(
			button({ id: 'edit-cancel-button', className: 'text', title: 'Cancel', onclick: EditCardIDDialog.close }, 'Cancel'),
		)
		document.getElementById('edit-dialog-actions').append(
			button({ id: 'edit-ok-button', className: 'text', title: 'Ok', onclick: EditCardIDDialog.submit }, 'Ok')
		)
		document.getElementById('edit-dialog').classList.add('open')
	}

	static submit() {
		if (EditDialog.checkIntValue('card-id')) {
			const newCardId = EditDialog.getIntValue('card-id')
			tauri_invoke('update_card_id', { newCardId })
			EditCardIDDialog.close()
		}
	}
}

class EditBuildDateDialog extends EditDialog {
	static open() {
		document.getElementById('edit-dialog-title').innerText = 'Edit Build Date'

		EditDialog.addIntInput('Year', 'year', cardData.card_header.year, 0, 3000)
		EditDialog.addIntInput('Month', 'month', cardData.card_header.month, 0, 12)
		EditDialog.addIntInput('Day', 'day', cardData.card_header.day, 0, 31)
		EditDialog.addIntInput('Revision', 'revision', cardData.card_header.revision, 0, 256)

		document.getElementById('edit-dialog-actions').append(
			button({ id: 'edit-cancel-button', className: 'text', title: 'Cancel', onclick: EditBuildDateDialog.close }, 'Cancel'),
		)
		document.getElementById('edit-dialog-actions').append(
			button({ id: 'edit-ok-button', className: 'text', title: 'Ok', onclick: EditBuildDateDialog.submit }, 'Ok')
		)
		document.getElementById('edit-dialog').classList.add('open')
	}

	static submit() {
		if (EditDialog.checkIntValue('year') &&
			EditDialog.checkIntValue('month') &&
			EditDialog.checkIntValue('day') &&
			EditDialog.checkIntValue('revision')
		) {
			const newYear = EditDialog.getIntValue('year')
			const newMonth = EditDialog.getIntValue('month')
			const newDay = EditDialog.getIntValue('day')
			const newRevision = EditDialog.getIntValue('revision')
			tauri_invoke('update_build_date', { newYear, newMonth, newDay, newRevision })
			EditBuildDateDialog.close()
		}
	}
}
