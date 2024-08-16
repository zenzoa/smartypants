class EditDialog {
	static isOpen() {
		return document.getElementById('edit-dialog').classList.contains('open')
	}

	static open(title, inputLabel, value, onchange) {
		document.getElementById('edit-dialog').classList.add('open')
		document.getElementById('edit-dialog-title').innerText = title

		const inputEl = input({ id: 'edit-dialog-input', value })
		inputEl.addEventListener('keydown', (event) => {
			if (event.key === 'Enter') {
				onchange(inputEl.value)
			} else if (event.key === 'Escape') {
				EditDialog.close()
			} else {
				inputEl.classList.remove('invalid')
			}
		})


		EditDialog.callback = () => onchange(inputEl.value)

		document.getElementById('edit-dialog-body').append(
			label([
				span(inputLabel),
				inputEl,
				span({ className: 'validation-error' }, 'invalid')
			])
		)

		inputEl.focus()

		return inputEl
	}

	static openNumberEditor(title, inputLabel, value, fn, min, max) {
		const onchange = (newValue) => {
			const intValue = parseInt(newValue)
			if (intValue === parseFloat(newValue) && intValue >= min && intValue <= max) {
				fn(intValue)
			} else {
				inputEl.classList.add('invalid')
			}
		}

		const inputEl = EditDialog.open(title, inputLabel, value, onchange)

		inputEl.setAttribute('type', 'number')
		inputEl.setAttribute('min', min)
		inputEl.setAttribute('max', max)
		inputEl.setAttribute('step', 1)
		inputEl.classList.remove('invalid')
	}

	static openHexEditor(title, inputLabel, value, fn) {
		const onchange = (newValue) => {
			const intValue = parseInt(newValue, 16)
			if (!isNaN(intValue)) {
				fn(intValue)
			} else {
				inputEl.classList.add('invalid')
			}
		}
		const inputEl = EditDialog.open(title, inputLabel, value, onchange)

		inputEl.setAttribute('type', 'text')
		inputEl.classList.remove('invalid')
	}

	static openStringEditor(title, inputLabel, value, fn, maxLength) {
		const onchange = (newValue) => {
			const inputEl = document.getElementById('edit-dialog-input')
			tauri_invoke('validate_string', { string: newValue, maxLength }).then(result => {
				if (result[0]) {
					fn(result[1])
				} else {
					inputEl.classList.add('invalid')
				}
			})
		}

		const inputEl = EditDialog.open(title, inputLabel, value, onchange)

		inputEl.setAttribute('type', 'text')
		inputEl.classList.remove('invalid')

		const previewEl = div({ className: 'string-preview-container' }, [
			div({ id: 'preview-small-text', className: 'string-preview' }),
			div({ id: 'preview-large-text', className: 'string-preview' })
		])

		document.getElementById('edit-dialog-body').append(previewEl)

		const updatePreview = () => {
			const inputEl = document.getElementById('edit-dialog-input')
			const smallPreviewEl = document.getElementById('preview-small-text')
			const largePreviewEl = document.getElementById('preview-large-text')
			tauri_invoke('decode_string_js', { string: inputEl.value }).then(result => {
				smallPreviewEl.replaceChildren()
				largePreviewEl.replaceChildren()
				result.forEach(i => {
					if (i <= 256) {
						smallPreviewEl.append(div({ className: 'preview-letter' }, [displayImage('smallfont', i-1)]))
						largePreviewEl.append(div({ className: 'preview-letter' }, [displayImage('largefont', i-1)]))
					} else if (i === 61440) { // line break
						smallPreviewEl.append(div({ className: 'preview-line-break' }))
						largePreviewEl.append(div({ className: 'preview-line-break' }))
					} else if (i === 61441) { // page break
						smallPreviewEl.append(div({ className: 'preview-page-break' }))
						largePreviewEl.append(div({ className: 'preview-page-break' }))
					} else if (i === 61442 || i ===  61443 || i === 61447 || i === 61448) { // {username} {charname} {variable} {pronoun}
						Array(8).fill(0).forEach(_ => smallPreviewEl.append(div({ className: 'preview-blank' })))
						Array(8).fill(0).forEach(_ => largePreviewEl.append(div({ className: 'preview-blank' })))
					} else if (i === 61444 || i ===  61445 || i === 61446) { // {statement} {question1} {question2}
						Array(4).fill(0).forEach(_ => smallPreviewEl.append(div({ className: 'preview-blank' })))
						Array(4).fill(0).forEach(_ => largePreviewEl.append(div({ className: 'preview-blank' })))
					} else if (i === 61449 || i ===  61450) { // {nickname} {friend}
						Array(2).fill(0).forEach(_ => smallPreviewEl.append(div({ className: 'preview-blank' })))
						Array(2).fill(0).forEach(_ => largePreviewEl.append(div({ className: 'preview-blank' })))
					}
				})
			})
		}

		inputEl.addEventListener('keyup', updatePreview)

		updatePreview()
	}

	static close() {
		document.getElementById('edit-dialog').classList.remove('open')
		document.getElementById('edit-dialog-title').replaceChildren()
		document.getElementById('edit-dialog-body').replaceChildren()
		EditDialog.callback = () => {}
	}

	static setup() {
		document.getElementById('edit-close-button')
			.addEventListener('click', EditDialog.close)

		document.getElementById('edit-cancel-button')
			.addEventListener('click', EditDialog.close)

		document.getElementById('edit-ok-button')
			.addEventListener('click', () => EditDialog.callback())
	}

	static callback() {}
}
