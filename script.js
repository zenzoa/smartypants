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
				parseDataPack(packageData)
			} else if (i === 1) {
				parseSpritePack(packageData)
			}
		}
	}
}
