const setupMenuStrings = () => {
	const menuStrings = cardData.menu_strings
	return table([
		thead([tr([
			th('ID'),
			th('Value')
		])]),
		tbody(menuStrings.map((menuString, i) => renderMenuString(i, menuString)))
	])
}

const renderMenuString = (i, menuString) => {
	return tr({id: `menu-string-${i}`}, [
		th(i),
		td([
			span(menuString.string),
			button({className: 'edit', onclick: editMenuString.bind(this, i)}, '✏️')
		])
	])
}

const viewMenuStrings = () => {
	selectSection('menuStrings')
	contents.append(sections.menuStrings)
}

const editMenuString = (i) => {
	const menuString = cardData.menu_strings[i]
	EditDialog.openStringEditor(
		`Edit String ${i}`,
		'Value:',
		menuString.string,
		(newValue) => {
			tauri_invoke('update_menu_string', { index: i, name: newValue }).then(result => {
				if (result != null) cardData.menu_strings[i] = result
				const menuStringEl = document.getElementById(`menu-string-${i}`)
				if (menuStringEl != null) menuStringEl.replaceWith(renderMenuString(i, cardData.menu_strings[i]))
			})
			EditDialog.close()
		},
		999
	)
}
