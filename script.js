const LITTLE_ENDIAN = true

let clockFaceOffsets = []
let clockFaceLayerOffsets = []
let table10Offsets = []
let animOffsets = []
let table16Offsets = []

window.onload = () => {
	const fileInput = document.getElementById('file-uploader')
	fileInput.addEventListener('change', handleFileUpload, false)
}

const handleFileUpload = () => {
	const fileInput = document.getElementById('file-uploader')
	if (fileInput != null && fileInput.files != null && fileInput.files.length >= 1 && fileInput.files[0] != null) {
		const file = fileInput.files[0]
		const reader = new FileReader()
		reader.addEventListener('load', () => {
			const buffer = reader.result
			const data = new DataView(buffer)
			parseBin(data)
		}, false)
		reader.readAsArrayBuffer(file)
	}
}

const parseBin = (data) => {
	let prevWordIsEmpty = false
	for (let i = 0; i < data.byteLength; i += 2) {
		if (data.getUint16(i) === 0x0000 || data.getUint16(i) === 0xFFFF) {
			prevWordIsEmpty = true
		} else if (prevWordIsEmpty && data.getUint16(i) === 0x3232) {
			parseArchive(new DataView(data.buffer, i))
			break;
		} else {
			prevWordIsEmpty = false
		}
	}
}

const parseArchive = (data) => {
	const packageCount = data.getUint16(2, LITTLE_ENDIAN)

	for (let i = 0; i < packageCount; i++) {
		const packageOffset = data.getUint32((i*16) + 8, LITTLE_ENDIAN)
		const packageSize = data.getUint32((i*16) + 16, LITTLE_ENDIAN)

		if (packageOffset > 0 && packageSize > 0) {
			const packageData = new DataView(data.buffer, data.byteOffset + packageOffset, packageSize)
			if (i === 0) {
				parseDataDefs(packageData)
			} else if (i === 1) {
				parseSprites(packageData)
			}
		}
	}
}

const parseDataDefs = (data) => {
	const tableDataEl = document.getElementById('table-data')
	tableDataEl.innerHTML = ''

	let tableOffsets = []
	let tableSizes = []

	for (let i = 0; i < 20; i++) {
		const tableOffset = data.getUint32(i*4, LITTLE_ENDIAN) * 2 // offsets are in 16-bit words, not bytes
		tableOffsets.push(tableOffset)
		if (i >= 1) {
			const tableSize = tableOffsets[i] - tableOffsets[i-1]
			tableSizes.push(tableSize)
		}
	}

	tableSizes.push(data.byteLength - tableOffsets[19])

	for (let i = 0; i < 20; i++) {
		const tableHeaderEl = document.createElement('h3')
		tableDataEl.append(tableHeaderEl)
		tableHeaderEl.className = 'collapse'
		tableHeaderEl.innerText = TABLE_NAMES[i] || `Table ${i + 1}`
		tableHeaderEl.addEventListener('click', () => tableHeaderEl.classList.toggle('collapse'))

		if (tableSizes[i] > 0) {
			const tableData = new DataView(data.buffer, data.byteOffset + tableOffsets[i], tableSizes[i])

			if (i === 3) {
				parseOffsetTable(tableData, 4)
			} else if (i === 4) {
				parseClockFaceOffsetTable(tableData)
			} else if (i === 5) {
				parseClockFaceTable(tableData)
			} else if (i === 6) {
				parseDialogTable(tableData)
			} else if (i === 7) {
				parseOffsetTable(tableData, 6)
			} else if (i === 8) {
				parseOffsetTable(tableData, 9)
			} else if (i === 10) {
				parseItemTable(tableData)
			} else if (i === 11) {
				parseTamaTable(tableData)
			} else if (i === 13) {
				parseOffsetTable(tableData, 14)
			} else if (i === 14) {
				parseAnimationTable(tableData)
			} else if (i === 15) {
				parseTable17(new DataView(data.buffer, data.byteOffset + tableOffsets[16], tableSizes[16]), false)
				parseTable16(tableData)
			} else if (i === 16) {
				parseTable17(tableData, true)
			} else {
				parseTable(tableData, i)
				tableHeaderEl.className = 'collapse'
			}

		} else {
			const tableContentEl = document.createElement('code')
			tableDataEl.append(tableContentEl)
			tableContentEl.innerText = 'empty'
		}
	}
}

