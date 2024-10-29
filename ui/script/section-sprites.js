const bppToMaxColors = {
	2: 4,
	4: 16,
	6: 64,
	8: 256
}

const setupSprites = () => {
	return div([
		table([
			thead([tr([
				th('ID'),
				th('Sub-Images'),
				th('Dimensions'),
				cardData.lock_colors ? th('First Palette ID') : '',
				cardData.lock_colors ? th('Max Colors') : '',
				th('Actions')
			])]),
			tbody(cardData.sprite_pack.image_defs.map((imageDef, i) =>
				renderImageDef(i, imageDef)
			))
		])
	])
}

const updateImageDef = (i, newImageDef) => {
	cardData.sprite_pack.image_defs[i] = newImageDef
	const imageEl = document.getElementById(`image-${i}`)
	imageEl.replaceWith(renderImageDef(i, newImageDef))
}

const renderImageDef = (i, imageDef) => {
	return tr({id: `image-${i}`}, [
		th(i),
		td(imageDef.subimage_defs.map((_, j) => displayImage(i, j, true))),
		td(`${imageDef.width}×${imageDef.height}`),
		cardData.lock_colors ? td([linkToPalette(imageDef.first_palette_index)]) : '',
		cardData.lock_colors ? td(bppToMaxColors[imageDef.subimage_defs[0].sprites[0].bits_per_pixel]) : '',
		td([
			div({className:'button-row'}, [
				button({
					className: 'icon', title: 'Import Spritesheet',
					onclick: importImageSpritesheet.bind(this, i)
				}, IMPORT_ICON),
				button({
					className: 'icon', title: 'Export Spritesheet',
					onclick: exportImageSpritesheet.bind(this, i)
				}, EXPORT_ICON),
				button({
					className: 'icon', title: 'Edit Image Definition',
					onclick: () => EditSpriteDialog.open(i, imageDef)
				}, EDIT_ICON)
			])
		])
	])
}

const viewSprites = () => {
	selectSection('sprites')
	contents.append(sections.sprites)
}
