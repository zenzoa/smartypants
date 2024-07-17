class EditDialog {
	static isOpen() {
		return document.getElementById('edit-dialog').classList.contains('open')
	}

	static open(newCallback) {
		document.getElementById('edit-dialog').classList.add('open')
		EditDialog.callback = newCallback
	}

	static close() {
		document.getElementById('edit-dialog').classList.remove('open')
		document.getElementById('edit-dialog-title').replaceChildren()
		document.getElementById('edit-dialog-contents').replaceChildren()
		EditDialog.callback = () => {}
	}

	static setup() {
		document.getElementById('edit-close-button')
			.addEventListener('click', EditDialog.close)

		document.getElementById('edit-cancel-button')
			.addEventListener('click', EditDialog.close)

		document.getElementById('edit-ok-button')
			.addEventListener('click', () => {
				EditDialog.callback()
				EditDialog.close()
			})
	}

	static setTitle(newTitle) {
		document.getElementById('edit-dialog-title').innerHTML = newTitle
	}

	static setContents(newContents) {
		document.getElementById('edit-dialog-contents').innerHTML = newContents
	}

	static callback() {
	}
}
