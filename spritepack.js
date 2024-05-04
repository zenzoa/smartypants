const parseSpritePack = (data) => {
	let imageDefs = []
	let spriteDefs = []
	let palettes = []

	const imageDefsOffset = data.getUint32(0, LITTLE_ENDIAN)
	const spriteDefsOffset = data.getUint32(4, LITTLE_ENDIAN)
	const palettesOffset = data.getUint32(8, LITTLE_ENDIAN)
	const charactersOffset = data.getUint32(12, LITTLE_ENDIAN)

	imageDefs = parseImageDefs(new DataView(data.buffer, data.byteOffset + imageDefsOffset, spriteDefsOffset - imageDefsOffset))
	spriteDefs = parseSpriteDefs(new DataView(data.buffer, data.byteOffset + spriteDefsOffset, palettesOffset - spriteDefsOffset))
	palettes = parsePalettes(new DataView(data.buffer, data.byteOffset + palettesOffset, charactersOffset - palettesOffset))

	const characterData = new DataView(data.buffer, data.byteOffset + charactersOffset, data.byteLength - charactersOffset)
	for (const spriteDef of spriteDefs) {
		spriteDef.charData = parseSpriteData(characterData, spriteDef)
	}

	const imageEl = document.getElementById('images')
	imageEl.innerHTML = ''

	for (let i = 0; i < imageDefs.length; i++) {
		const imageDef = imageDefs[i]

		const nextSpriteIndex = imageDefs[i+1] ? imageDefs[i+1].spriteStartIndex : spriteDefs.length
		const spriteCount = nextSpriteIndex - imageDef.spriteStartIndex
		const sprites = spriteDefs.slice(imageDef.spriteStartIndex, imageDef.spriteStartIndex + spriteCount)

		const colors = palettes.slice(imageDef.paletteStartIndex).flat()

		const imageSetHeaderEl = document.createElement('h3')
		imageSetHeaderEl.innerText = `Image Set ${i}`
		imageSetHeaderEl.id = `image-set-${i}`
		imageSetHeaderEl.addEventListener('click', () => imageSetHeaderEl.classList.toggle('collapse'))
		imageEl.append(imageSetHeaderEl)

		const imageSetEl = drawImageSet(imageDef, sprites, colors)
		imageEl.append(imageSetEl)
	}
}

const parseImageDefs = (data) => {
	let imageDefs = []
	for (let i = 0; i < data.byteLength; i += 6) {
		const imageDef = {
			spriteStartIndex: data.getUint16(i, LITTLE_ENDIAN),
			width: data.getUint8(i + 2),
			height: data.getUint8(i + 3),
			paletteStartIndex: data.getUint16(i + 4, LITTLE_ENDIAN)
		}
		imageDefs.push(imageDef)
	}
	return imageDefs
}

const parseSpriteDefs = (data) => {
	let spriteDefs = []
	for (let i = 0; i < data.byteLength; i += 8) {
		const attributes = data.getUint16(i + 6, LITTLE_ENDIAN)
		const spriteDef = {
			charNum: data.getUint16(i, LITTLE_ENDIAN),
			offsetX: data.getInt16(i + 2, LITTLE_ENDIAN), // relative to pivot point
			offsetY: data.getInt16(i + 4, LITTLE_ENDIAN), // relative to pivot point
			bpp: BITS_PER_PIXEL[attributes & 0x0003],
			isFlipped: (attributes & 0x000c) >> 2, // unused on Smart
			width: CHAR_SIZES[(attributes & 0x0030) >> 4],
			height: CHAR_SIZES[(attributes & 0x00c0) >> 6],
			paletteBank: (attributes & 0x0f00) >> 8,
			drawDepth: (attributes & 0x3000) >> 12, // typically not set
			blendEnabled: (attributes & 0x4000) >> 14, // typically not set
			isQuadrupled: (attributes & 0x8000) >> 15 // unused on Smart
		}
		spriteDefs.push(spriteDef)
	}
	return spriteDefs
}

const parseSpriteData = (data, spriteDef) => {
	let charData = []

	const charSize = spriteDef.width * spriteDef.height
	const numBytes = charSize * spriteDef.bpp / 8
	const charOffset = numBytes * spriteDef.charNum

	let bits = []
	for (let i = 0; i < numBytes; i++) {
		const byte = data.getUint8(charOffset + i)
		for (let b = 7; b >= 0; b--) {
			const bit = (byte >> b) & 1
			bits.push(bit)
		}
	}

	for (let i = 0; i < charSize; i++) {
		const bitOffset = i * spriteDef.bpp
		const colorBits = bits.slice(bitOffset, bitOffset + spriteDef.bpp)
		const colorByte = colorBits.join('')
		const colorIndex = parseInt(colorByte, 2)
		charData.push(colorIndex)
	}

	return charData
}

const parsePalettes = (data) => {
	let palettes = []
	let currentPalette = []
	for (let i = 0; i < data.byteLength; i += 2) {
		const colorByte = data.getUint16(i, LITTLE_ENDIAN)
		const color = {
			a: colorByte >> 15,
			r: (colorByte & 0x7c00) >> 7,
			g: (colorByte & 0x03e0) >> 2,
			b: (colorByte & 0x001f) << 3
		}
		currentPalette.push(color)
		if (currentPalette.length === 4) {
			palettes.push(currentPalette)
			currentPalette = []
		}
	}
	return palettes
}

const drawImageSet = (imageDef, sprites, colors) => {
	const imageSetEl = document.createElement('div')
	imageSetEl.className = 'image-set'
	const spritesPerImage = imageDef.width * imageDef.height
	const imageCount = sprites.length / spritesPerImage

	for (let i = 0; i < imageCount; i++) {
		const imageEl = document.createElement('div')
		imageEl.className = 'image'
		if (spritesPerImage === 1) {
			imageEl.innerHTML = `<div>${imageDef.spriteStartIndex + i}</div>`
		} else {
			imageEl.innerHTML = `<div>${imageDef.spriteStartIndex + (i * spritesPerImage)}-${imageDef.spriteStartIndex + (i * spritesPerImage) + spritesPerImage - 1}</div>`
		}
		imageSetEl.append(imageEl)

		const firstSpriteIndex = spritesPerImage * i
		const firstSprite = sprites[firstSpriteIndex]

		const canvas = document.createElement('canvas')
		canvas.width = imageDef.width * firstSprite.width
		canvas.height = imageDef.height * firstSprite.height
		imageEl.append(canvas)
		const ctx = canvas.getContext('2d')

		for (let row = 0; row < imageDef.height; row++) {
			for (let col = 0; col < imageDef.width; col++) {
				const sprite = sprites[firstSpriteIndex + (row * imageDef.width) + col]
				for (let y = 0; y < sprite.height; y++) {
					for (let x = 0; x < sprite.width; x++) {
						const colorIndex = sprite.charData[y * sprite.width + x]
						const color = colors[colorIndex]
						if (color.a === 0) {
							ctx.fillStyle = `rgb(${color.r} ${color.g} ${color.b})`
							ctx.fillRect((col * sprite.width) + x, (row * sprite.height) + y, 1, 1)
						}
					}
				}
			}
		}
	}

	return imageSetEl
}
