const el = (type, props, contents) => {
	const element = document.createElement(type)

	if (typeof props === 'object' && props != null && props.length == null) {
		for (const propName in props) {
			const propValue = props[propName]
			if (propName === 'onclick') {
				element.addEventListener('click', propValue)
			} else if (propName === 'onchange') {
				element.addEventListener('change', propValue)
			} else if (propName === 'onkeyup') {
				element.addEventListener('keyup', propValue)
			} else if (propName === 'className') {
				element.className = propValue
			} else {
				element.setAttribute(propName, propValue)
			}
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
const span = el.bind(this, 'span')
const a = el.bind(this, 'a')
const button = el.bind(this, 'button')
const select = el.bind(this, 'select')
const option = el.bind(this, 'option')
const input = el.bind(this, 'input')
const label = el.bind(this, 'label')
const table = el.bind(this, 'table')
const thead = el.bind(this, 'thead')
const tbody = el.bind(this, 'tbody')
const tr = el.bind(this, 'tr')
const th = el.bind(this, 'th')
const td = el.bind(this, 'td')
const br = el.bind(this, 'br')
const hr = el.bind(this, 'hr')
const img = el.bind(this, 'img')

const formatEntityId = (id) => {
	const { card_id, entity_id } = id
	return `${card_id != null ? card_id + '-' : ''}${entity_id}`
}

const formatHexCode = (byte) => {
	return byte.toString(16).padStart(4, 0)
}

const linkToCharacter = (characterIndex) => {
	if (cardData.data_pack.characters[characterIndex] != null) {
		const characterName = cardData.data_pack.characters[characterIndex].name.string
		const link = button(characterName)
		link.addEventListener('click', () => {
			viewCharacters()
			const characterEl = document.getElementById(`character-${characterIndex}`)
			if (characterEl != null) {
				characterEl.scrollIntoView()
			}
		})
		return [link]
	} else {
		return [div('-')]
	}
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
	if (cardData.card_header == null || imageId.card_id === cardData.card_header.card_id) {
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
		return [div(formatEntityId(imageId))]
	}
}

const displayImage = (imageId, subimageIndex, showTooltip, subimageOffset = 0) => {
	const img = document.createElement('img')
	img.className = 'preview-image'
	img.id = `subimage-${imageId}-${subimageOffset + subimageIndex}`
	if (showTooltip) {
		img.title = `${imageId}-${subimageIndex}`
	}
	img.src = convertFileSrc(`${timestamp}-${imageId}-${subimageOffset + subimageIndex}`, 'getimage')
	return img
}

const displayImageWithLink = (imageId, subimageIndex) => {
	if (imageId != null && subimageIndex != null && (cardData.card_header == null || imageId.card_id === cardData.card_header.card_id)) {
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
