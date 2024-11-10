const Tauri = window.__TAURI__
const tauri_listen = Tauri.event.listen
const tauri_invoke = Tauri.core.invoke
const convertFileSrc = Tauri.core.convertFileSrc

let cardData = null
let textEncoding = null
let contents = null

let timestamp = Date.now()

let currentSection = ''
let sections = {}

let defaultStyle = null

const U8_MAX = 255
const U16_MAX = 65535

window.addEventListener('load', () => {
	defaultStyle = document.documentElement.style

	tauri_listen('update_theme', updateTheme)
	tauri_listen('update_toolbar_visibility', updateToolbarVisibility)
	tauri_invoke('load_config')

	// disable context menu
	document.body.addEventListener('contextmenu', event => {
		event.preventDefault()
		return false
	}, false)

	setupDialogs()

	document.getElementById('open-button')
		.addEventListener('click', () => tauri_invoke('open_bin'))

	document.getElementById('save-button')
		.addEventListener('click', () => tauri_invoke('save_bin'))

	document.getElementById('save-as-button')
		.addEventListener('click', () => tauri_invoke('save_bin_as'))

	document.getElementById('import-strings-button')
		.addEventListener('click', () => tauri_invoke('import_strings'))

	document.getElementById('import-images-button')
		.addEventListener('click', () => tauri_invoke('import_images'))

	document.getElementById('export-strings-button')
		.addEventListener('click', () => tauri_invoke('export_strings'))

	document.getElementById('export-images-button')
		.addEventListener('click', () => tauri_invoke('export_images'))

	tauri_listen('update_data', event => {
		cardData = event.payload

		timestamp = Date.now()

		const main = document.getElementById('main')
		main.replaceChildren()

		sections = {
			header: setupHeader(),
			table1: setupTable1(),
			particleEmitters: setupParticleEmitters(),
			scenes: setupScenes(),
			menuStrings: cardData.bin_type === "Firmware" ? setupMenuStrings() : null,
			tamaStrings: setupTamaStrings(),
			table9: setupTable9(),
			items: setupItems(),
			characters: setupCharacters(),
			animations: setupAnimations(),
			frames: setupFrames(),
			sprites: setupSprites()
		}

		const nav = div({id: 'sidebar'}, [
			button({id: 'view-header-button', onclick: viewHeader},
				'Header'),
			button({id: 'view-characters-button', onclick: viewCharacters},
				`Characters <span class="tag">${cardData.data_pack.characters.length}</span>`),
			button({id: 'view-items-button', onclick: viewItems},
				`Items <span class="tag">${cardData.data_pack.items.length}</span>`),
			cardData.bin_type === "Firmware" ? button({id: 'view-menuStrings-button', onclick: viewMenuStrings},
				`Menu Strings <span class="tag">${cardData.menu_strings.length}</span>`) : '',
			button({id: 'view-tamaStrings-button', onclick: viewTamaStrings},
				`Dialog Strings <span class="tag">${cardData.data_pack.tamastrings.length}</span>`),
			button({id: 'view-sprites-button', onclick: viewSprites},
				`Images <span class="tag">${cardData.image_sets.length}</span>`),
			button({id: 'view-frames-button', onclick: viewFrames},
				`Frames <span class="tag">${cardData.data_pack.frame_groups.length}</span>`),
			button({id: 'view-scenes-button', onclick: viewScenes},
				`Scenes <span class="tag">${cardData.data_pack.scenes.length}</span>`),
			button({id: 'view-animations-button', onclick: viewAnimations},
				`Animations <span class="tag">${cardData.data_pack.graphics_nodes.length}</span>`),
			button({id: 'view-particleEmitters-button', onclick: viewParticleEmitters},
				`Particle Emitters <span class="tag">${cardData.data_pack.particle_emitters.length}</span>`),
			button({id: 'view-table1-button', onclick: viewTable1},
				`Unknown 1 <span class="tag">${cardData.data_pack.table1.length}</span>`),
			button({id: 'view-table9-button', onclick: viewTable9},
				`Unknown 2 <span class="tag">${cardData.data_pack.table9.length}</span>`)
		])

		contents = div({id: 'contents'})

		main.append(nav)
		main.append(contents)

		viewHeader()
	})

	tauri_listen('refresh_tab', () => {
		selectSection(currentSection)
		contents.append(sections[currentSection])
	})

	tauri_listen('update_card_header', event => {
		cardData.card_header = event.payload
		sections.header = setupHeader()
		viewHeader()
	})

	tauri_listen('update_menu_strings', event => {
		cardData.menu_strings = event.payload[0]
		sections.menuStrings = setupMenuStrings()
		if (event.payload[1] || currentSection === 'menuStrings') {
			viewMenuStrings()
		}
	})

	tauri_listen('update_tamastrings', event => {
		cardData.data_pack.tamastrings = event.payload[0]
		sections.tamaStrings = setupTamaStrings()
		if (event.payload[1] || currentSection === 'tamaStrings') {
			viewTamaStrings()
		}
	})

	tauri_listen('update_items', event => {
		cardData.data_pack.items = event.payload[0]
		sections.items = setupItems()
		if (event.payload[1] || currentSection === 'items') {
			viewItems()
		}
	})

	tauri_listen('update_characters', event => {
		cardData.data_pack.characters = event.payload[0]
		sections.characters = setupCharacters()
		if (event.payload[1] || currentSection === 'characters') {
			viewCharacters()
		}
	})

	tauri_listen('update_image_set', event => {
		updateImageSet(event.payload[0], event.payload[1])
	})

	tauri_listen('update_image', event => {
		timestamp = Date.now()
		const imageIndex = event.payload
		const subimageCount = cardData.image_sets[imageIndex].subimages.length
		for (let i=0; i < subimageCount; i++) {
			const subimageEl = document.getElementById(`subimage-${imageIndex}-${i}`)
			subimageEl.src = convertFileSrc(`${timestamp}-${imageIndex}-${i}`, 'getimage')
		}
		sections.particleEmitters = setupParticleEmitters()
		sections.scenes = setupScenes()
		sections.items = setupItems()
		sections.characters = setupCharacters()
		sections.animations = setupAnimations()
		sections.frames = setupFrames()
	})

	tauri_listen('update_images', event => {
		timestamp = Date.now()
		sections.sprites = setupSprites()
		sections.particleEmitters = setupParticleEmitters()
		sections.scenes = setupScenes()
		sections.items = setupItems()
		sections.characters = setupCharacters()
		sections.animations = setupAnimations()
		sections.frames = setupFrames()
		viewSprites()
	})

	tauri_listen('update_encoding_language', event => {
		if (cardData != null) {
			cardData.encoding_language = event.payload
			sections.header = setupHeader()
			if (currentSection === 'header') {
				viewHeader()
			}
		}
	})

	tauri_listen('show_spinner', () => {
		document.getElementById('spinner').classList.add('on')
	})

	tauri_listen('hide_spinner', () => {
		document.getElementById('spinner').classList.remove('on')
	})

	tauri_listen('update_char_codes', event => {
		textEncoding = event.payload.slice(1, 257)
	})

	tauri_listen('open_encoding_dialog', event => {
		EditEncodingDialog.open()
	})

	tauri_invoke('get_default_char_codes').then(result => {
		textEncoding = result.slice(1, 257)
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

const updateTheme = (event) => {
	if (defaultStyle != null) {
		document.documentElement.style = defaultStyle
	}
	const style = document.documentElement.style
	for (let i = 0; i < event.payload.length; i++) {
		const prop = event.payload[i]
		style.setProperty(`--${prop.key}`, prop.value)
	}
}

const updateToolbarVisibility = (event) => {
	const style = document.documentElement.style
	if (event.payload) {
		style.setProperty('--toolbar-display', 'flex')
	} else {
		style.setProperty('--toolbar-display', 'none')
	}
}

const setupDialogs = () => {
	EditDialog.setup()
	AboutDialog.setup()
	ChooseEncodingDialog.setup()
	EditEncodingDialog.setup()
}

const closeDialogs = () => {
	EditDialog.close()
	AboutDialog.close()
	ChooseEncodingDialog.close()
	EditEncodingDialog.close()
}

const importImageSpritesheet = (imageIndex) => {
	tauri_invoke('import_image_spritesheet', { imageIndex })
}

const exportImageSpritesheet = (imageIndex) => {
	tauri_invoke('export_image_spritesheet', { imageIndex })
}

const selectSection = (sectionName) => {
	currentSection = sectionName
	for (child of document.getElementById('sidebar').children) {
		child.classList.remove('selected')
	}
	document.getElementById(`view-${sectionName}-button`).classList.add('selected')
	contents.replaceChildren()
	contents.scrollTo(0, 0)
}

const setupTable1 = () => {
	const entities = cardData.data_pack.table1
	if (entities.length === 0) {
		return div('[empty]')
	} else {
		return table([
			tbody(entities.map((entity, i) => tr({}, [
				th(i),
				td(entity.map(b => formatHexCode(b)).join(' '))
			])))
		])
	}
}
const viewTable1 = () => {
	selectSection('table1')
	contents.append(sections.table1)
}

const setupTable9 = () => {
	const entities = cardData.data_pack.table9
	if (entities.length === 0) {
		return div('[empty]')
	} else {
		return table([
			tbody(entities.map((entity, i) => tr({}, [
				th(i),
				td(entity.map(b => formatHexCode(b)).join(' '))
			])))
		])
	}
}
const viewTable9 = () => {
	selectSection('table9')
	contents.append(sections.table9)
}

const EDIT_ICON = '<svg viewBox="0 0 64 64"><path fill-rule="evenodd" clip-rule="evenodd" d="M40.4767 6.29289C42.4293 4.34027 45.5951 4.34028 47.5477 6.2929L56.7401 15.4853C58.6927 17.4379 58.6927 20.6037 56.7401 22.5564L30.8082 48.4883C30.2298 49.0667 29.5183 49.4943 28.7362 49.7338L15.4884 53.7892C11.6565 54.9622 8.07077 51.3765 9.24379 47.5446L13.2993 34.2968C13.5387 33.5147 13.9663 32.8032 14.5447 32.2248L40.4767 6.29289ZM44.0122 11.2426L22.6777 32.5772L30.4558 40.3553L51.7904 19.0208L44.0122 11.2426ZM16.7271 43.5971L18.7158 37.1006L25.9324 44.3172L19.4359 46.3059L16.7271 43.5971Z" fill="currentColor"/></svg>'
const IMPORT_ICON = '<svg viewBox="0 0 64 64"><path d="M47 23C47 21.8954 46.1046 21 45 21H39C37.8954 21 37 20.1046 37 19V17C37 15.8954 37.8954 15 39 15H48C50.7614 15 53 17.2386 53 20V48C53 50.7614 50.7614 53 48 53H20C17.2386 53 15 50.7614 15 48V39C15 37.8954 15.8954 37 17 37H19C20.1046 37 21 37.8954 21 39V45C21 46.1046 21.8954 47 23 47H45C46.1046 47 47 46.1046 47 45V23Z" fill="currentColor"/><path d="M13.1213 8.87868C11.9497 7.70711 10.0503 7.70711 8.87868 8.87868C7.70711 10.0503 7.70711 11.9497 8.87868 13.1213L22.2574 26.5H14.5C12.8431 26.5 11.5 27.8431 11.5 29.5C11.5 31.1569 12.8431 32.5 14.5 32.5H27.5C30.2614 32.5 32.5 30.2614 32.5 27.5V14.5C32.5 12.8431 31.1569 11.5 29.5 11.5C27.8431 11.5 26.5 12.8431 26.5 14.5V22.2574L13.1213 8.87868Z" fill="currentColor"/></svg>'
const EXPORT_ICON = '<svg viewBox="0 0 64 64"><path fill-rule="evenodd" clip-rule="evenodd" d="M17 23C17 21.8954 17.8954 21 19 21H28C29.1046 21 30 20.1046 30 19V17C30 15.8954 29.1046 15 28 15H16C13.2386 15 11 17.2386 11 20V48C11 50.7614 13.2386 53 16 53H44C46.7614 53 49 50.7614 49 48V36C49 34.8954 48.1046 34 47 34H45C43.8954 34 43 34.8954 43 36V45C43 46.1046 42.1046 47 41 47H19C17.8954 47 17 46.1046 17 45V23Z" fill="currentColor"/><path fill-rule="evenodd" clip-rule="evenodd" d="M37 9C35.3431 9 34 10.3431 34 12C34 13.6569 35.3431 15 37 15H44.7574L26.3787 33.3787C25.2071 34.5503 25.2071 36.4497 26.3787 37.6213C27.5503 38.7929 29.4497 38.7929 30.6213 37.6213L49 19.2426V27C49 28.6569 50.3431 30 52 30C53.6569 30 55 28.6569 55 27V14C55 11.2386 52.7614 9 50 9H37Z" fill="currentColor"/></svg>'
