const LITTLE_ENDIAN = true

let cardId = 0
let cardIdString = ''

let clockFaceOffsets = []
let clockFaceLayerOffsets = []
let table10Offsets = []
let animOffsets = []
let compSeqOffsets = []
let compGroupInfo = []

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

	cardId = data.getUint16(tableOffsets[19], LITTLE_ENDIAN)
	cardIdString = (128 + cardId).toString(16).toUpperCase()

	for (let i = 0; i < 20; i++) {
		const tableHeaderEl = document.createElement('h3')
		tableDataEl.append(tableHeaderEl)
		tableHeaderEl.className = 'collapse'
		tableHeaderEl.innerText = TABLE_NAMES[i] || `Table ${i + 1}`
		tableHeaderEl.addEventListener('click', () => tableHeaderEl.classList.toggle('collapse'))

		if (tableSizes[i] > 0) {
			const tableData = new DataView(data.buffer, data.byteOffset + tableOffsets[i], tableSizes[i])

			if (i === 2) {
				parseTableThree(tableData)
			} else if (i === 3) {
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
				parseCompSeqOffsets(new DataView(data.buffer, data.byteOffset + tableOffsets[16], tableSizes[16]), true)
				parseCompGroupInfo(new DataView(data.buffer, data.byteOffset + tableOffsets[18], tableSizes[18]), true)
				parseCompositionTable(tableData, i)
			} else if (i === 16) {
				parseCompSeqOffsets(tableData)
			} else if (i === 18) {
				parseCompGroupInfo(tableData)
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

		const word = data.getUint16(i, LITTLE_ENDIAN)
		const wordString = stringifyWord(data, i)
		if (wordString.startsWith(cardIdString)) {
			wordEl.innerHTML = `<a href="#image-set-${word & 0xff}">${wordString}</a> `
		} else {
			wordEl.innerText = `${wordString} `
		}

		if ((tableIndex === 9 && table10Offsets.includes(i+2)) || (tableIndex === 18 && (i+2) % 4 === 0)) {
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
		clockFaceOffsets = offsetList.map(o => o * 2)
	} else if (targetTable === 9) {
		table10Offsets = offsetList.map(o => o * 2)
	} else if (targetTable === 14) {
		animOffsets = offsetList.map(o => o * 4)
	}
}

const parseTableThree = (data) => {
	const tableDataEl = document.getElementById('table-data')

	const tableEl = document.createElement('table')
	tableDataEl.append(tableEl)

	const tableBodyEl = document.createElement('tbody')
	tableEl.append(tableBodyEl)

	const rowCount = data.byteLength / 66

	for (let i = 0; i < rowCount; i ++) {
		const tableRowEl = document.createElement('tr')
		tableBodyEl.append(tableRowEl)

		const tableCellEl = document.createElement('td')
		tableRowEl.append(tableCellEl)

		for (let j = 0; j < 66; j += 2) {
			const word = data.getUint16(i*66 + j, LITTLE_ENDIAN)
			const wordString = stringifyWord(data, i*66 + j)
			if (wordString.startsWith(cardIdString)) {
				tableCellEl.innerHTML += `<a href="#image-set-${word & 0xff}">${word & 0xff}</a> `
			} else {
				tableCellEl.innerHTML += `${wordString} `
			}
		}
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
		if (clockFaceOffsets.includes(i)) {
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
		tableRowEl.innerHTML = `<td>${clockFaceOffsets[i] / 2}</td><td>${clockFaces[i].join(' ')}</td>`
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
			<th>id</th>
			<th>??</th>
			<th>??</th>
			<th>??</th>
			<th>string</th>
		</tr></thead>`

	const tableBodyEl = document.createElement('tbody')
	tableEl.append(tableBodyEl)


	let i = 0
	while (i + 10 <= data.byteLength) {
		const id = data.getUint16(i, LITTLE_ENDIAN) & 0xff
		const flag1 = data.getUint16(i + 2, LITTLE_ENDIAN) ? stringifyWord(data, i + 2) : '-'
		const flag2 = data.getUint16(i + 4, LITTLE_ENDIAN) || '-'
		const flag3 = data.getUint16(i + 6, LITTLE_ENDIAN) || '-'

		// null-terminating string
		let strLength = 0
		while (data.getUint16(i + 8 + strLength*2) !== 0) {
			strLength += 1
		}
		const dialogStr = parseString(data, i + 8, strLength)

		const tableRowEl = document.createElement('tr')
		tableBodyEl.append(tableRowEl)
		tableRowEl.innerHTML = `
			<td>${id}</td>
			<td>${flag1}</td>
			<td>${flag2}</td>
			<td>${flag3}</td>
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
			<th>id</th>
			<th>type</th>
			<th>name</th><th>image set</th>
			<th>image set<br><small>worn</small></th>
			<th>image set<br><small>close-up</small></th>
			<th>??</th>
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
		const id = data.getUint16(i, LITTLE_ENDIAN) & 0xff
		const typeIndex = data.getUint16(i + 2, LITTLE_ENDIAN)
		const type = ITEM_TYPES[typeIndex] || typeIndex
		const itemName = parseString(data, i + 4, 10)
		const imageSet = data.getUint16(i + 24, LITTLE_ENDIAN) & 0xff
		const imageSetWorn = data.getUint16(i + 26, LITTLE_ENDIAN) & 0xff
		const imageSetCloseUp = data.getUint16(i + 28, LITTLE_ENDIAN) & 0xff
		const animId = (data.getUint16(i + 30, LITTLE_ENDIAN) & 0xff) || '-'
		const flag2 = data.getUint16(i + 32, LITTLE_ENDIAN) || '-'
		const flag3 = data.getUint16(i + 34, LITTLE_ENDIAN) || '-'
		const flag4 = data.getUint16(i + 36, LITTLE_ENDIAN) || '-'
		const flag5 = data.getUint16(i + 38, LITTLE_ENDIAN) ? stringifyWord(data, i + 38) : '-'

		let unlockedCharacter = data.getUint16(i + 40, LITTLE_ENDIAN)
		let gameType = i === 0 ? GAME_TYPES[unlockedCharacter] : ''

		const tableRowEl = document.createElement('tr')
		tableBodyEl.append(tableRowEl)
		tableRowEl.innerHTML = `
			<td>${id}</td>
			<td>${type}</td>
			<td>${itemName}</td>
			<td>${imageSet ? `<a href="#image-set-${imageSet}">${imageSet}</a>` : '-'}</td>
			<td>${imageSetWorn ? `<a href="#image-set-${imageSetWorn}">${imageSetWorn}</a>` : '-'}</td>
			<td>${imageSetCloseUp ? `<a href="#image-set-${imageSetCloseUp}">${imageSetCloseUp}</a>` : '-'}</td>
			<td>${animId}</td>
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
		const id = data.getUint16(i, LITTLE_ENDIAN) & 0xff
		const idString = stringifyWord(data, i)
		const type = data.getUint16(i + 2, LITTLE_ENDIAN)
		const tamaName = parseString(data, i + 4, 10)
		const memoryIndex = data.getUint16(i + 24, LITTLE_ENDIAN) & 0xff
		const iconIndex = data.getUint16(i + 26, LITTLE_ENDIAN) & 0xff
		const idAgain = stringifyWord(data, i + 28)
		const flag4 = data.getUint16(i + 30, LITTLE_ENDIAN) ? stringifyWord(data, i + 30) : '-'
		const pronoun = parseString(data, i + 32, 6)
		const statement = parseString(data, i + 44, 6)
		const question1 = parseString(data, i + 56, 6)
		const question2 = parseString(data, i + 68, 6)
		const other1 = data.getUint16(i + 80, LITTLE_ENDIAN)
		const other2 = stringifyWord(data, i + 82)
		const originalId = stringifyWord(data, i + 84)
		const other4 = data.getUint16(i + 86, LITTLE_ENDIAN)
		const other5 = data.getUint16(i + 88, LITTLE_ENDIAN)
		const other6 = stringifyWord(data, i + 90)
		const other7 = data.getUint16(i + 92, LITTLE_ENDIAN)
		const gender = data.getUint16(i + 94, LITTLE_ENDIAN) ? 'M' : 'F'

		const tableRowEl = document.createElement('tr')
		tableBodyEl.append(tableRowEl)
		tableRowEl.id = `tama-${data.getUint16(i, LITTLE_ENDIAN) & 0xff}`
		tableRowEl.innerHTML = `
			<td>${id}</td>
			<td>${type}</td>
			<td>${tamaName}</td>
			<td>${memoryIndex ? `<a href="#image-set-${memoryIndex}">${memoryIndex}</a>` : '-'}</td>
			<td>${iconIndex ? `<a href="#image-set-${iconIndex}">${iconIndex}</a>` : '-'}</td>
			<td ${idString === idAgain ? 'class="fade"' : ''}>${idAgain}</td>
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
			<th>offset<br>x2</th>
			<th>id</th>
			<th>data</th>
		</tr></thead>`

	const tableBodyEl = document.createElement('tbody')
	tableEl.append(tableBodyEl)

	let sequences = []
	for (let i = 0; i < animOffsets.length; i++) {
		let sequence = []
		const offset = animOffsets[i]
		if (i + 1 < animOffsets.length) {
			const nextOffset = animOffsets[i + 1]
			const bytesInSequence = nextOffset - offset
			for (let j = 0; j < bytesInSequence; j += 2) {
				const word = data.getUint16(offset + j, LITTLE_ENDIAN)
				const wordString = stringifyWord(data, offset + j)
				if (wordString.startsWith(cardIdString)) {
					sequence.push(`<a href="#image-set-${word & 0xff}">${wordString}</a>`)
				} else {
					sequence.push(`${wordString}`)
				}
			}
			sequences.push(sequence)
		}
	}

	for (let i = 0; i < sequences.length; i++) {
		const tableRowEl = document.createElement('tr')
		tableBodyEl.append(tableRowEl)
		tableRowEl.id = `anim-${i}`
		tableRowEl.innerHTML = `
			<td>${animOffsets[i] / 2}</td>
			<td>${i}</td>
			<td>${sequences[i].join(' ')}</td>`
	}
}

const parseCompositionTable = (data) => {
	let sequences = []
	for (let i = 0; i < compSeqOffsets.length; i++) {
		let sequence = []
		const offset = compSeqOffsets[i]
		if (i + 1 < compSeqOffsets.length) {
			const nextOffset = compSeqOffsets[i + 1]
			const bytesInSequence = nextOffset - offset
			for (let j = 0; j < bytesInSequence; j += 2) {
				const word = data.getUint16(offset + j, LITTLE_ENDIAN)
				sequence.push(word)
			}
			sequences.push(sequence)
		}
	}

	let groups = []
	for (let i = 0; i < compGroupInfo.length; i++) {
		const { seqId, seqLength } = compGroupInfo[i]
		groups.push(sequences.slice(seqId, seqId + seqLength))
	}

	const tableDataEl = document.getElementById('table-data')

	const tableEl = document.createElement('table')
	tableDataEl.append(tableEl)

	const tableHeaderEl = document.createElement('thead')
	tableEl.append(tableHeaderEl)
	tableHeaderEl.innerHTML = `
		<tr>
			<th>group id</th>
			<th>sequence id</th>
			<th>sequence type</th>
			<th>??</th>
			<th>??</th>
			<th>??</th>
			<th>??</th>
			<th>image set</th>
		</tr>`

	const tableBodyEl = document.createElement('tbody')
	tableEl.append(tableBodyEl)

	let seqId = 0
	for (let i = 0; i < groups.length; i++) {
		const group = groups[i]
		for (let j = 0; j < group.length; j++) {
			const sequence = group[j]

			const tableRowEl = document.createElement('tr')
			tableBodyEl.append(tableRowEl)

			if (j === 0) {
				tableRowEl.innerHTML += `<td rowspan=${group.length}>${i}</td>`
			}

			const seqType = sequence[0]
			const flag1 = sequence[1]
			const flag2 = sequence[2]
			const flag3 = sequence[3]
			const flag4 = (sequence.length === 5) ? '-' : sequence[4]
			const imageSet = (sequence.length === 5) ? sequence[4] : sequence[5]

			tableRowEl.innerHTML += `
				<td>${seqId}</td>
				<td>${seqType.toString(16).padStart(4, '0').toUpperCase()}</td>
				<td>${flag1 > 255 ? `(${255 - (flag1 & 0xff)})` : flag1}</td>
				<td>${flag2.toString(16).padStart(4, '0').toUpperCase()}</td>
				<td>${flag3}</td>
				<td>${flag4}</td>
				<td><a href="#image-set-${imageSet & 0xff}">${imageSet & 0xff}</a></td>`

			seqId++
		}
	}
}

const parseCompSeqOffsets = (data, hidden) => {
	const tableDataEl = document.getElementById('table-data')
	const tableContentEl = document.createElement('code')
	if (!hidden) {
		tableDataEl.append(tableContentEl)
	}

	let offsets = []
	for (let i = 0; i < data.byteLength; i += 4) {
		const wordEl = document.createElement('span')
		tableContentEl.append(wordEl)

		let offset = data.getUint32(i, LITTLE_ENDIAN)
		offsets.push(offset)
		wordEl.innerText = `${offset} `
	}

	compSeqOffsets = offsets.map(o => o * 2)
}

const parseCompGroupInfo = (data, hidden) => {
	const tableDataEl = document.getElementById('table-data')

	const tableEl = document.createElement('table')
	if (!hidden) tableDataEl.append(tableEl)

	const tableHeaderEl = document.createElement('thead')
	if (!hidden) tableEl.append(tableHeaderEl)
	tableHeaderEl.innerHTML = '<tr><th>sequence id</th><th>sequence length</th></tr>'

	const tableBodyEl = document.createElement('tbody')
	if (!hidden) tableEl.append(tableBodyEl)

	compGroupInfo = []
	for (let i = 0; i < data.byteLength; i += 4) {
		const tableRowEl = document.createElement('tr')
		if (!hidden) tableBodyEl.append(tableRowEl)

		const seqId = data.getUint16(i, LITTLE_ENDIAN)
		const seqLength = data.getUint16(i + 2, LITTLE_ENDIAN)

		if (seqId !== 0xffff) {
			compGroupInfo.push({seqId, seqLength})
			if (!hidden) tableRowEl.innerHTML = `<td>${seqId}</td><td>${seqLength}</td>`
		} else {
			if (!hidden) tableRowEl.innerHTML = `<td class="fade">FFFF</td><td class="fade">${seqLength}</td>`
		}
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
	console.log(imageDefs.length)
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
