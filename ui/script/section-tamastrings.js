const setupTamaStrings = () => {
	const tamaString = cardData.data_pack.tamastrings
	return table([
		thead([tr([
			th('ID'),
			th('Unknown 1'),
			th('Unknown 2'),
			th('Unknown 3'),
			th('Value'),
			th('Actions')
		])]),
		tbody(tamaString.map((tamaString, i) => renderTamaString(i, tamaString)))
	])
}

const renderTamaString = (i, tamaString) => {
	return tr({id: `tamastring-${tamaString.id.entity_id}`}, [
		th(tamaString.id.entity_id),
		td(tamaString.unknown1),
		td(tamaString.unknown2),
		td(tamaString.unknown3),
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
