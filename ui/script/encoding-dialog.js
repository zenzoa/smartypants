class EncodingDialog {
	static isOpen() {
		return document.getElementById('encoding-dialog').classList.contains('open')
	}

	static open() {
		document.getElementById('encoding-dialog-body').replaceChildren()

		document.getElementById('encoding-dialog-body').append(
			table([
				tbody(textEncoding.map(charCode =>
					tr([
						th(charCode.data),
						td([displayImage('smallfont', charCode.data-1)]),
						td([displayImage('largefont', charCode.data-1)]),
						td([
							input({ id: `encoding-${charCode.data}`, value: charCode.text.join(', ') })
						])
					])
				))
			])
		)

		document.getElementById('encoding-dialog').classList.add('open')

		document.getElementById('encoding-1').focus()

		document.getElementById('spinner').classList.remove('on')
	}

	static close() {
		document.getElementById('encoding-dialog').classList.remove('open')
		document.getElementById('encoding-dialog-body').replaceChildren()
	}

	static update_encoding(afterSuccess) {
		const newCharCodes = textEncoding.map(charCode => {
			const input = document.getElementById(`encoding-${charCode.data}`)
			if (input != null) {
				input.classList.remove('invalid')
				const text = input.value.split(', ')
				return { data: charCode.data, text }
			} else {
				return charCode
			}
		})
		tauri_invoke('update_char_codes', { newCharCodes }).then(result => {
			if (result[1].length == 0) {
				textEncoding = result[0].slice(1, 257)
				afterSuccess()
			} else {
				result[1].forEach(char_code => {
					const input = document.getElementById(`encoding-${char_code}`)
					input.classList.add('invalid')
				})
			}
		})
	}

	static setup() {
		document.getElementById('encoding-close-button')
			.addEventListener('click', EncodingDialog.close)

		document.getElementById('encoding-import-button')
			.addEventListener('click', () => tauri_invoke("import_encoding"))

		document.getElementById('encoding-export-button')
			.addEventListener('click', () =>
				EncodingDialog.update_encoding(() => tauri_invoke("export_encoding"))
			)

		document.getElementById('encoding-cancel-button')
			.addEventListener('click', EncodingDialog.close)

		document.getElementById('encoding-ok-button')
			.addEventListener('click', () =>
				EncodingDialog.update_encoding(EncodingDialog.close)
			)

		tauri_listen('show_encoding_dialog', () => {
			document.getElementById('spinner').classList.add('on')
			const p = new Promise(() => setTimeout(EncodingDialog.open, 100))
			p.then()
		})
	}
}
