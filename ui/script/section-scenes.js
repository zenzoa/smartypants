const setupScenes = () => {
	let el = document.createElement('div')
	const scenes = cardData.data_pack.scenes
	scenes.forEach((scene, i) => {
		el.append(div({id: `scene-${i}`, className: 'table-title'}, `Scene ${i}`))
		el.append(table([
			thead([tr([
				th('-'),
				th('X'),
				th('Y'),
				th('Image ID'),
				th('Subimage Index'),
				th('Preview'),
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
				td(layer.subimage_index ||  0),
				td(layer.image_id != null ? displayImageWithLink(layer.image_id, layer.subimage_index || 0) : '-'),
				td(layer.unknown1 != null ? layer.unknown1 :  '-'),
				td(layer.unknown2 != null ? layer.unknown2 :  '-'),
				td(layer.unknown3 != null ? layer.unknown3 :  '-'),
				td(layer.unknown4 != null ? layer.unknown4 :  '-'),
				td(layer.unknown5 != null ? layer.unknown5 :  '-'),
				td(layer.unknown6 != null ? layer.unknown6 :  '-'),
				td(layer.unknown7 != null ? layer.unknown7 :  '-'),
				td(layer.unknown8 != null ? layer.unknown8 :  '-'),
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
