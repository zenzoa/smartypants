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
		tbody(items.map(item => tr({id: `item-${item.id.entity_id}`}, [
			th(item.id.entity_id),
			td(item.item_type),
			td(item.name),
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
		])))
	])
}

const viewItems = () => {
	selectSection('view-items-button')
	contents.append(sections.items)
}
