const setupCardHeader = () => {
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

const viewCardHeader = () => {
	selectSection('view-header-button')
	contents.append(sections.header)
}

const setupFirmwareHeader = () => {
	return div('Tamagotchi Smart Firmware')
}

const viewFirmwareHeader = () => {
	selectSection('view-header-button')
	contents.append(sections.header)
}
