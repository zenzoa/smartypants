const setupStrings = () => {
	const strings = cardData.data_pack.strings
	return table([
		thead([tr([
			th('ID'),
			th('Unknown 1'),
			th('Unknown 2'),
			th('Unknown 3'),
			th('Value')
		])]),
		tbody(strings.map(str => tr({id: `string-${str.id.entity_id}`}, [
			th(str.id.entity_id),
			td('#' + formatHexCode(str.unknown1)),
			td(str.unknown2),
			td(str.unknown3),
			td(str.value)
		])))
	])
}

const viewStrings = () => {
	selectSection('view-strings-button')
	contents.append(sections.strings)
}
