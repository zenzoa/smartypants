const setupFrames = () => {
	const frame_groups = cardData.data_pack.frame_groups
	let el = document.createElement('div')
	frame_groups.forEach((frame_group, i) => {
		el.append(div({id: `framegroup-${i}`, 'class': 'table-title'}, `Frame Group ${i}`))
		el.append(table([
			thead([tr([
				th('-'),
				th('Name (guess)'),
				th('Type'),
				th('X'),
				th('Y'),
				th('Image ID'),
				th('Subimage Index'),
				th('Preview')
			])]),
			tbody(frame_group.frames.map((frame, i) => {
				if (frame === 'Implicit') {
					return tr({}, [
						th(i),
						td(frameNames[i]),
						td({'colspan': 6}, '<em>Implicit</em>')
					])
				} else {
					const frame_info = frame.Explicit[0]
					return tr({}, [
						th(i),
						td(frameNames[i]),
						td(frame_info.layer_type),
						td(frame_info.x),
						td(frame_info.y),
						td(frame_info.image_id != null ? linkToImage(frame_info.image_id) : '-'),
						td(frame_info.subimage_index != null ? frame_info.subimage_index : (frame_info.image_id != null ? 0 : '-')),
						td(displayImageWithLink(frame_info.image_id, frame_info.subimage_index))
					])
				}
			}))
		]))
	})
	return el
}

const viewFrames = () => {
	selectSection('view-frames-button')
	contents.append(sections.frames)
}

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
