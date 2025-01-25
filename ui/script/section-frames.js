let frameCanvases = []

const setupFrameCanvases = () => {
	const frame_groups = cardData.data_pack.frame_groups
	frameCanvases = frame_groups.map(frame_group => {
		return frame_group.frames.map(frame => {
			if (frame === 'Implicit') {
				return null
			} else {
				const previewCanvas = document.createElement('canvas')
				previewCanvas.width = 128
				previewCanvas.height = 128
				return previewCanvas
			}
		})
	})
}

const drawFrameCanvases = () => {
	const frame_groups = cardData.data_pack.frame_groups
	frame_groups.forEach((frame_group, i) => {
		frame_group.frames.forEach((frame, j) => {
			if (frameCanvases[i] && frameCanvases[i][j] && frame.Explicit) {
				const previewContext = frameCanvases[i][j].getContext('2d')
				frame.Explicit.forEach(layer => {
					if (!specialLayers.includes(layer.image_id.entity_id)) {
						const imageIndex = layer.image_id.entity_id
						const imageSet = cardData.image_sets[imageIndex]
						if (imageSet && spriteImages[imageIndex]) {
							const subimageIndex = layer.subimage_index
							const subimage = imageSet.subimages[subimageIndex]
							if (subimage && spriteImages[imageIndex][subimageIndex]) {
								const x = 64 + subimage.offset_x + layer.x
								const y = 64 + subimage.offset_y + layer.y
								previewContext.drawImage(spriteImages[imageIndex][subimageIndex], x, y)
							}
						}
					}
				})
			}
		})
	})
}

const setupFrames = () => {
	setupFrameCanvases()
	drawFrameCanvases()

	const frame_groups = cardData.data_pack.frame_groups
	let el = document.createElement('div')

	frame_groups.forEach((frame_group, i) => {
		let rows = []

		frame_group.frames.forEach((frame, j) => {
			let isLast = j === frame_group.frames.length - 1
			let editButton = button({
				title: 'Edit Frame', className: 'icon',
				onclick: () => EditFrameDialog.open(i, j, frame)
			}, EDIT_ICON)

			if (frame === 'Implicit') {
				rows.push(tr({ id: `frame-${i}-${j}` }, [
					th(j),
					td(frameNames[j]),
					td({ colspan: 10 }, '<em>Implicit</em>')
				]))

			} else {
				frame.Explicit.forEach((frameLayer, k) => {
					let cells = []

					if (k === 0) {
						cells = cells.concat([
							th({ rowspan: frame.Explicit.length, className: isLast ? 'bottom-left-cell' : '' }, j),
							td({ rowspan: frame.Explicit.length }, frameNames[j])
						])
					}

					cells = cells.concat([
						td({ className: 'subrow' }, frameLayer.layer_type),
						td({ className: 'subrow' }, frameLayer.x),
						td({ className: 'subrow' }, frameLayer.y),
						td({ className: 'subrow' }, frameLayer.image_id != null ? linkToImage(frameLayer.image_id) : '-'),
						td({ className: 'subrow' }, frameLayer.subimage_index != null ? frameLayer.subimage_index : (frameLayer.image_id != null ? 0 : '-')),
						td({ className: 'subrow' }, frameLayer.unknown1),
						td({ className: 'subrow' }, frameLayer.unknown2),
						td({ className: 'subrow' }, frameLayer.unknown3)
					])

					if (k === 0) {
						cells = cells.concat([
							td({ rowspan: frame.Explicit.length }, [
								div({ className: 'preview' }, [ frameCanvases[i][j] ])
							]),
							td({ rowspan: frame.Explicit.length, className: isLast ? 'bottom-right-cell' : '' }, [
								editButton
							])
						])
					}

					rows.push(tr({ id: `frame-${i}-${j}` }, cells))
				})
			}
		})

		el.append(div({id: `framegroup-${i}`, className: 'table-title'}, `Frame Group ${i}`))
		el.append(table([
			thead([tr([
				th('-'),
				th('Name (guess)'),
				th('Type'),
				th('X'),
				th('Y'),
				th('Image ID'),
				th('Subimage Index'),
				th('?'),
				th('?'),
				th('?'),
				th('Preview'),
				th('Actions')
			])]),
			tbody(rows)
		]))
	})
	return el
}

const viewFrames = () => {
	selectSection('frames')
	contents.append(sections.frames)
}

const specialLayers = [
	1227,	// Head accessory
	1228,	// Head accessory (close-up)
	1264,	// Face accessory
	1265,	// Face accessory (close-up)
	1266,	// Body accessory
	1267,	// Body accessory (close-up)
	1276,	// Hand accessory
	1271,	// Hand accessory (close-up)
	1280,	// Dirt clouds
	1282	// Dirt clouds (close-up)
]

const frameNames = [
	'?',
	'Neutral 1',
	'Neutral 2',
	'Happy 1',
	'Happy 2',
	'Overjoyed 1',
	'Overjoyed 2',
	'Sad 1',
	'Sad 2',
	'Bored 1',
	'Bored 2',
	'Angry 1',
	'Angry 2',
	'?',
	'?',
	'Disappointed? 1',
	'Disappointed? 2',
	'Confused/Upset 1',
	'Confused/Upset 2',
	'?',
	'?',
	'?',
	'Walk 1, face left',
	'Walk 2, face left',
	'Walk 1, face right?',
	'Walk 2, face right?',
	'Sit + Gawk 1, face left',
	'Sit + Gawk 2, face left',
	'Sit + Gawk 1, face right?',
	'Sit + Gawk 2, face right?',
	'?',
	'?',
	'Far Neutral',
	'Far Sad',
	'?',
	'Far Walk, face left',
	'?',
	'?',
	'Close Neutral',
	'Close Happy',
	'Close Blush',
	'?',
	'Close Upset',
	'Kiss',
	'Skipping',
	'Far Skip',
	'Far Happy',
	'Far Angry',
	'Far Sit, face left',
	'Close Neutral, eyes closed',
	'Close Cry',
	'Close Unimpressed',
	'Close Distressed',
]