const parseTable = (data, tableIndex) => {
	const tableDataEl = document.getElementById('table-data')

	const tableContentEl = document.createElement('code')
	tableDataEl.append(tableContentEl)
	tableContentEl.innerHTML = `(size: ${data.byteLength} bytes | ${data.byteLength / 2} words)<br><br>`

	for (let i = 0; i < data.byteLength; i += 2) {
		const wordEl = document.createElement('span')
		tableContentEl.append(wordEl)
		wordEl.innerText = stringifyWord(data, i) + ' '

		if (tableIndex === 9 && table10Offsets.includes((i/2)+1)) {
			tableContentEl.append(document.createElement('br'))
			tableContentEl.append(document.createElement('br'))
		}
	}
}

const parseOffsetTable = (data, targetTable) => {
	const tableDataEl = document.getElementById('table-data')

	const tableContentEl = document.createElement('code')
	tableDataEl.append(tableContentEl)
	tableContentEl.innerHTML = `(size: ${data.byteLength} bytes | ${data.byteLength / 2} words)<br><br>`

	let offsetList = []
	for (let i = 0; i < data.byteLength; i += 2) {
		let offset = data.getUint16(i, LITTLE_ENDIAN)
		offsetList.push(offset)

		const wordEl = document.createElement('span')
		tableContentEl.append(wordEl)
		wordEl.innerText = `${offset} `
	}

	if (targetTable === 4) {
		clockFaceOffsets = offsetList
	} else if (targetTable === 9) {
		table10Offsets = offsetList
	} else if (targetTable === 14) {
		animOffsets = offsetList
	}
}

const parseClockFaceOffsetTable = (data) => {
	const tableDataEl = document.getElementById('table-data')

	const tableEl = document.createElement('table')
	tableDataEl.append(tableEl)
	tableEl.innerHTML = '<thead><tr><th>offset</th><th>layer offsets</th></tr></thead>'

	const tableBodyEl = document.createElement('tbody')
	tableEl.append(tableBodyEl)

	let clockFaces = []
	let currentClockFace = []
	for (let i = 0; i < data.byteLength; i += 2) {
		if (clockFaceOffsets.includes(i/2)) {
			if (currentClockFace.length > 0) {
				clockFaces.push(currentClockFace)
			}
			currentClockFace = []
		}
		let offset = data.getUint16(i, LITTLE_ENDIAN)
		currentClockFace.push(offset)
	}
	clockFaces.push(currentClockFace)

	for (let i = 0; i < clockFaces.length; i++) {
		const tableRowEl = document.createElement('tr')
		tableBodyEl.append(tableRowEl)
		tableRowEl.innerHTML = `<td>${clockFaceOffsets[i]}</td><td>${clockFaces[i].join(' ')}</td>`
	}

	clockFaceLayerOffsets = clockFaces
}

