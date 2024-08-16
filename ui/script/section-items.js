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
		td([
			span('#' + formatHexCode(item.unknown1)),
			button({className: 'edit', onclick: editItemUnknown1.bind(this, i)}, '✏️')
		]),
		td([
			span(item.price),
			button({className: 'edit', onclick: editItemPrice.bind(this, i)}, '✏️')
		]),
		td([
			span('#' + formatHexCode(item.unknown2)),
			button({className: 'edit', onclick: editItemUnknown2.bind(this, i)}, '✏️')
		]),
		td([
			span('#' + formatHexCode(item.unknown3)),
			button({className: 'edit', onclick: editItemUnknown3.bind(this, i)}, '✏️')
		]),
		td([
			span('#' + formatHexCode(item.unknown4)),
			button({className: 'edit', onclick: editItemUnknown4.bind(this, i)}, '✏️')
		]),
		td(item.unlocked_character != null ? linkToCharacter(item.unlocked_character) : '-'),
		td(item.game_type != null ? item.game_type : '-')
	])
}

const viewItems = () => {
	selectSection('items')
	contents.append(sections.items)
}

const updateItem = (i, prop, value) => {
	console.log(`update item ${i}: ${prop} = ${value}`)
	tauri_invoke('update_item', { index: i, propertyName: prop, newValue: `${value}` }).then(result => {
		if (result != null) {
			cardData.data_pack.items[i] = result
			const itemEl = document.getElementById(`item-${i}`)
			if (itemEl != null) itemEl.replaceWith(renderItem(i, result))
		}
	})
	EditDialog.close()
}

const editItemName = (i) => {
	const item = cardData.data_pack.items[i]
	EditDialog.openStringEditor(
		`Edit Item ${i}: Name`,
		'Name:',
		item.name.string,
		updateItem.bind(this, i, 'name'),
		8
	)
}

const editItemUnknown1 = (i) => {
	const item = cardData.data_pack.items[i]
	EditDialog.openHexEditor(
		`Edit Item ${i}: Unknown 1`,
		'Unknown 1:',
		item.unknown1.toString(16),
		updateItem.bind(this, i, 'unknown1'),
	)
}

const editItemPrice = (i) => {
	const item = cardData.data_pack.items[i]
	EditDialog.openNumberEditor(
		`Edit Item ${i}: Price`,
		'Price:',
		item.price,
		updateItem.bind(this, i, 'price'),
		0,
		9999
	)
}

const editItemUnknown2 = (i) => {
	const item = cardData.data_pack.items[i]
	EditDialog.openHexEditor(
		`Edit Item ${i}: Unknown 2`,
		'Unknown 2:',
		item.unknown2.toString(16),
		updateItem.bind(this, i, 'unknown2'),
	)
}

const editItemUnknown3 = (i) => {
	const item = cardData.data_pack.items[i]
	EditDialog.openHexEditor(
		`Edit Item ${i}: Unknown 3`,
		'Unknown 3:',
		item.unknown3.toString(16),
		updateItem.bind(this, i, 'unknown3'),
	)
}

const editItemUnknown4 = (i) => {
	const item = cardData.data_pack.items[i]
	EditDialog.openHexEditor(
		`Edit Item ${i}: Unknown 4`,
		'Unknown 4:',
		item.unknown4.toString(16),
		updateItem.bind(this, i, 'unknown4'),
	)
}
