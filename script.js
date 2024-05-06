const LITTLE_ENDIAN = true

let cardId = 0
let cardIdString = ''

window.onload = () => {
	const fileInput = document.getElementById('file-uploader')
	fileInput.addEventListener('change', handleFileUpload, false)
	fileInput.focus()
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
	if (parseASCII(data, 0, 14) === 'GP-SPIF-HEADER') {
		parseFirmware(data)
	} else {
		parseCard(data)
	}
}

const parseCardHeader = (data) => {
	const packageSum = data.getUint16(2, LITTLE_ENDIAN)
	const deviceIds = [
		data.getUint32(4, LITTLE_ENDIAN),
		data.getUint32(8, LITTLE_ENDIAN),
		data.getUint32(12, LITTLE_ENDIAN)
	]
	const headerString = parseASCII(data, 16, 32)
	const cardId = data.getUint16(50, LITTLE_ENDIAN)
	const date = {
		year: data.getUint16(54, LITTLE_ENDIAN),
		month: data.getUint16(56, LITTLE_ENDIAN),
		day: data.getUint16(58, LITTLE_ENDIAN)
	}
	let md5 = ''
	for (i = 0; i < 16; i++) {
		md5 += data.getUint8(64 + i).toString(16).padStart(2, '0')
	}
	console.log({ md5, packageSum, deviceIds, headerString, cardId, date })
}

const parseCard = (data) => {
	parseCardHeader(data)

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

const parseFirmware = (data) => {
	if (data.getUint16(0x6CA9AC) === 0x2800) {
		const tableDataEl = document.getElementById('table-data')
		tableDataEl.innerHTML = '<code>This seems to be the Niziu firmware, and it has some issues with offsets and does not decode properly. Sorry!</code>'
	} else {
		const dataPack = new DataView(data.buffer, 0x6CE000, 0x730000 - 0x6CE000)
		parseDataPack(dataPack)
		const spritePack = new DataView(data.buffer, 0x730000)
		parseSpritePack(spritePack)
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
				parseDataPack(packageData)
			} else if (i === 1) {
				parseSpritePack(packageData)
			}
		}
	}
}

parseASCII = (data, offset, length) => {
	let result = ''
	for (i = 0; i < length; i++) {
		nextChar = data.getUint8(offset + i)
		result += String.fromCharCode(nextChar)
	}
	return result
}