const parseClockFaceTable = (data) => {
	const tableDataEl = document.getElementById('table-data')

	const clockDiv = document.createElement('div')
	tableDataEl.append(clockDiv)

	for (let i = 0; i < clockFaceLayerOffsets.length; i++) {
		let clock = clockFaceLayerOffsets[i]

		const headerEl = document.createElement('h4')
		clockDiv.append(headerEl)
		headerEl.innerText = `Clock Face ${i+1}`

		const tableEl = document.createElement('table')
		clockDiv.append(tableEl)
		tableEl.innerHTML = `
			<thead><tr>
				<th>offset</th>
				<th>layer<br>type (?)</th>
				<th>x</th>
				<th>y</th>
				<th>image set</th>
				<th>?</th>
			</tr></thead>`

		const tableBodyEl = document.createElement('tbody')
		tableEl.append(tableBodyEl)

		for (let j = 0; j < clock.length; j++) {
			const offset = clock[j] * 2
			const layerType = stringifyWord(data, offset)
			const x = data.getInt16(offset + 2, LITTLE_ENDIAN)
			const y = data.getInt16(offset + 4, LITTLE_ENDIAN)
			const imageSet = data.getUint16(offset + 6, LITTLE_ENDIAN) & 0xff
			let flag = 0
			if (j+1 < clock.length && clock[j+1] > clock[j] + 4) {
				flag = stringifyWord(data, offset + 8)
			}

			const tableRowEl = document.createElement('tr')
			tableBodyEl.append(tableRowEl)
			tableRowEl.innerHTML = `
				<td>${offset / 2}</td>
				<td>${layerType || '-'}</td>
				<td>${x || '-'}</td>
				<td>${y || '-'}</td>
				<td>${imageSet ? `<a href="#image-set-${imageSet}">${imageSet}</a>` : '-'}</td>
				<td>${flag || '-'}</td>`
		}
	}
}

const parseDialogTable = (data) => {
	const tableDataEl = document.getElementById('table-data')

	const tableEl = document.createElement('table')
	tableDataEl.append(tableEl)
	tableEl.innerHTML = `
		<thead><tr>
			<th>offset</th>
			<th>id</th>
			<th>flags</th>
			<th>string</th>
		</tr></thead>`

	const tableBodyEl = document.createElement('tbody')
	tableEl.append(tableBodyEl)


	let i = 0
	while (i + 10 <= data.byteLength) {
		const id = stringifyWord(data, i)
		const flag1 = stringifyWord(data, i + 2)
		const flag2 = stringifyWord(data, i + 4)
		const flag3 = stringifyWord(data, i + 6)

		// null-terminating string
		let strLength = 0
		while (data.getUint16(i + 8 + strLength*2) !== 0) {
			strLength += 1
		}
		const dialogStr = parseString(data, i + 8, strLength)

		const tableRowEl = document.createElement('tr')
		tableBodyEl.append(tableRowEl)
		tableRowEl.innerHTML = `
			<td>${i / 2}</td>
			<td>${id}</td>
			<td>${flag1} ${flag2} ${flag3}</td>
			<td>${dialogStr}</td>`

		i += 10 + (strLength*2)
	}
}

const parseItemTable = (data) => {
	const tableDataEl = document.getElementById('table-data')

	const tableEl = document.createElement('table')
	tableDataEl.append(tableEl)
	tableEl.innerHTML = `
		<thead><tr>
			<th>offset</th>
			<th>id</th>
			<th>type</th>
			<th>name</th><th>image set</th>
			<th>image set<br><small>worn</small></th>
			<th>image set<br><small>close-up</small></th>
			<th>animation<br>id (?)</th>
			<th>??</th>
			<th>??</th>
			<th>??</th>
			<th>??</th>
			<th>unlocked<br>character</th>
		</tr></thead>`

	const tableBodyEl = document.createElement('tbody')
	tableEl.append(tableBodyEl)

	let i = 0
	while (i + 42 <= data.byteLength) {
		const id = stringifyWord(data, i)
		const typeIndex = data.getUint16(i + 2, LITTLE_ENDIAN)
		const type = ITEM_TYPES[typeIndex] || typeIndex
		const itemName = parseString(data, i + 4, 10)
		const imageSet = data.getUint16(i + 24, LITTLE_ENDIAN) & 0xff
		const imageSetWorn = data.getUint16(i + 26, LITTLE_ENDIAN) & 0xff
		const imageSetCloseUp = data.getUint16(i + 28, LITTLE_ENDIAN) & 0xff
		const animId = data.getUint16(i + 30, LITTLE_ENDIAN) & 0xff
		const flag2 = stringifyWord(data, i + 32)
		const flag3 = stringifyWord(data, i + 34)
		const flag4 = stringifyWord(data, i + 36)
		const flag5 = stringifyWord(data, i + 38)

		let unlockedCharacter = data.getUint16(i + 40, LITTLE_ENDIAN)
		let gameType = i === 0 ? GAME_TYPES[unlockedCharacter] : ''

		const tableRowEl = document.createElement('tr')
		tableBodyEl.append(tableRowEl)
		tableRowEl.innerHTML = `
			<td>${i / 2}</td>
			<td>${id}</td>
			<td>${type}</td>
			<td>${itemName}</td>
			<td>${imageSet ? `<a href="#image-set-${imageSet}">${imageSet}</a>` : '-'}</td>
			<td>${imageSetWorn ? `<a href="#image-set-${imageSetWorn}">${imageSetWorn}</a>` : '-'}</td>
			<td>${imageSetCloseUp ? `<a href="#image-set-${imageSetCloseUp}">${imageSetCloseUp}</a>` : '-'}</td>
			<td>${animId ? `<a href="#anim-${animId}">${animId}</a>` : '-'}</td>
			<td>${flag2}</td>
			<td>${flag3}</td>
			<td>${flag4}</td>
			<td>${flag5}</td>
			<td>${i === 0 ? gameType : (unlockedCharacter ? `<a href="#tama-${unlockedCharacter}">${unlockedCharacter}</a>` : '-')}</td>`

		i += 42
	}
}

