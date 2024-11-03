const setupScenes = () => {
	let el = document.createElement('div')
	const scenes = cardData.data_pack.scenes
	scenes.forEach((scene, i) => {
		let previewLayers = []
		scene.layers.forEach(layer => {
			if (layer.image_id != null && (cardData.card_header == null || layer.image_id.card_id === cardData.card_header.card_id)) {
				let imageDef = cardData.sprite_pack.image_defs[layer.image_id.entity_id]
				if (imageDef != null) {
					let subimageDef = imageDef.subimage_defs[layer.subimage_index]
					if (subimageDef != null) {
						let x = subimageDef.offset_x + layer.x
						let y = subimageDef.offset_y + layer.y
						previewLayers.push(img({
							className: 'preview-layer',
							style: `left: ${x}px; top: ${y}px`,
							src: convertFileSrc(`${timestamp}-${layer.image_id.entity_id}-${layer.subimage_index}`, 'getimage')
						}))
					}
				}
			}
		})

		el.append(div({ id: `scene-${i}`, className: 'table-title' }, `Scene ${i}`))
		el.append(div({ className: 'preview' }, previewLayers))
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
				th('? 9'),
				th('? 10'),
				th('? 11'),
				th('? 12')
			])]),
			tbody(scene.layers.map((layer, i) => tr({}, [
				th(i),
				td(layer.x != null ? layer.x : '-'),
				td(layer.y != null ? layer.y : '-'),
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
				td(layer.flag4 ? '✔' :  '-')
			])))
		]))
	})
	return el
}

const viewScenes = () => {
	selectSection('scenes')
	contents.append(sections.scenes)
}
