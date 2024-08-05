const setupParticleEmitters = () => {
	const emitters = cardData.data_pack.particle_emitters
	if (emitters.length === 0) {
		return div('[empty]')
	}
	return table([
		tbody(emitters.map((emitter, i) =>
			tr([th(i), td(emitter.data.map(b => formatHexCode(b)).join(' '))])
		))
	])
}

const viewParticleEmitters = () => {
	selectSection('particleEmitters')
	contents.append(sections.particleEmitters)
}
