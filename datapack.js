let clockfaceOffsets = []
let clockfaceLayerOffsets = []
let table9Offsets = []
let gfxNodeOffsets = []
let compOffsets = []
let compGroups = []

const parseDataPack = (data) => {
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

	for (let i = 0; i < 20; i++) {
		const tableOffset = data.byteOffset + tableOffsets[i]

		const containerEl = displayTableHeader(tableDataEl, `Table ${i}: ${TABLE_NAMES[i]}`, tableOffset, tableSizes[i])

		if (tableSizes[i] > 0) {
			const tableData = new DataView(data.buffer, tableOffset, tableSizes[i])

			switch(i) {
				case 0: parseGenericTable(containerEl, tableData); break;
				case 1: parseGenericTable(containerEl, tableData); break;
				case 2: parseParticleEmitterDefs(containerEl, tableData); break;
				case 3: parseClockfaceOffsets(containerEl, tableData); break;
				case 4: parseClockfaceLayerOffsets(containerEl, tableData); break;
				case 5: parseClockfaceDefs(containerEl, tableData); break;
				case 6: parseStringDefs(containerEl, tableData); break;
				case 7: parseStringOffsets(containerEl, tableData); break;
				case 8: parseTable8(containerEl, tableData); break;
				case 9: parseTable9(containerEl, tableData); break;
				case 10: parseItemDefs(containerEl, tableData); break;
				case 11: parseTamaDefs(containerEl, tableData); break;
				case 12: parseGenericTable(containerEl, tableData); break;
				case 13: parseGfxNodeOffsets(containerEl, tableData); break;
				case 14: parseGfxNodeDefs(containerEl, tableData); break;
				case 15:
					const compOffsetData = new DataView(data.buffer, data.byteOffset + tableOffsets[16], tableSizes[16])
					const compGroupData = new DataView(data.buffer, data.byteOffset + tableOffsets[18], tableSizes[18])
					parseCompOffsets(containerEl, compOffsetData, true)
					parseCompGroups(containerEl, compGroupData, true)
					parseCompDefs(containerEl, tableData)
					break
				case 16: parseCompOffsets(containerEl, tableData); break;
				case 17: parseGenericTable(containerEl, tableData); break;
				case 18: parseCompGroups(containerEl, tableData); break;
				case 19: parseGenericTable(containerEl, tableData); break;
			}
		}
	}
}

const displayTableHeader = (parentEl, tableName, tableOffset, tableSize) => {
	const tableHeaderEl = document.createElement('h3')
	parentEl.append(tableHeaderEl)
	tableHeaderEl.className = 'collapse'
	tableHeaderEl.innerText = tableName
	tableHeaderEl.addEventListener('click', () => tableHeaderEl.classList.toggle('collapse'))

	const tableContentsEl = document.createElement('div')
	parentEl.append(tableContentsEl)

	const tableInfoEl = document.createElement('code')
	tableContentsEl.append(tableInfoEl)
	if (tableSize > 0) {
		tableInfoEl.innerHTML = `offset: ${tableOffset} | size: ${tableSize} bytes | ${tableSize / 2} words`
	} else {
		tableInfoEl.innerHTML = `offset: ${tableOffset} | empty`
	}

	return tableContentsEl
}

const displayTable = (parentEl, headers, rows, rowIds) => {
	const tableEl = document.createElement('table')
	parentEl.append(tableEl)

	const tableHeaderEl = document.createElement('thead')
	tableEl.append(tableHeaderEl)

	const tableHeaderRowEl = document.createElement('tr')
	tableHeaderEl.append(tableHeaderRowEl)

	for (const headerContents of headers) {
		const tableHeaderCellEl = document.createElement('th')
		tableHeaderRowEl.append(tableHeaderCellEl)
		tableHeaderCellEl.innerHTML = headerContents
	}

	const tableBodyEl = document.createElement('tbody')
	tableEl.append(tableBodyEl)

	for (let i = 0; i < rows.length; i++) {
		const rowContents = rows[i]
		const tableBodyRowEl = document.createElement('tr')
		if (rowIds != null && rowIds[i] != null) {
			tableBodyRowEl.id = rowIds[i]
		}
		tableBodyEl.append(tableBodyRowEl)
		for (let j = 0; j < rowContents.length; j++) {
			const cell = rowContents[j]
			if (cell != null) {
				const tableBodyCellEl = headers[j] === 'offset' ? document.createElement('th') : document.createElement('td')
				if (!isNaN(rowContents[j].rowSpan)) {
					tableBodyCellEl.rowSpan = rowContents[j].rowSpan
					tableBodyCellEl.innerHTML = rowContents[j].contents
					tableBodyRowEl.append(tableBodyCellEl)
				} else if (rowContents[j] != null) {
					tableBodyCellEl.innerHTML = rowContents[j]
					tableBodyRowEl.append(tableBodyCellEl)
				}
			}
		}
	}
}

