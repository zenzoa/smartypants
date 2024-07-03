const setupAnimations = () => {
	const animations = cardData.data_pack.graphics_nodes
	return table([
		tbody(animations.map((animation, i) => tr({}, [
			th(i),
			td(animation.data.map(b => formatHexCode(b)).join(' '))
		])))
	])
}

const viewAnimations = () => {
	selectSection('view-animations-button')
	contents.append(sections.animations)
}
