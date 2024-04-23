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
		tableHeaderEl.innerText = `Table ${i + 1}`
		tableDataEl.append(tableHeaderEl)
		if (tableSizes[i] > 0) {
			parseTable(new DataView(data.buffer, data.byteOffset + tableOffsets[i], tableSizes[i]))
		} else {
			const tableContentEl = document.createElement('code')
			tableContentEl.innerText = 'empty'
			tableDataEl.append(tableContentEl)
		}
	}
}

const parseTable = (data) => {
	const tableDataEl = document.getElementById('table-data')

	const tableContentEl = document.createElement('code')

	let contents = ''
	for (let i = 0; i < data.byteLength; i++) {
		const byte = data.getUint8(i).toString(16).toUpperCase().padStart(2, '0')
		contents = `${contents}${byte}`
		if (i % 2 === 1) {
			contents += ' '
		}
	}
	tableContentEl.innerText = contents

	tableDataEl.append(tableContentEl)
}