const parseGenericTable = (parentEl, data) => {
	const el = document.createElement('code')
	parentEl.append(el)
	for (let i = 0; i < data.byteLength; i += 2) {
		el.innerHTML += `${stringifyWord(data, i)} `
	}
}

const parseParticleEmitterDefs = (parentEl, data) => {
	const headers = ['offset', 'id', 'particle emitter data']
	let rows = []
	let rowIds = []

	const rowCount = data.byteLength / 66
	for (let i = 0; i < rowCount; i ++) {
		let rowContents = ''
		for (let j = 0; j < 66; j += 2) {
			const word = data.getUint16(i*66 + j, LITTLE_ENDIAN)
			rowContents += `${formatImageLink(word)} `
		}
		rows.push([i*66, i, rowContents])
		rowIds.push(`particle-emitter-${i}`)
	}

	displayTable(parentEl, headers, rows)
}

const parseClockfaceOffsets = (parentEl, data) => {
	clockfaceOffsets = []

	const el = document.createElement('code')
	parentEl.append(el)

	for (let i = 0; i < data.byteLength; i += 2) {
		const offset = data.getUint16(i, LITTLE_ENDIAN)
		clockfaceOffsets.push(offset * 2)
		el.innerText += `${offset} `
	}
}

const parseClockfaceLayerOffsets = (parentEl, data) => {
	const headers = ['clock offset', 'layer offsets']
	let rows = []

	clockfaceLayerOffsets = []
	let currentClockface = []
	for (let i = 0; i < data.byteLength; i += 2) {
		if (clockfaceOffsets.includes(i)) {
			if (currentClockface.length > 0) {
				clockfaceLayerOffsets.push(currentClockface)
			}
			currentClockface = []
		}
		let offset = data.getUint16(i, LITTLE_ENDIAN)
		currentClockface.push(offset)
	}
	clockfaceLayerOffsets.push(currentClockface)

	for (let i = 0; i < clockfaceLayerOffsets.length; i++) {
		rows.push([clockfaceOffsets[i] / 2, clockfaceLayerOffsets[i].join(' ')])
	}

	displayTable(parentEl, headers, rows)
}

const parseClockfaceDefs = (parentEl, data) => {
	for (let i = 0; i < clockfaceLayerOffsets.length; i++) {
		let clockface = clockfaceLayerOffsets[i]

		const headerEl = document.createElement('h4')
		parentEl.append(headerEl)
		headerEl.innerText = `Clock Face ${i+1}`

		const headers = ['offset', 'layer<br>type?', 'x', 'y', 'image set', '?']
		let rows = []

		for (let j = 0; j < clockface.length; j++) {
			const offset = clockface[j] * 2
			const layerType = stringifyWord(data, offset) || '-'
			const x = data.getInt16(offset + 2, LITTLE_ENDIAN) || '-'
			const y = data.getInt16(offset + 4, LITTLE_ENDIAN) || '-'
			const imageSet = data.getUint16(offset + 6, LITTLE_ENDIAN)
			let flag = 0
			if (j+1 < clockface.length && clockface[j+1] > clockface[j] + 4) {
				flag = stringifyWord(data, offset + 8)
			}

			rows.push([ offset, layerType, x, y, formatImageLink(imageSet), flag || '-' ])
		}

		displayTable(parentEl, headers, rows)
	}
}

const parseStringDefs = (parentEl, data) => {
	const headers = ['offset', 'id', '?', '?', '?', 'string']
	let rows = []

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
		const str = parseString(data, i + 8, strLength)

		rows.push([i, id, flag1, flag2, flag3, str])

		i += 10 + (strLength*2)
	}

	displayTable(parentEl, headers, rows)
}

const parseStringOffsets = (parentEl, data) => {
	const descriptionEl = document.createElement('code')
	parentEl.append(descriptionEl)
	descriptionEl.innerHTML = '<em>NOTE: Offsets measured in words. Multiply by 2 to get offset in bytes.</em>'

	const el = document.createElement('code')
	parentEl.append(el)

	for (let i = 0; i < data.byteLength; i += 2) {
		const offset = data.getUint16(i, LITTLE_ENDIAN)
		el.innerText += `${offset} `
	}
}

const parseTable8 = (parentEl, data) => {
	table9Offsets = []

	const descriptionEl = document.createElement('code')
	parentEl.append(descriptionEl)
	descriptionEl.innerHTML = '<em>NOTE: Offsets measured in words. Multiply by 2 to get offset in bytes.</em>'

	const el = document.createElement('code')
	parentEl.append(el)

	for (let i = 0; i < data.byteLength; i += 2) {
		const offset = data.getUint16(i, LITTLE_ENDIAN)
		el.innerText += `${offset} `
		table9Offsets.push(offset * 2)
	}
}

