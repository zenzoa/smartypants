const setupPalettes = () => {
	const colors = cardData.sprite_pack.palettes
	const paletteCount = Math.ceil(colors.length / 4)
	let palettes = []
	for (let i=0; i < paletteCount; i++) {
		let palette = []
		for (let j=0; j < 4; j++) {
			const colorIndex = i*4 + j
			if (colors[colorIndex] != null) {
				palette.push(colors[colorIndex])
			}
		}
		palettes.push(palette)
	}

	return div({id: 'palettes'}, palettes.map((palette, i) =>
		div({id: `palette-${i}`, className: 'palette'}, [div({className: 'palette-index'}, i)].concat(palette.map((color, j) => {
			const hexR = (color.r).toString(16).padStart(2, '0')
			const hexG = (color.g).toString(16).padStart(2, '0')
			const hexB = (color.b).toString(16).padStart(2, '0')
			const hexCode = `#${hexR}${hexG}${hexB}`
			if (color.a > 0) {
				return div({className: 'palette-color', style: `background-color: ${hexCode}`, title: hexCode})
			} else {
				return div({className: 'palette-color transparent-color', title: 'transparent'})
			}
		})))
	))
}

const viewPalettes = () => {
	selectSection('palettes')
	contents.append(sections.palettes)
}
