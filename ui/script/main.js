const Tauri = window.__TAURI__
const tauri_listen = Tauri.event.listen
const tauri_invoke = Tauri.core.invoke
const convertFileSrc = Tauri.core.convertFileSrc

let cardData = null
let contents = null

let timestamp = Date.now()

let sections = {}

window.addEventListener('load', () => {
	setupDialogs()

	document.getElementById('open-button').addEventListener('click', openBin)
	document.getElementById('save-button').addEventListener('click', saveBin)
	document.getElementById('save-as-button').addEventListener('click', saveBinAs)
	document.getElementById('export-data-button').addEventListener('click', exportData)
	document.getElementById('export-images-button').addEventListener('click', exportImages)

	tauri_listen('show_card', event => {
		cardData = event.payload

		timestamp = Date.now()

		const main = document.getElementById('main')
		main.replaceChildren()

		sections = {
			header: setupCardHeader(),
			particleEmitters: setupParticleEmitters(),
			scenes: setupScenes(),
			strings: setupStrings(),
			table9: setupTable9(),
			items: setupItems(),
			characters: setupCharacters(),
			animations: setupAnimations(),
			frames: setupFrames(),
			sprites: setupSprites(),
			palettes: setupPalettes()
		}

		const nav = div({id: 'sidebar'}, [
			button({id: 'view-header-button', onclick: viewCardHeader},
				'Header'),
			button({id: 'view-particle-emitters-button', onclick: viewParticleEmitters},
				`Particle Emitters <span class="tag">${cardData.data_pack.particle_emitters.length}</span>`),
			button({id: 'view-scenes-button', onclick: viewScenes},
				`Scenes <span class="tag">${cardData.data_pack.scenes.length}</span>`),
			button({id: 'view-strings-button', onclick: viewStrings},
				`Strings <span class="tag">${cardData.data_pack.strings.length}</span>`),
			button({id: 'view-table9-button', onclick: viewTable9},
				`Unknown <span class="tag">${cardData.data_pack.table9.length}</span>`),
			button({id: 'view-items-button', onclick: viewItems},
				`Items <span class="tag">${cardData.data_pack.items.length}</span>`),
			button({id: 'view-characters-button', onclick: viewCharacters},
				`Characters <span class="tag">${cardData.data_pack.characters.length}</span>`),
			button({id: 'view-animations-button', onclick: viewAnimations},
				`Animations <span class="tag">${cardData.data_pack.graphics_nodes.length}</span>`),
			button({id: 'view-frames-button', onclick: viewFrames},
				`Frames <span class="tag">${cardData.data_pack.frame_groups.length}</span>`),
			button({id: 'view-sprites-button', onclick: viewSprites},
				`Images <span class="tag">${cardData.sprite_pack.image_defs.length}</span>`),
			button({id: 'view-palettes-button', onclick: viewPalettes},
				`Palettes <span class="tag">${Math.ceil(cardData.sprite_pack.palettes.length / 4)}</span>`)
		])

		contents = div({id: 'contents'})

		main.append(nav)
		main.append(contents)

		viewCardHeader()
	})

	tauri_listen('show_firmware', event => {
		cardData = event.payload

		timestamp = Date.now()

		const main = document.getElementById('main')
		main.replaceChildren()

		sections = {
			header: setupFirmwareHeader(),
			particleEmitters: setupParticleEmitters(),
			scenes: setupScenes(),
			menu_strings: setupMenuStrings(),
			strings: setupStrings(),
			table9: setupTable9(),
			items: setupItems(),
			characters: setupCharacters(),
			animations: setupAnimations(),
			frames: setupFrames(),
			sprites: setupSprites(),
			palettes: setupPalettes()
		}

		const nav = div({id: 'sidebar'}, [
			button({id: 'view-header-button', onclick: viewFirmwareHeader},
				'Header'),
			button({id: 'view-particle-emitters-button', onclick: viewParticleEmitters},
				`Particle Emitters <span class="tag">${cardData.data_pack.particle_emitters.length}</span>`),
			button({id: 'view-scenes-button', onclick: viewScenes},
				`Scenes <span class="tag">${cardData.data_pack.scenes.length}</span>`),
			button({id: 'view-menu-strings-button', onclick: viewMenuStrings},
				`Menu Strings <span class="tag">${cardData.menu_strings.length}</span>`),
			button({id: 'view-strings-button', onclick: viewStrings},
				`Dialog Strings <span class="tag">${cardData.data_pack.strings.length}</span>`),
			button({id: 'view-table9-button', onclick: viewTable9},
				`Unknown <span class="tag">${cardData.data_pack.table9.length}</span>`),
			button({id: 'view-items-button', onclick: viewItems},
				`Items <span class="tag">${cardData.data_pack.items.length}</span>`),
			button({id: 'view-characters-button', onclick: viewCharacters},
				`Characters <span class="tag">${cardData.data_pack.characters.length}</span>`),
			button({id: 'view-animations-button', onclick: viewAnimations},
				`Animations <span class="tag">${cardData.data_pack.graphics_nodes.length}</span>`),
			button({id: 'view-frames-button', onclick: viewFrames},
				`Frames <span class="tag">${cardData.data_pack.frame_groups.length}</span>`),
			button({id: 'view-sprites-button', onclick: viewSprites},
				`Images <span class="tag">${cardData.sprite_pack.image_defs.length}</span>`),
			button({id: 'view-palettes-button', onclick: viewPalettes},
				`Palettes <span class="tag">${Math.ceil(cardData.sprite_pack.palettes.length / 4)}</span>`)
		])

		contents = div({id: 'contents'})

		main.append(nav)
		main.append(contents)

		viewFirmwareHeader()
	})

	tauri_listen('update_strings', event => {
		cardData.data_pack.strings = event.payload
		sections.strings = setupStrings()
		viewStrings()
	})

	tauri_listen('update_menu_strings', event => {
		cardData.menu_strings = event.payload
		sections.menu_strings = setupMenuStrings()
		viewMenuStrings()
	})

	tauri_listen('update_image_def', event => {
		updateImageDef(event.payload[0], event.payload[1])
	})

	tauri_listen('update_image', event => {
		timestamp = Date.now()
		const imageIndex = event.payload
		const subimageCount = cardData.sprite_pack.image_defs[imageIndex].subimage_count
		for (let i=0; i < subimageCount; i++) {
			const subimageEl = document.getElementById(`subimage-${imageIndex}-${i}`)
			subimageEl.src = convertFileSrc(`${timestamp}-${imageIndex}-${i}`, 'getimage')
		}
	})

	tauri_listen('show_spinner', () => {
		document.getElementById('spinner').classList.add('on')
	})

	tauri_listen('hide_spinner', () => {
		document.getElementById('spinner').classList.remove('on')
	})

	document.body.addEventListener('keydown', (event) => {
		const KEY = event.key.toUpperCase()
		const ONLY = !event.ctrlKey && !event.shiftKey && !event.altKey
		const CTRL = event.ctrlKey && !event.shiftKey && !event.altKey
		const CTRL_SHIFT = event.ctrlKey && event.shiftKey && !event.altKey

		if (CTRL && KEY === 'Q') {
			event.preventDefault()
			tauri_invoke('try_quit')
		} else if (AboutDialog.isOpen()) {
			if (ONLY && KEY === 'ESCAPE') {
				event.preventDefault()
				closeDialogs()
			}
		} else if (CTRL && KEY === 'O') {
			event.preventDefault()
			openBin()
		}
	})
})

