const LITTLE_ENDIAN = true

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
	const fileCount = data.getUint16(2, LITTLE_ENDIAN)

	for (let i = 0; i < fileCount; i++) {
		const fileOffset = data.getUint32((i*16) + 8, LITTLE_ENDIAN)
		const fileSize = data.getUint32((i*16) + 16, LITTLE_ENDIAN)

		if (fileOffset > 0 && fileSize > 0) {
			parseFile(new DataView(data.buffer, data.byteOffset + fileOffset, fileSize))
		}
		break;
	}
}

const parseFile = (data) => {
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
		tableHeaderEl.innerText = TABLE_NAMES[i] || `Table ${i + 1}`
		tableHeaderEl.addEventListener('click', () => tableHeaderEl.classList.toggle('collapse'))
		tableDataEl.append(tableHeaderEl)
		if (tableSizes[i] > 0) {
			const tableData = new DataView(data.buffer, data.byteOffset + tableOffsets[i], tableSizes[i])
			if (i === 6) {
				parseDialogTable(tableData)
			} else if (i === 10) {
				parseItemTable(tableData)
			} else if (i === 11) {
				parseTamaTable(tableData)
			} else {
				parseTable(tableData)
				tableHeaderEl.className = 'collapse'
			}
		} else {
			const tableContentEl = document.createElement('code')
			tableContentEl.innerText = 'empty'
			tableDataEl.append(tableContentEl)
			tableHeaderEl.className = 'collapse'
		}
	}
}

const parseTable = (data) => {
	const tableDataEl = document.getElementById('table-data')
	const tableContentEl = document.createElement('code')

	for (let i = 0; i < data.byteLength; i += 2) {
		const value = data.getUint16(i, LITTLE_ENDIAN)
		const hexString = stringifyWord(data, i)
		const wordEl = document.createElement('span')
		wordEl.setAttribute('data-val', value)
		wordEl.setAttribute('data-txt', TEXT_ENCODING[value])
		wordEl.setAttribute('data-hex', hexString)
		wordEl.setAttribute('title', value)
		wordEl.className = 'hex word'
		wordEl.addEventListener('click', (event) => {
			if (event.shiftKey) {
				const siblingEls = event.target.parentNode.childNodes
				for (const siblingEl of siblingEls) {
					toggleWordView(siblingEl)
				}
			} else {
				toggleWordView(event.target)
			}
		})
		wordEl.innerText = hexString
		tableContentEl.append(wordEl)
	}

	tableDataEl.append(tableContentEl)
}

const toggleWordView = (wordEl) => {
	if (wordEl.className === 'hex word' && wordEl.getAttribute('data-txt') != 'undefined') {
		wordEl.className = 'val word'
		wordEl.innerText = wordEl.getAttribute('data-txt')
	} else {
		wordEl.className = 'hex word'
		wordEl.innerText = wordEl.getAttribute('data-hex')
	}
}

const parseDialogTable = (data) => {
	const tableDataEl = document.getElementById('table-data')
	const tableEl = document.createElement('table')
	tableEl.innerHTML = '<thead><tr><th>id</th><th>flags</th><th>string</th></tr></thead>'
	const tableBodyEl = document.createElement('tbody')
	let i = 0
	while (i + 10 <= data.byteLength) {
		const tableRowEl = document.createElement('tr')

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

		tableRowEl.innerHTML = `<td>${id}</td><td>${flag1} ${flag2} ${flag3}</td><td>${dialogStr}</td>`
		tableBodyEl.append(tableRowEl)

		i += 10 + (strLength*2)
	}

	tableEl.append(tableBodyEl)
	tableDataEl.append(tableEl)
}

const parseItemTable = (data) => {
	const tableDataEl = document.getElementById('table-data')
	const tableEl = document.createElement('table')
	tableEl.innerHTML = '<thead><tr><th>id</th><th>type</th><th>name</th><th>unparsed</th></tr></thead>'
	const tableBodyEl = document.createElement('tbody')

	let i = 0
	while (i + 42 <= data.byteLength) {
		const tableRowEl = document.createElement('tr')

		const id = stringifyWord(data, i)
		const typeIndex = data.getUint16(i + 2, LITTLE_ENDIAN)
		const type = ITEM_TYPES[typeIndex] || typeIndex
		const itemName = parseString(data, i + 4, 10)

		let otherData = []
		for (let j = 0; j < 9; j++) {
			otherData.push(stringifyWord(data, i + 24 + j*2))
		}

		tableRowEl.innerHTML = `<td>${id}</td><td>${type}</td><td>${itemName}</td><td>${otherData.join(' ')}</td>`
		tableBodyEl.append(tableRowEl)

		i += 42
	}

	tableEl.append(tableBodyEl)
	tableDataEl.append(tableEl)
}

const parseTamaTable = (data) => {
	const tableDataEl = document.getElementById('table-data')
	const tableEl = document.createElement('table')
	tableEl.innerHTML = '<thead><tr><th>id</th><th>type</th><th>name</th><th>flags</th><th>pronoun</th><th>statement<br><small>{ndesu}<small></th><th>question 1<br><small>{ndesuka}<small></th><th>question 2<br><small>{desuka}<small></th><th>unparsed</th></tr></thead>'
	const tableBodyEl = document.createElement('tbody')

	let i = 0
	while (i + 96 <= data.byteLength) {
		const tableRowEl = document.createElement('tr')

		const id = stringifyWord(data, i)
		const type = data.getUint16(i + 2, LITTLE_ENDIAN)
		const tamaName = parseString(data, i + 4, 10)
		const flag1 = stringifyWord(data, i + 24)
		const flag2 = stringifyWord(data, i + 26)
		const flag3 = stringifyWord(data, i + 28)
		const flag4 = stringifyWord(data, i + 30)
		const pronoun = parseString(data, i + 32, 6)
		const statement = parseString(data, i + 44, 6)
		const question1 = parseString(data, i + 56, 6)
		const question2 = parseString(data, i + 68, 6)

		let otherData = []
		for (let j = 0; j < 8; j++) {
			otherData.push(stringifyWord(data, i + 80 + j*2))
		}

		tableRowEl.innerHTML = `<td>${id}</td><td>${type}</td><td>${tamaName}</td><td>${flag1} ${flag2} ${flag3} ${flag4}</td><td>${pronoun}</td><td>${statement}</td><td>${question1}</td><td>${question2}</td><td>${otherData.join(' ')}</td>`
		tableBodyEl.append(tableRowEl)

		i += 96
	}

	tableEl.append(tableBodyEl)
	tableDataEl.append(tableEl)
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
	const byte1 = data.getUint8(offset).toString(16).padStart(2, '0').toUpperCase()
	const byte2 = data.getUint8(offset+1).toString(16).padStart(2, '0').toUpperCase()
	return `${byte1}${byte2}`
}
