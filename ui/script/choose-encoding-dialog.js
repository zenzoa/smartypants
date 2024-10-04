class ChooseEncodingDialog {
	static isOpen() {
		return document.getElementById('choose-encoding-dialog').classList.contains('open')
	}

	static open() {
		document.getElementById('choose-encoding-jp').className = cardData.encoding_language === 'Japanese' ? 'toggle on' : 'toggle off'
		document.getElementById('choose-encoding-en').className = cardData.encoding_language === 'English' ? 'toggle on' : 'toggle off'
		document.getElementById('choose-encoding-latin').className = cardData.encoding_language === 'Latin' ? 'toggle on' : 'toggle off'
		document.getElementById('choose-encoding-custom').className = cardData.encoding_language === 'Custom' ? 'toggle on' : 'toggle off'

		document.getElementById('choose-encoding-dialog').classList.add('open')
		document.getElementById('choose-encoding-ok-button').focus()
	}

	static close() {
		document.getElementById('choose-encoding-dialog').classList.remove('open')
	}

	static setup() {
		document.getElementById('choose-encoding-close-button')
			.addEventListener('click', ChooseEncodingDialog.close)

		document.getElementById('choose-encoding-ok-button')
			.addEventListener('click', ChooseEncodingDialog.close)

		document.getElementById('choose-encoding-jp')
			.addEventListener('click', () => {
				tauri_invoke('set_to_preset_encoding', { name: 'jp' })
				document.getElementById('choose-encoding-jp').className = 'toggle on'
				document.getElementById('choose-encoding-en').className = 'toggle off'
				document.getElementById('choose-encoding-latin').className = 'toggle off'
				document.getElementById('choose-encoding-custom').className = 'toggle off'
			})

		document.getElementById('choose-encoding-en')
			.addEventListener('click', () => {
				tauri_invoke('set_to_preset_encoding', { name: 'en' })
				document.getElementById('choose-encoding-jp').className = 'toggle off'
				document.getElementById('choose-encoding-en').className = 'toggle on'
				document.getElementById('choose-encoding-latin').className = 'toggle off'
				document.getElementById('choose-encoding-custom').className = 'toggle off'
			})

		document.getElementById('choose-encoding-latin')
			.addEventListener('click', () => {
				tauri_invoke('set_to_preset_encoding', { name: 'latin' })
				document.getElementById('choose-encoding-jp').className = 'toggle off'
				document.getElementById('choose-encoding-en').className = 'toggle off'
				document.getElementById('choose-encoding-latin').className = 'toggle on'
				document.getElementById('choose-encoding-custom').className = 'toggle off'
			})

		document.getElementById('choose-encoding-custom')
			.addEventListener('click', () => {
				tauri_invoke('import_encoding', { name: 'custom' })
				document.getElementById('choose-encoding-jp').className = 'toggle off'
				document.getElementById('choose-encoding-en').className = 'toggle off'
				document.getElementById('choose-encoding-latin').className = 'toggle off'
				document.getElementById('choose-encoding-custom').className = 'toggle on'
			})

		document.getElementById('choose-encoding-edit')
			.addEventListener('click', () => {
					document.getElementById('spinner').classList.add('on')
					setTimeout(() => {
						EditEncodingDialog.open()
						document.getElementById('spinner').classList.remove('on')
					}, 100)
				})

		tauri_listen('show_choose_encoding_dialog', () => {
			ChooseEncodingDialog.open()
		})
	}
}
