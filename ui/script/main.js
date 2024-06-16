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

	tauri_listen('show_card', event => {
		cardData = event.payload

		timestamp = Date.now()

		const main = document.getElementById('main')
		main.innerHTML = ''

		sections = {
			header: setupHeader(),
			particleEmitters: setupParticleEmitters(),
			scenes: setupScenes(),
			strings: setupStrings(),
			table9: setupTable9(),
			items: setupItems(),
			characters: setupCharacters(),
			animations: setupAnimations(),
			frames: setupFrames(),
			sprites: setupSprites()
		}

		const nav = div({id: 'sidebar'}, [
			button({id: 'view-header-button', onclick: 'viewHeader()'},
				'Header'),
			button({id: 'view-particle-emitters-button', onclick: 'viewParticleEmitters()'},
				`Particle Emitters <span class="tag">${cardData.data_pack.particle_emitters.length}</span>`),
			button({id: 'view-scenes-button', onclick: 'viewScenes()'},
				`Scenes <span class="tag">${cardData.data_pack.scenes.length}</span>`),
			button({id: 'view-strings-button', onclick: 'viewStrings()'},
				`Strings <span class="tag">${cardData.data_pack.strings.length}</span>`),
			button({id: 'view-table9-button', onclick: 'viewTable9()'},
				`Unknown <span class="tag">${cardData.data_pack.table9.length}</span>`),
			button({id: 'view-items-button', onclick: 'viewItems()'},
				`Items <span class="tag">${cardData.data_pack.items.length}</span>`),
			button({id: 'view-characters-button', onclick: 'viewCharacters()'},
				`Characters <span class="tag">${cardData.data_pack.characters.length}</span>`),
			button({id: 'view-animations-button', onclick: 'viewAnimations()'},
				`Animations <span class="tag">${cardData.data_pack.graphics_nodes.length}</span>`),
			button({id: 'view-frames-button', onclick: 'viewFrames()'},
				`Frames <span class="tag">${cardData.data_pack.frame_groups.length}</span>`),
			button({id: 'view-sprites-button', onclick: 'viewSprites()'},
				`Images <span class="tag">${cardData.sprite_pack.image_defs.length}</span>`),
		])

		contents = div({id: 'contents'})

		main.append(nav)
		main.append(contents)

		viewHeader()
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
	AboutDialog.setup()
}

const closeDialogs = () => {
	AboutDialog.close()
}

const openBin = () => {
	tauri_invoke('open_bin')
}

const exportData = () => {
	tauri_invoke('export_data')
}

const exportImages = () => {
	tauri_invoke('export_images')
}

const selectSection = (sectionId) => {
	for (child of document.getElementById('sidebar').children) {
		child.classList.remove('selected')
	}
	document.getElementById(sectionId).classList.add('selected')
	contents.innerHTML = ''
	contents.scrollTo(0, 0)
}

const setupHeader = () => {
	const header = cardData.header
	return table([
		tbody([
			tr([th('Sector Count'), td(header.sector_count)]),
			tr([th('Checksum'), td(header.checksum)]),
			tr([th('Device IDs'), td(`${header.device_ids.map(id => ` ${id.toString(16)}`)}`)]),
			tr([th('Vendor ID'), td(header.vendor_id)]),
			tr([th('Product ID'), td(header.product_id)]),
			tr([th('Card Type'), td(header.card_type)]),
			tr([th('Card ID'), td(header.card_id)]),
			tr([th('Build Date'), td(`${header.year}-${header.month}-${header.day} revision ${header.revision}`)]),
			tr([th('MD5'), td(header.md5)])
		])
	])
}
const viewHeader = () => {
	selectSection('view-header-button')
	contents.append(sections.header)
}

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

const setupScenes = () => {
	let el = document.createElement('div')
	const scenes = cardData.data_pack.scenes
	scenes.forEach((scene, i) => {
		el.append(div({id: `scene-${i}`, 'class': 'table-title'}, `Scene ${i}`))
		el.append(table([
			thead([tr([
				th('-'),
				th('Props'),
				th('X'),
				th('Y'),
				th('Image ID'),
				th('Subimage Index'),
				th('Preview')
			])]),
			tbody(scene.layers.map((layer, i) => tr({}, [
				th(i),
				td('#' + formatHexCode(layer.props)),
				td(layer.x),
				td(layer.y),
				td(linkToImage(layer.image_id)),
				td(layer.subimage_index),
				td(displayImageWithLink(layer.image_id, layer.subimage_index))
			])))
		]))
	})
	return el
}
const viewScenes = () => {
	selectSection('view-scenes-button')
	contents.append(sections.scenes)
}

const setupStrings = () => {
	const strings = cardData.data_pack.strings
	return table([
		thead([tr([
			th('ID'),
			th('Unknown 1'),
			th('Unknown 2'),
			th('Unknown 3'),
			th('Value')
		])]),
		tbody(strings.map(str => tr({id: `string-${str.id.entity_id}`}, [
			th(str.id.entity_id),
			td('#' + formatHexCode(str.unknown1)),
			td(str.unknown2),
			td(str.unknown3),
			td(str.value)
		])))
	])
}
const viewStrings = () => {
	selectSection('view-strings-button')
	contents.append(sections.strings)
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

const setupItems = () => {
	const items = cardData.data_pack.items
	return table([
		thead([tr([
			th('ID'),
			th('Type'),
			th('Name'),
			th('Image ID'),
			th('Image ID (Worn)'),
			th('Image ID (Close)'),
			th('Unknown 1'),
			th('Price'),
			th('Unknown 2'),
			th('Unknown 3'),
			th('Unknown 4'),
			th('Unlocked Character'),
			th('Game Type')
		])]),
		tbody(items.map(item => tr({id: `item-${item.id.entity_id}`}, [
			th(item.id.entity_id),
			td(item.item_type),
			td(item.name),
			td(item.item_type === 'Game' ? '-' : displayImageWithLink(item.image_id, 0)),
			td(displayImageWithLink(item.worn_image_id, 0)),
			td(displayImageWithLink(item.close_image_id, 0)),
			td('#' + formatHexCode(item.unknown1)),
			td(item.price),
			td('#' + formatHexCode(item.unknown2)),
			td('#' + formatHexCode(item.unknown3)),
			td('#' + formatHexCode(item.unknown4)),
			td(item.unlocked_character != null ? linkToCharacter(item.unlocked_character) : '-'),
			td(item.game_type != null ? item.game_type : '-')
		])))
	])
}
const viewItems = () => {
	selectSection('view-items-button')
	contents.append(sections.items)
}

const setupCharacters = () => {
	const characters = cardData.data_pack.characters
	return table([
		thead([tr([
			th('ID'),
			th('Type'),
			th('Name'),
			th('Profile Image ID'),
			th('Icon Image ID'),
			th('Frame ID'),
			th('Unknown ID'),
			th('Pronoun'),
			th('Statement Ending'),
			th('Question Ending 1'),
			th('Question Ending 2'),
			th('Unknown 2'),
			th('Unknown 3'),
			th('Global ID'),
			th('Unknown 4'),
			th('Unknown 5'),
			th('Unknown 6'),
			th('Unknown 7'),
			th('Gender')
		])]),
		tbody(characters.map(character => tr({id: `character-${character.id.entity_id}`}, [
			th(character.id.entity_id),
			td(character.character_type),
			td(character.name),
			td(displayImageWithLink(character.profile_image_id, 0)),
			td(displayImageWithLink(character.icon_image_id, 0)),
			td(linkToFrame(character.composition_id)),
			td(formatEntityId(character.unknown1)),
			td(character.pronoun),
			td(character.statement),
			td(character.question1),
			td(character.question2),
			td(character.unknown2),
			td(character.unknown3),
			td(formatEntityId(character.global_id)),
			td(character.unknown4),
			td(character.unknown5),
			td('#' + formatHexCode(character.unknown6)),
			td(character.unknown7),
			td(character.gender)
		])))
	])
}
const viewCharacters = () => {
	selectSection('view-characters-button')
	contents.append(sections.characters)
}

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

const setupSprites = () => {
	const imageDefs = cardData.sprite_pack.image_defs
	return table([
		tbody(imageDefs.map((imageDef, i) => tr({id: `image-${i}`}, [
			th(i),
			td(Array(imageDef.subimage_count).fill(0).map((_, j) => {
				return displayImage(i, j, true)
			}))
		])))
	])
}
const viewSprites = () => {
	selectSection('view-sprites-button')
	contents.append(sections.sprites)
}

const el = (type, props, contents) => {
	const element = document.createElement(type)

	if (typeof props === 'object' && props.length == null) {
		for (const propName in props) {
			element.setAttribute(propName, props[propName])
		}
	} else if (contents == null) {
		contents = props
	}

	if (typeof contents === 'string' || !isNaN(contents)) {
		element.innerHTML = contents
	} else if (typeof contents === 'object' && contents.length != null) {
		for (const child of contents) {
			element.append(child)
		}
	}

	return element
}

const div = el.bind(this, 'div')
const button = el.bind(this, 'button')
const table = el.bind(this, 'table')
const thead = el.bind(this, 'thead')
const tbody = el.bind(this, 'tbody')
const tr = el.bind(this, 'tr')
const th = el.bind(this, 'th')
const td = el.bind(this, 'td')

const formatEntityId = (id) => {
	const { card_id, entity_id } = id
	return `${card_id != null ? card_id + '-' : ''}${entity_id}`
}

const formatHexCode = (byte) => {
	return byte.toString(16).padStart(4, 0)
}

const linkToCharacter = (characterIndex) => {
	const characterName = cardData.data_pack.characters[characterIndex].name
	const link = button(characterName)
	link.addEventListener('click', () => {
		viewCharacters()
		const characterEl = document.getElementById(`character-${characterIndex}`)
		if (characterEl != null) {
			characterEl.scrollIntoView()
		}
	})
	return [link]
}

const linkToFrame = (frameId) => {
	const link = button(`Frame Group ${frameId.entity_id}`)
	link.addEventListener('click', () => {
		viewFrames()
		const frameEl = document.getElementById(`framegroup-${frameId.entity_id}`)
		if (frameEl != null) {
			frameEl.scrollIntoView()
		}
	})
	return [link]
}

const linkToImage = (imageId) => {
	if (imageId.card_id != null) {
		const link = button(formatEntityId(imageId))
		link.addEventListener('click', () => {
			viewSprites()
			const imageEl = document.getElementById(`image-${imageId.entity_id}`)
			if (imageEl != null) {
				imageEl.scrollIntoView()
			}
		})
		return [link]
	} else {
		return '-'
	}
}

const linkToSubimage = (imageId, subimageIndex) => {
	if (imageId.card_id != null) {
		const link = button(formatEntityId(imageId))
		link.addEventListener('click', () => {
			viewSprites()
			const subimageEl = document.getElementById(`subimage-${imageId.entity_id}-${subimageIndex}`)
			if (subimageEl != null) {
				subimageEl.scrollIntoView()
			}
		})
		return [link]
	} else {
		return '-'
	}
}

const displayImage = (imageId, subimageIndex, showTooltip) => {
	const img = document.createElement('img')
	img.className = 'preview-image'
	img.id = `subimage-${imageId}-${subimageIndex}`
	if (showTooltip) {
		img.title = `${imageId}-${subimageIndex}`
	}
	img.src = convertFileSrc(`${timestamp}-${imageId}-${subimageIndex}`, 'getimage')
	return img
}

const displayImageWithLink = (imageId, subimageIndex) => {
	if (imageId != null && subimageIndex != null && imageId.card_id == cardData.header.card_id) {
		const img = displayImage(imageId.entity_id, subimageIndex)
		const link = button([img])
		link.addEventListener('click', () => {
			viewSprites()
			const subimageEl = document.getElementById(`subimage-${imageId.entity_id}-${subimageIndex}`)
			if (subimageEl != null) {
				subimageEl.scrollIntoView()
			}
		})

		return [link]

	} else {
		return '-'
	}
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
