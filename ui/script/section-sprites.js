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
				th('First Palette'),
				th('Max Colors'),
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
		td([
			linkToPalette(imageDef.first_palette_index),
			button({className: 'edit', onclick: editImageFirstPalette.bind(this, i)}, '✏️')
		]),
		td(bppToMaxColors[cardData.sprite_pack.sprites[imageDef.first_sprite_index].bits_per_pixel]),
		td([
			button({className: 'text', onclick: importImageSpritesheet.bind(this, i)}, 'Import'),
			button({className: 'text', onclick: exportImageSpritesheet.bind(this, i)}, 'Export')
		])
	])
}

const viewSprites = () => {
	selectSection('view-sprites-button')
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
