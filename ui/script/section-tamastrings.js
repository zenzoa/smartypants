const setupTamaStrings = () => {
	const tamaString = cardData.data_pack.tamastrings
	return table([
		thead([tr([
			th('ID'),
			th('Expression'),
			th('Field 1'),
			th('Field 2'),
			th('Value'),
			th('Actions')
		])]),
		tbody(tamaString.map((tamaString, i) => renderTamaString(i, tamaString)))
	])
}

const renderTamaString = (i, tamaString) => {
	let expression = tamaString.expression
	if (tamaString.expression == 0) {
		expression = '-'
	} else if (tamaString.expression == 373) {
		expression = 'Q&A'
	} else if (tamaString.expression == 542) {
		expression = 'Joyful'
	} else if (tamaString.expression == 543) {
		expression = 'Cry'
	} else if (tamaString.expression == 544) {
		expression = 'Frown'
	} else if (tamaString.expression == 545) {
		expression = 'Blush'
	} else if (tamaString.expression == 546) {
		expression = 'Eyes Closed'
	} else if (tamaString.expression == 547) {
		expression = 'Bored'
	} else if (tamaString.expression == 548) {
		expression = 'Smiling'
	} else if (tamaString.expression == 549) {
		expression = 'Neutral'
	} else if (tamaString.expression == 550) {
		expression = 'Blank'
	} else if (tamaString.expression == 570) {
		expression = 'Q&A w/ Image'
	}

	return tr({id: `tamastring-${tamaString.id.entity_id}`}, [
		th(tamaString.id.entity_id),
		td(expression),
		td(tamaString.field1),
		td(tamaString.field2),
		td(tamaString.value.string),
		td([
			button({
				title: 'Edit Dialog String', className: 'icon',
				onclick: () => EditTamaStringDialog.open(i, tamaString)
			}, EDIT_ICON)
		])
	])
}

const viewTamaStrings = () => {
	selectSection('tamaStrings')
	contents.append(sections.tamaStrings)
}
