const setupParticleEmitters = () => {
	const emitters = cardData.data_pack.particle_emitters
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
