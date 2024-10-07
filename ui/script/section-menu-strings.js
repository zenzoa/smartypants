const setupMenuStrings = () => {
	const menuStrings = cardData.menu_strings
	return table([
		thead([tr([
			th('ID'),
			th('Value'),
			th('Actions')
		])]),
		tbody(menuStrings.map((menuString, i) => renderMenuString(i, menuString)))
	])
}

const renderMenuString = (i, menuString) => {
	return tr({id: `menu-string-${i}`}, [
		th(i),
		td(menuString.string),
		td([
			button({
				title: 'Edit Menu String', className: 'icon', onclick: () => EditMenuStringDialog.open(i, menuString)
			}, EDIT_ICON)
		])
	])
}

const viewMenuStrings = () => {
	selectSection('menuStrings')
	contents.append(sections.menuStrings)
}
