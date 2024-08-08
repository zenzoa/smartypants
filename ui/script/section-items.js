const setupItems = () => {
	const items = cardData.data_pack.items
	return table([
		thead([tr([
			th('ID'),
			th('Type'),
			th('Name'),
			th('Image ID'),
			th('Image ID (Worn)'),
			th('Image ID (Close)'),
			th('Unknown 1'),
			th('Price'),
			th('Unknown 2'),
			th('Unknown 3'),
			th('Unknown 4'),
			th('Unlocked Character'),
			th('Game Type')
		])]),
		tbody(items.map((item, i) => renderItem(i, item)))
	])
}

const renderItem = (i, item) => {
	return tr({id: `item-${item.id.entity_id}`}, [
		th(item.id.entity_id),
		td(item.item_type),
		td([
			span(item.name.string),
			button({className: 'edit', onclick: editItemName.bind(this, i)}, '✏️')
		]),
		td(item.item_type === 'Game' ? '-' : displayImageWithLink(item.image_id, 0)),
		td(displayImageWithLink(item.worn_image_id, 0)),
		td(displayImageWithLink(item.close_image_id, 0)),
		td('#' + formatHexCode(item.unknown1)),
		td(item.price),
		td('#' + formatHexCode(item.unknown2)),
		td('#' + formatHexCode(item.unknown3)),
		td('#' + formatHexCode(item.unknown4)),
		td(item.unlocked_character != null ? linkToCharacter(item.unlocked_character) : '-'),
		td(item.game_type != null ? item.game_type : '-')
	])
}

const viewItems = () => {
	selectSection('items')
	contents.append(sections.items)
}

const editItemName = (i) => {
	const item = cardData.data_pack.items[i]
	EditDialog.openStringEditor(
		`Edit Item ${i}: Name`,
		'Name:',
		item.name.string,
		(newValue) => {
			tauri_invoke('update_item', { index: i, name: newValue }).then(result => {
				if (result != null) item.name = result
				const itemEl = document.getElementById(`item-${i}`)
				if (itemEl != null) itemEl.replaceWith(renderItem(i, item))
			})
			EditDialog.close()
		},
		8
	)
}
