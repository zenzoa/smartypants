let spriteImages = []

const bppToMaxColors = {
	2: 4,
	4: 16,
	6: 64,
	8: 256
}

const setupSprites = () => {
	loadSpriteImages()
	return div([
		table([
			thead([tr([
				th('ID'),
				th('Sub-Images'),
				th('Dimensions'),
				th('# Palettes'),
				th('Actions')
			])]),
			tbody(cardData.image_sets.map((imageSet, i) =>
				renderImageSet(i, imageSet)
			))
		])
	])
}

const loadSpriteImages = () => {
	let spritesToLoad = cardData.image_sets.reduce((prev, curr) => prev + curr.subimages.length * curr.palette_count, 0)
	spriteImages = cardData.image_sets.map((imageSet, i) => {
		let subimages = []
		for (let p = 0; p < imageSet.palette_count; p++) {
			let subimageOffset = imageSet.subimages.length * p;
			for (let j = 0; j < imageSet.subimages.length; j++) {
				let imageEl = createImage(i, j, subimageOffset)
				imageEl.onload = () => {
					spritesToLoad -= 1
					if (spritesToLoad === 0) {
						drawSceneCanvases()
						drawFrameCanvases()
					}
				}
				subimages.push(imageEl)
			}
		}
		return subimages
	})
}

const loadSpriteImage = (i, imageSet) => {
	let spritesToLoad = imageSet.subimages.length * imageSet.palette_count
	spriteImages[i] = []
	for (let p = 0; p < imageSet.palette_count; p++) {
		let subimageOffset = imageSet.subimages.length * p;
		for (let j = 0; j < imageSet.subimages.length; j++) {
			const imageEl = createImage(i, j, subimageOffset)
			imageEl.onload = () => {
				spritesToLoad -= 1
				if (spritesToLoad === 0) {
					drawSceneCanvases()
					drawFrameCanvases()
				}
			}
			const imageCopies = document.getElementsByClassName(`preview-image subimage-${i}-${j + subimageOffset}`)
			for (let c = 0; c < imageCopies.length; c++) {
				imageCopies[c].replaceWith(imageEl.cloneNode())
			}
			spriteImages[i].push(imageEl)
		}
	}
}

const updateImageSet = (i, newImageSet) => {
	cardData.image_sets[i] = newImageSet
	loadSpriteImage(i, newImageSet)
	const imageSetEl = document.getElementById(`image-${i}`)
	imageSetEl.replaceWith(renderImageSet(i, newImageSet))
}

const renderImageSet = (i, imageSet) => {
	let subimageCellContents = []
	if (imageSet.palette_count === 1) {
		subimageCellContents = imageSet.subimages.map((_, j) => displayImage(i, j))
	} else {
		let subimagesByPalette = [];
		for (let p = 0; p < imageSet.palette_count; p++) {
			let subimageOffset = imageSet.subimages.length * p;
			subimagesByPalette.push(div(
				{ className: 'subimage-block' },
				imageSet.subimages.map((_, j) => displayImage(i, j, subimageOffset))
			))
		}
		subimageCellContents = [ div({ className: 'subimage-list' }, subimagesByPalette) ]
	}

	return tr({id: `image-${i}`}, [
		th(i),
		td({ id: `imagecontents-${i}` }, subimageCellContents),
		td(`${imageSet.width}×${imageSet.height}`),
		td(`${imageSet.palette_count}`),
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
