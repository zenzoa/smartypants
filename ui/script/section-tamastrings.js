const setupTamaStrings = () => {
	const tamaString = cardData.data_pack.tamastrings
	return table([
		thead([tr([
			th('ID'),
			th('Unknown 1'),
			th('Unknown 2'),
			th('Unknown 3'),
			th('Value')
		])]),
		tbody(tamaString.map((tamaString, i) => renderTamaString(i, tamaString)))
	])
}

const renderTamaString = (i, tamaString) => {
	return tr({id: `tamastring-${tamaString.id.entity_id}`}, [
		th(tamaString.id.entity_id),
		td('#' + formatHexCode(tamaString.unknown1)),
		td(tamaString.unknown2),
		td(tamaString.unknown3),
		td([
			span(tamaString.value.string),
			button({className: 'edit', onclick: editTamaString.bind(this, i)}, '✏️')
		])
	])
}

const viewTamaStrings = () => {
	selectSection('tamaStrings')
	contents.append(sections.tamaStrings)
}

const editTamaString = (i) => {
	const tamaString = cardData.data_pack.tamastrings[i]
	EditDialog.openStringEditor(
		`Edit String ${i}`,
		'Value:',
		tamaString.value.string,
		(newValue) => {
			tauri_invoke('update_tamastring', { index: i, name: newValue }).then(result => {
				if (result != null) cardData.data_pack.tamastrings[i].value = result
				const tamaStringEl = document.getElementById(`tamastring-${i}`)
				if (tamaStringEl != null) tamaStringEl.replaceWith(renderTamaString(i, cardData.data_pack.tamastrings[i]))
			})
			EditDialog.close()
		},
		999
	)
}