const parseTable9 = (parentEl, data) => {
	const el = document.createElement('code')
	parentEl.append(el)

	for (let i = 0; i < data.byteLength; i += 2) {
		el.innerHTML += `${stringifyWord(data, i)} `
		if (table9Offsets.includes(i + 2)) {
			el.innerHTML += '<br>'
		}
	}
}

const parseItemDefs = (parentEl, data) => {
	const headers = [
		'offset',
		'id',
		'type',
		'name',
		'image set',
		'image set<br><small>worn</small>',
		'image set<br><small>close-up</small>',
		'?',
		'?',
		'?',
		'?',
		'?',
		'unlocked<br>character'
	]
	let rows = []

	let i = 0
	while (i + 42 <= data.byteLength) {
		const id = data.getUint16(i, LITTLE_ENDIAN) & 0xff
		const typeIndex = data.getUint16(i + 2, LITTLE_ENDIAN)
		const type = ITEM_TYPES[typeIndex] || typeIndex
		const itemName = parseString(data, i + 4, 10)
		const imageSet = data.getUint16(i + 24, LITTLE_ENDIAN)
		const imageSetWorn = data.getUint16(i + 26, LITTLE_ENDIAN)
		const imageSetCloseUp = data.getUint16(i + 28, LITTLE_ENDIAN)
		const flag1 = data.getUint16(i + 30, LITTLE_ENDIAN) ? stringifyWord(data, i + 30) : '-'
		const flag2 = data.getUint16(i + 32, LITTLE_ENDIAN) || '-'
		const flag3 = data.getUint16(i + 34, LITTLE_ENDIAN) || '-'
		const flag4 = data.getUint16(i + 36, LITTLE_ENDIAN) || '-'
		const flag5 = data.getUint16(i + 38, LITTLE_ENDIAN) ? stringifyWord(data, i + 38) : '-'

		let unlockedCharacter = data.getUint16(i + 40, LITTLE_ENDIAN)
		const gameType = i === 0 ? GAME_TYPES[unlockedCharacter] : ''
		unlockedCharacter = unlockedCharacter > 0 ? `<a href="#tama-${unlockedCharacter}">${unlockedCharacter}</a>` : '-'

		rows.push([
			i,
			id,
			type,
			itemName,
			formatImageLink(imageSet),
			formatImageLink(imageSetWorn),
			formatImageLink(imageSetCloseUp),
			flag1,
			flag2,
			flag3,
			flag4,
			flag5,
			i === 0 ? gameType : `<a href="#tama-${id}">${id}</a>`
		])

		i += 42
	}

	displayTable(parentEl, headers, rows)
}

const parseTamaDefs = (parentEl, data) => {
	const headers = [
		'offset',
		'id',
		'type',
		'name',
		'memory<br>image',
		'icon',
		'id again?',
		'??',
		'pronoun',
		'statement<br><small>{ndesu}<small>',
		'question 1<br><small>{ndesuka}<small>',
		'question 2<br><small>{desuka}<small>',
		'??',
		'??',
		'original<br>card id',
		'??',
		'??',
		'??',
		'??',
		'gender'
	]
	let rows = []
	let rowIds = []

	let i = 0
	while (i + 96 <= data.byteLength) {
		const id = data.getUint16(i, LITTLE_ENDIAN) & 0xff
		const idString = stringifyWord(data, i)
		const type = data.getUint16(i + 2, LITTLE_ENDIAN)
		const tamaName = parseString(data, i + 4, 10)
		const memoryIndex = data.getUint16(i + 24, LITTLE_ENDIAN)
		const iconIndex = data.getUint16(i + 26, LITTLE_ENDIAN)
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

		rows.push([
			i,
			id,
			type,
			tamaName,
			formatImageLink(memoryIndex),
			formatImageLink(iconIndex),
			idAgain,
			flag4,
			pronoun,
			statement,
			question1,
			question2,
			other1,
			other2,
			originalId,
			other4,
			other5,
			other6,
			other7,
			gender
		])
		rowIds.push(`tama-${data.getUint16(i, LITTLE_ENDIAN) & 0xff}`)

		i += 96
	}

	displayTable(parentEl, headers, rows, rowIds)
}

const parseGfxNodeOffsets = (parentEl, data) => {
	gfxNodeOffsets = []

	const descriptionEl = document.createElement('code')
	parentEl.append(descriptionEl)
	descriptionEl.innerHTML = '<em>NOTE: Offsets measured in 32-bit chunks. Multiply by 4 to get offset in bytes.</em>'

	const el = document.createElement('code')
	parentEl.append(el)

	for (let i = 0; i < data.byteLength; i += 2) {
		const offset = data.getUint16(i, LITTLE_ENDIAN)
		el.innerText += `${offset} `
		gfxNodeOffsets.push(offset * 4)
	}
}