const parseTamaTable = (data) => {
	const tableDataEl = document.getElementById('table-data')

	const tableEl = document.createElement('table')
	tableDataEl.append(tableEl)
	tableEl.innerHTML = `
		<thead><tr>
			<th>offset</th>
			<th>id</th>
			<th>type</th>
			<th>name</th>
			<th>memory<br>image</th>
			<th>icon</th>
			<th>id again?</th>
			<th>??</th>
			<th>pronoun</th>
			<th>statement<br><small>{ndesu}<small></th>
			<th>question 1<br><small>{ndesuka}<small></th>
			<th>question 2<br><small>{desuka}<small></th>
			<th>??</th>
			<th>??</th>
			<th>original<br>card id</th>
			<th>??</th>
			<th>??</th>
			<th>??</th>
			<th>??</th>
			<th>gender</th>
		</tr></thead>`

	const tableBodyEl = document.createElement('tbody')
	tableEl.append(tableBodyEl)

	let i = 0
	while (i + 96 <= data.byteLength) {

		const id = stringifyWord(data, i)
		const type = data.getUint16(i + 2, LITTLE_ENDIAN)
		const tamaName = parseString(data, i + 4, 10)
		const memoryIndex = data.getUint16(i + 24, LITTLE_ENDIAN) & 0xff
		const iconIndex = data.getUint16(i + 26, LITTLE_ENDIAN) & 0xff
		const flag3 = stringifyWord(data, i + 28)
		const flag4 = stringifyWord(data, i + 30)
		const pronoun = parseString(data, i + 32, 6)
		const statement = parseString(data, i + 44, 6)
		const question1 = parseString(data, i + 56, 6)
		const question2 = parseString(data, i + 68, 6)
		const other1 = stringifyWord(data, i + 80)
		const other2 = stringifyWord(data, i + 82)
		const originalId = stringifyWord(data, i + 84)
		const other4 = stringifyWord(data, i + 86)
		const other5 = stringifyWord(data, i + 88)
		const other6 = stringifyWord(data, i + 90)
		const other7 = stringifyWord(data, i + 92)
		const gender = data.getUint16(i + 94, LITTLE_ENDIAN) ? 'M' : 'F'

		const tableRowEl = document.createElement('tr')
		tableBodyEl.append(tableRowEl)
		tableRowEl.id = `tama-${data.getUint16(i, LITTLE_ENDIAN) & 0xff}`
		tableRowEl.innerHTML = `
			<td>${i / 2}</td>
			<td>${id}</td>
			<td>${type}</td>
			<td>${tamaName}</td>
			<td>${memoryIndex ? `<a href="#image-set-${memoryIndex}">${memoryIndex}</a>` : '-'}</td>
			<td>${iconIndex ? `<a href="#image-set-${iconIndex}">${iconIndex}</a>` : '-'}</td>
			<td>${flag3}</td>
			<td>${flag4}</td>
			<td>${pronoun}</td>
			<td>${statement}</td>
			<td>${question1}</td>
			<td>${question2}</td>
			<td>${other1}</td>
			<td>${other2}</td>
			<td>${originalId}</td>
			<td>${other4}</td>
			<td>${other5}</td>
			<td>${other6}</td>
			<td>${other7}</td>
			<td>${gender}</td>`

		i += 96
	}
}

