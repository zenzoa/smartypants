const setupSprites = () => {
	const imageDefs = cardData.sprite_pack.image_defs
	return table([
		tbody(imageDefs.map((imageDef, i) => tr({id: `image-${i}`}, [
			th(i),
			td(Array(imageDef.subimage_count).fill(0).map((_, j) => {
				return displayImage(i, j, true)
			}))
		])))
	])
}

const viewSprites = () => {
	selectSection('view-sprites-button')
	contents.append(sections.sprites)
}
