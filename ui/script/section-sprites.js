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
				th('X-Offset'),
				th('Y-Offset'),
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
		td(Array(imageDef.subimage_count).fill(0).map((_, j) =>
			displayImage(i, j, true)
		)),
		td(imageDef.offset_x),
		td(imageDef.offset_y),
		cardData.lock_colors ? td([linkToPalette(imageDef.first_palette_index)]) : '',
		cardData.lock_colors ? td(bppToMaxColors[cardData.sprite_pack.sprites[imageDef.first_sprite_index].bits_per_pixel]) : '',
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

const editImageFirstPalette = (i) => {
	const image_def = cardData.sprite_pack.image_defs[i]
	EditDialog.openNumberEditor(
		`Edit Image ${i}: First Palette`,
		'First Palette',
		image_def.first_palette_index,
		(newValue) => {
			image_def.first_palette_index = newValue
			tauri_invoke('update_image_def', { index: i, firstPaletteIndex: newValue })
			EditDialog.close()
		},
		0,
		Math.floor(cardData.sprite_pack.palettes.length / 4) - 1
	)
}