const parseAnimationTable = (data) => {
	const tableDataEl = document.getElementById('table-data')

	const tableEl = document.createElement('table')
	tableDataEl.append(tableEl)
	tableEl.innerHTML = `
		<thead><tr>
			<th>offset</th>
			<th>id</th>
			<th>data</th>
		</tr></thead>`

	const tableBodyEl = document.createElement('tbody')
	tableEl.append(tableBodyEl)

	let sequences = []

	for (let i = 0; i < animOffsets.length; i++) {
		let sequence = []
		const offset = animOffsets[i] * 2
		if (i + 1 < animOffsets.length) {
			const nextOffset = animOffsets[i + 1] * 2
			const bytesInSequence = nextOffset - offset
			for (let j = 0; j < bytesInSequence; j += 2) {
				const word = stringifyWord(data, offset + j)
				sequence.push(word)
			}
			sequences.push(sequence)
		}
	}

	for (let i = 0; i < sequences.length; i++) {
		const tableRowEl = document.createElement('tr')
		tableBodyEl.append(tableRowEl)
		tableRowEl.id = `anim-${i}`
		tableRowEl.innerHTML = `
			<td>${animOffsets[i]}</td>
			<td>${i}</td>
			<td>${sequences[i].join(' ')}</td>`
	}
}

const parseTable16 = (data) => {
	const tableDataEl = document.getElementById('table-data')

	const tableContentEl = document.createElement('code')
	tableDataEl.append(tableContentEl)
	tableContentEl.innerHTML = `(size: ${data.byteLength} bytes | ${data.byteLength / 2} words)<br><br>`

	for (let i = 0; i < data.byteLength; i += 2) {
		const wordEl = document.createElement('span')
		tableContentEl.append(wordEl)
		wordEl.innerText = stringifyWord(data, i) + ' '

		if (table16Offsets.includes((i/2)+1)) {
			tableContentEl.append(document.createElement('br'))
			tableContentEl.append(document.createElement('br'))
		}
	}
}

const parseTable17 = (data, appendEl) => {
	const tableDataEl = document.getElementById('table-data')
	const tableContentEl = document.createElement('code')

	let offsets = []
	for (let i = 0; i < data.byteLength; i += 4) {
		const wordEl = document.createElement('span')
		tableContentEl.append(wordEl)

		let offset = data.getUint32(i, LITTLE_ENDIAN)
		if (offset > 0) {
			offsets.push(offset)
		}
		wordEl.innerText = `${offset} `
	}

	if (appendEl) {
		tableDataEl.append(tableContentEl)
	} else {
		table16Offsets = offsets
	}
}

const parseString = (data, offset, length) => {
	let str = ''
	for (let i = 0; i < length; i++) {
		const value = data.getUint16(offset + i*2, LITTLE_ENDIAN)
		const char = TEXT_ENCODING[value] || `[${stringifyWord(data, offset + i*2)}]`
		str = `${str}${char}`
	}
	return str
}

const stringifyWord = (data, offset) => {
	return data.getUint16(offset, LITTLE_ENDIAN).toString(16).padStart(4, '0').toUpperCase()
}

const parseSprites = (data) => {
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
