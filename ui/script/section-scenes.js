let sceneCanvases = []

const setupSceneCanvases = () => {
	const scenes = cardData.data_pack.scenes
	sceneCanvases = scenes.map((scene, i) => {
		const previewCanvas = document.createElement('canvas')
		previewCanvas.width = 128
		previewCanvas.height = 128
		return previewCanvas
	})
}

const drawSceneCanvases = () => {
	const scenes = cardData.data_pack.scenes
	scenes.forEach((scene, i) => {
		const previewContext = sceneCanvases[i].getContext('2d')
		scene.layers.forEach(layer => {
			if (layer.image_id != null && (cardData.card_header == null || layer.image_id.card_id === cardData.card_header.card_id)) {
				const imageIndex = layer.image_id.entity_id
				const imageSet = cardData.image_sets[imageIndex]
				if (imageSet && spriteImages[imageIndex]) {
					const subimageIndex = layer.subimage_index
					const subimage = imageSet.subimages[subimageIndex]
					if (subimage && spriteImages[imageIndex][subimageIndex]) {
						const x = subimage.offset_x + layer.x
						const y = subimage.offset_y + layer.y
						previewContext.drawImage(spriteImages[imageIndex][subimageIndex], x, y)
					}
				}
			}
		})
	})
}

const setupScenes = () => {
	setupSceneCanvases()
	drawSceneCanvases()
	let el = document.createElement('div')
	const scenes = cardData.data_pack.scenes
	scenes.forEach((scene, i) => {
		el.append(div({ id: `scene-${i}`, className: 'table-title' }, `Scene ${i}`))
		el.append(div({ className: 'preview' }, [ sceneCanvases[i] ]))
		el.append(table([
			thead([tr([
				th('-'),
				th('X'),
				th('Y'),
				th('Image ID'),
				th('Subimage Index'),
				th('? 1'),
				th('? 2'),
				th('? 3'),
				th('? 4'),
				th('? 5'),
				th('? 6'),
				th('? 7'),
				th('? 8'),
				th('Flag 1'),
				th('Flag 2'),
				th('Flag 3'),
				th('Flag 4'),
				th('Actions')
			])]),
			tbody(scene.layers.map((layer, j) => tr({}, [
				th(j),
				td(layer.x),
				td(layer.y),
				td(layer.image_id != null ? linkToImage(layer.image_id) : '-'),
				td(layer.subimage_index),
				td(layer.unknown1),
				td(layer.unknown2),
				td(layer.unknown3),
				td(layer.unknown4),
				td(layer.unknown5),
				td(layer.unknown6),
				td(layer.unknown7),
				td(layer.unknown8),
				td(layer.flag1 ? '✔' :  '-'),
				td(layer.flag2 ? '✔' :  '-'),
				td(layer.flag3 ? '✔' :  '-'),
				td(layer.flag4 ? '✔' :  '-'),
				td([
					button({
						title: 'Edit Scene Layer', className: 'icon',
						onclick: () => EditSceneLayerDialog.open(i, j, layer)
					}, EDIT_ICON)
				])
			])))
		]))
	})
	return el
}

const viewScenes = () => {
	selectSection('scenes')
	contents.append(sections.scenes)
}
