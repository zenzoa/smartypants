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
				th('Actions')
			])]),
			tbody(cardData.image_sets.map((imageSet, i) =>
				renderImageSet(i, imageSet)
			))
		])
	])
}

const updateImageSet = (i, newImageSet) => {
	cardData.image_sets[i] = newImageSet
	const imageEl = document.getElementById(`image-${i}`)
	imageEl.replaceWith(renderImageSet(i, newImageSet))
}

const renderImageSet = (i, imageSet) => {
	return tr({id: `image-${i}`}, [
		th(i),
		td(imageSet.subimages.map((_, j) => displayImage(i, j, true))),
		td(`${imageSet.width}×${imageSet.height}`),
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
					onclick: () => EditSpriteDialog.open(i, imageSet)
				}, EDIT_ICON)
			])
		])
	])
}

const viewSprites = () => {
	selectSection('sprites')
	contents.append(sections.sprites)
}
