const setupScenes = () => {
	let el = document.createElement('div')
	const scenes = cardData.data_pack.scenes
	scenes.forEach((scene, i) => {
		el.append(div({id: `scene-${i}`, className: 'table-title'}, `Scene ${i}`))
		el.append(table([
			thead([tr([
				th('-'),
				th('Props'),
				th('X'),
				th('Y'),
				th('Image ID'),
				th('Subimage Index'),
				th('Preview')
			])]),
			tbody(scene.layers.map((layer, i) => tr({}, [
				th(i),
				td('#' + formatHexCode(layer.props)),
				td(layer.x),
				td(layer.y),
				td(linkToImage(layer.image_id)),
				td(layer.subimage_index),
				td(displayImageWithLink(layer.image_id, layer.subimage_index))
			])))
		]))
	})
	return el
}

const viewScenes = () => {
	selectSection('scenes')
	contents.append(sections.scenes)
}
