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
			th('Animation ID?'),
			th('Price'),
			th('Fullness Increase?'),
			th('Happiness Increase?'),
			th('Unknown 3'),
			th('Unlocked Character'),
			th('Game Type'),
			th('Actions')
		])]),
		tbody(items.map((item, i) => renderItem(i, item)))
	])
}

const renderItem = (i, item) => {
	return tr({id: `item-${item.id.entity_id}`}, [
		th(item.id.entity_id),
		td(item.item_type),
		td(item.name.string),
		td(item.item_type === 'Game' ? `Scene: ${item.image_id.entity_id}` : displayImageWithLink(item.image_id, 0)),
		td(displayImageWithLink(item.worn_image_id, 0)),
		td(displayImageWithLink(item.close_image_id, 0)),
		td(item.animation_id != null ? item.animation_id.entity_id : '-'),
		td(item.price),
		td(item.unknown1),
		td(item.unknown2),
		td(item.unknown3),
		td(item.unlocked_character != null ? linkToCharacter(item.unlocked_character) : '-'),
		td(item.game_type != null ? item.game_type : '-'),
		td([
			button({
				title: 'Edit Item', className: 'icon',
				onclick: () => EditItemDialog.open(i, item)
			}, EDIT_ICON)
		])
	])
}

const viewItems = () => {
	selectSection('items')
	contents.append(sections.items)
}