const parseGfxNodeDefs = (parentEl, data) => {
	const headers = ['offset', 'id', 'data']
	let rows = []
	let rowIds = []

	let sequences = []
	for (let i = 0; i < gfxNodeOffsets.length; i++) {
		let sequence = []
		const offset = gfxNodeOffsets[i]
		if (i + 1 < gfxNodeOffsets.length) {
			const nextOffset = gfxNodeOffsets[i + 1]
			const bytesInSequence = nextOffset - offset
			for (let j = 0; j < bytesInSequence; j += 2) {
				const word = data.getUint16(offset + j, LITTLE_ENDIAN)
				sequence.push(formatImageLink(word))
			}
			sequences.push(sequence)
		}
	}

	for (let i = 0; i < sequences.length; i++) {
		rows.push([ gfxNodeOffsets[i], i, sequences[i].join(' ') ])
		rowIds.push(`gfx-node-${i}`)
	}

	displayTable(parentEl, headers, rows, rowIds)
}

const parseCompDefs = (parentEl, data) => {
	const headers = [
		'offset',
		'group id',
		'group type',
		'?',
		'?',
		'?',
		'?',
		'image set'
	]
	let rows = []

	let comps = []
	for (let i = 0; i < compOffsets.length; i++) {
		const offset = compOffsets[i]
		let comp = [offset]
		if (i + 1 < compOffsets.length) {
			const nextOffset = compOffsets[i + 1]
			const bytesInComp = nextOffset - offset
			for (let j = 0; j < bytesInComp; j += 2) {
				const word = data.getUint16(offset + j, LITTLE_ENDIAN)
				comp.push(word)
			}
			comps.push(comp)
		}
	}

	let groups = []
	for (let i = 0; i < compGroups.length; i++) {
		const { groupId, groupSize } = compGroups[i]
		groups.push(comps.slice(groupId, groupId + groupSize))
	}

	let groupId = 0
	for (let i = 0; i < groups.length; i++) {
		const group = groups[i]
		for (let j = 0; j < group.length; j++) {
			const comp = group[j]

			const offset = comp[0]
			const groupType = comp[1].toString(16).padStart(4, '0').toUpperCase()
			const flag1 = comp[2].toString(16).padStart(4, '0').toUpperCase() // > 255 ? `(${255 - (comp[1] & 0xff)})` : comp[1]
			const flag2 = comp[3].toString(16).padStart(4, '0').toUpperCase()
			const flag3 = comp[4]
			const flag4 = (comp.length === 6) ? '-' : comp[5]
			const imageSet = (comp.length === 6) ? comp[5] : comp[6]

			rows.push([
				offset,
				j === 0 ? { rowSpan: group.length,  contents: i } : null,
				groupType,
				flag1,
				flag2,
				flag3,
				flag4,
				formatImageLink(imageSet)
			])

			groupId++
		}
	}

	displayTable(parentEl, headers, rows)
}

const parseCompOffsets = (parentEl, data, hidden) => {
	compOffsets = []

	const descriptionEl = document.createElement('code')
	if (!hidden) parentEl.append(descriptionEl)
	descriptionEl.innerHTML = '<em>NOTE: Offsets measured in words. Multiply by 2 to get offset in bytes. They\'re also 32-bit integers instead of the usual 16-bit words.</em>'

	const el = document.createElement('code')
	if (!hidden) parentEl.append(el)

	for (let i = 0; i < data.byteLength; i += 4) {
		const offset = data.getUint32(i, LITTLE_ENDIAN)
		el.innerText += `${offset} `
		compOffsets.push(offset * 2)
	}
}

const parseCompGroups = (parentEl, data, hidden) => {
	compGroups = []

	const headers = ['offset', 'group id', 'group size']
	let rows = []

	for (let i = 0; i < data.byteLength; i += 4) {
		const groupId = data.getUint16(i, LITTLE_ENDIAN)
		const groupSize = data.getUint16(i + 2, LITTLE_ENDIAN)
		if (groupId !== 0xffff) {
			compGroups.push({ groupId, groupSize })
			rows.push([ i, groupId, groupSize ])
		} else {
			rows.push([ i, `<span class="fade">FFFF</span>`, `<span class="fade">${groupSize}</span>` ])
		}
	}

	if (!hidden) displayTable(parentEl, headers, rows)
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

const formatImageLink = (id) => {
	if (id >> 8 === cardId + 128) {
		return `<a href="#image-set-${id & 0xff}">${id & 0xff}</a>`
	} else {
		return id.toString(16).padStart(4, '0').toUpperCase()
	}
}
