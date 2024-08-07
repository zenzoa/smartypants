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
		tbody(strings.map((str, i) => tr({id: `string-${str.id.entity_id}`}, [
			th(str.id.entity_id),
			td('#' + formatHexCode(str.unknown1)),
			td(str.unknown2),
			td(str.unknown3),
			td(str.value.string)
		])))
	])
}

const viewStrings = () => {
	selectSection('strings')
	contents.append(sections.strings)
}

const importStrings = () => {
	tauri_invoke('import_strings')
}

const setupMenuStrings = () => {
	const strings = cardData.menu_strings
	return table([
		thead([tr([
			th('ID'),
			th('Value')
		])]),
		tbody(strings.map((str, i) => tr({id: `menu-string-${i}`}, [
			th(i),
			td(str.string)
		])))
	])
}

const viewMenuStrings = () => {
	selectSection('menuStrings')
	contents.append(sections.menuStrings)
}

const importMenuStrings = () => {
	tauri_invoke('import_menu_strings')
}