const setupDialogs = () => {
	EditDialog.setup()
	AboutDialog.setup()
}

const closeDialogs = () => {
	EditDialog.close()
	AboutDialog.close()
}

const openBin = () => {
	tauri_invoke('open_bin')
}

const saveBin = () => {
	tauri_invoke('save_bin')
}

const saveBinAs = () => {
	tauri_invoke('save_bin_as')
}

const exportData = () => {
	tauri_invoke('export_data')
}

const exportImages = () => {
	tauri_invoke('export_images')
}

const importImageSpritesheet = (imageIndex) => {
	tauri_invoke('import_image_spritesheet', { imageIndex })
}

const exportImageSpritesheet = (imageIndex) => {
	tauri_invoke('export_image_spritesheet', { imageIndex })
}

const selectSection = (sectionId) => {
	for (child of document.getElementById('sidebar').children) {
		child.classList.remove('selected')
	}
	document.getElementById(sectionId).classList.add('selected')
	contents.replaceChildren()
	contents.scrollTo(0, 0)
}

const setupTable9 = () => {
	const entities = cardData.data_pack.table9
	return table([
		tbody(entities.map((entity, i) => tr({}, [
			th(i),
			td(entity.map(b => formatHexCode(b)).join(' '))
		])))
	])
}
const viewTable9 = () => {
	selectSection('view-table9-button')
	contents.append(sections.table9)
}
