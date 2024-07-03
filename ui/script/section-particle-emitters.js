const setupParticleEmitters = () => {
	const emitters = cardData.data_pack.particle_emitters
	return table([
		tbody(emitters.map((emitter, i) =>
			tr([th(i), td(emitter.data.map(b => formatHexCode(b)).join(' '))])
		))
	])
}

const viewParticleEmitters = () => {
	selectSection('view-particle-emitters-button')
	contents.append(sections.particleEmitters)
}
