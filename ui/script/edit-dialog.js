class EditDialog {
	static isOpen() {
		return document.getElementById('edit-dialog').classList.contains('open')
	}

	static open() {
		document.getElementById('edit-dialog').classList.add('open')
	}

	static close() {
		document.getElementById('edit-dialog').classList.remove('open')
		document.getElementById('edit-dialog-title').replaceChildren()
		document.getElementById('edit-dialog-body').replaceChildren()
		document.getElementById('edit-dialog-actions').replaceChildren()
	}

	static setup() {
		document.getElementById('edit-close-button')
			.addEventListener('click', EditDialog.close)
	}

	static addSectionTitle(title) {
		document.getElementById('edit-dialog-body').append(
			div({ className: 'dialog-section-title' }, title)
		)
	}

	static addDescription(name, value) {
		document.getElementById('edit-dialog-body').append(
			div({ id: name, className: 'dialog-description' }, value)
		)
	}

	static addStrInput(title, name, value, maxlength) {
		document.getElementById('edit-dialog-body').append(
			label({ id: `label-${name}` }, [
				span(title),
				input({
					id: `edit-${name}`, value,
					onkeyup: () => EditDialog.updateStringPreview(name),
					onchange: (event) => EditDialog.validateString(event, name)
				}),
				div({ id: `${name}-preview-large`, className: 'string-preview string-preview-large' }),
				span({ id: `edit-${name}-invalid`, className: 'validation-error' }, 'invalid characters')
			])
		)
		EditDialog.updateStringPreview(name)
	}

	static checkStrValue(name) {
		return !document.getElementById(`edit-${name}`).classList.contains('invalid')
	}

	static getStrValue(name) {
		return {
			data: [],
			string: document.getElementById(`edit-${name}`).value
		}
	}

	static addBigStrInput(title, name, value) {
		document.getElementById('edit-dialog-body').append(
			label({ id: `label-${name}` }, [
				span(title),
				input({
					id: `edit-${name}`, className: 'fill', value,
					onkeyup: () => EditDialog.updateStringPreview(name),
					onchange: (event) => EditDialog.validateString(event, name)
				}),
				span({ id: `edit-${name}-invalid`, className: 'validation-error' }, 'invalid characters')
			])
		)
		document.getElementById('edit-dialog-body').append(
			div({ className: `string-preview-container` }, [
				div({ id: `${name}-preview-small`, className: 'string-preview string-preview-small' }),
				div({ id: `${name}-preview-large`, className: 'string-preview string-preview-large' })
			])
		)
		EditDialog.updateStringPreview(name)
	}

	static addIntInput(title, name, value, min, max) {
		document.getElementById('edit-dialog-body').append(
			label({ id: `label-${name}` }, [
				span(title),
				input({ id: `edit-${name}`, type: 'number', step: 1, min, max, value })
			])
		)
	}

	static checkIntValue(name) {
		return document.getElementById(`edit-${name}`).checkValidity()
	}

	static getIntValue(name) {
		return parseInt(document.getElementById(`edit-${name}`).value)
	}

	static addDropdown(title, name, value, list) {
		document.getElementById('edit-dialog-body').append(
			label({ id: `label-${name}` }, [
				span(title),
				div({ className: 'select-wrapper' }, [
					select({ id: `edit-${name}` },
						list.map(opt => {
							if (value === opt.value) {
								return option({ value: opt.value, selected: 'selected' }, opt.title)
							} else {
								return option({ value: opt.value }, opt.title)
							}
						})
					),
					div({ className: 'select-arrow' }, '▼')
				])
			])
		)
	}

	static getDropdownValue(name) {
		return document.getElementById(`edit-${name}`).value
	}

	static addCheckbox(title, name, value) {
		const checkbox = button({ id: `edit-${name}`, className: value ? 'toggle on' : 'toggle off' })

		checkbox.addEventListener('click', () => {
			if (checkbox.className === 'toggle on') {
				checkbox.className = 'toggle off'
			} else {
				checkbox.className = 'toggle on'
			}
		})

		document.getElementById('edit-dialog-body').append(
			label({ id: `label-${name}` }, [ span(title), checkbox ])
		)

		return checkbox
	}

	static getCheckboxValue(name) {
		return document.getElementById(`edit-${name}`).classList.contains('on')
	}

	static addIdInput(title, name, id) {
		const cardId = id == null ? -1 : (id.card_id == null ? -1 : id.card_id)
		const entityId = id == null ? -1 : id.entity_id
		document.getElementById('edit-dialog-body').append(
			label({ id: `label-${name}`, className: 'entity-id-input' }, [
				span(title),
				input({ id: `edit-${name}-card-id`, type: 'number', step: 1, min: -1, max: U16_MAX, value: cardId }),
				input({ id: `edit-${name}-entity-id`, type: 'number', step: 1, min: -1, max: U16_MAX, value: entityId })
			])
		)
	}

	static checkIdValue(name) {
		return (
			document.getElementById(`edit-${name}-card-id`).checkValidity() &&
			document.getElementById(`edit-${name}-entity-id`).checkValidity()
		)
	}

	static getIdValue(name) {
		let card_id = parseInt(document.getElementById(`edit-${name}-card-id`).value)
		const entity_id = parseInt(document.getElementById(`edit-${name}-entity-id`).value)
		if (card_id < 0) card_id = null
		if (entity_id < 0) return null
		return { card_id, entity_id }
	}

	static updateStringPreview(name) {
		const inputEl = document.getElementById(`edit-${name}`)
		const smallPreviewEl = document.getElementById(`${name}-preview-small`)
		const largePreviewEl = document.getElementById(`${name}-preview-large`)
		tauri_invoke('decode_string_js', { string: inputEl.value }).then(result => {
			if (smallPreviewEl) smallPreviewEl.replaceChildren()
			if (largePreviewEl) largePreviewEl.replaceChildren()
			result.forEach(i => {
				if (i <= 256) {
					if (smallPreviewEl) smallPreviewEl.append(div({ className: 'preview-letter' }, [displayLetter('smallfont', i-1)]))
					if (largePreviewEl) largePreviewEl.append(div({ className: 'preview-letter' }, [displayLetter('largefont', i-1)]))
				} else if (i === 61440) { // line break
					if (smallPreviewEl) smallPreviewEl.append(div({ className: 'preview-line-break' }))
					if (largePreviewEl) largePreviewEl.append(div({ className: 'preview-line-break' }))
				} else if (i === 61441) { // page break
					if (smallPreviewEl) smallPreviewEl.append(div({ className: 'preview-page-break' }))
					if (largePreviewEl) largePreviewEl.append(div({ className: 'preview-page-break' }))
				} else if (i === 61442 || i ===  61443 || i === 61447 || i === 61448) { // {username} {charname} {variable} {pronoun}
					if (smallPreviewEl) Array(8).fill(0).forEach(_ => smallPreviewEl.append(div({ className: 'preview-blank' })))
					if (largePreviewEl) Array(8).fill(0).forEach(_ => largePreviewEl.append(div({ className: 'preview-blank' })))
				} else if (i === 61444 || i ===  61445 || i === 61446) { // {statement} {question1} {question2}
					if (smallPreviewEl) Array(4).fill(0).forEach(_ => smallPreviewEl.append(div({ className: 'preview-blank' })))
					if (largePreviewEl) Array(4).fill(0).forEach(_ => largePreviewEl.append(div({ className: 'preview-blank' })))
				} else if (i === 61449 || i ===  61450) { // {nickname} {friend}
					if (smallPreviewEl) Array(2).fill(0).forEach(_ => smallPreviewEl.append(div({ className: 'preview-blank' })))
					if (largePreviewEl) Array(2).fill(0).forEach(_ => largePreviewEl.append(div({ className: 'preview-blank' })))
				}
			})
		})
	}

	static validateString(event, name) {
		tauri_invoke('validate_string', { string: event.target.value }).then(result => {
			const inputEl = document.getElementById(`edit-${name}`)
			if (inputEl) {
				if (result) {
					inputEl.classList.remove('invalid')
				} else {
					inputEl.classList.add('invalid')
				}
			}
		})
	}
}
