const setupHeader = () => {
	if (cardData.bin_type === "Firmware") {
		return setupFirmwareHeader()
	} else {
		return setupCardHeader()
	}
}

const setupCardHeader = () => {
	const header = cardData.card_header
	return div([
		div({className: 'table-title'}, 'TamaSma Card'),
		table([
			tbody([
				tr([
					th('Device IDs'),
					td({ className: 'button-row' }, [
						div(
							{ style: 'flex-grow: 1; margin-right: 16px;' },
							`${header.device_ids.map(id => ` ${id.toString(16)}`)}`
						),
						button({
							title: 'Reset Device IDs', className: 'text',
							onclick: () => tauri_invoke('clear_device_ids')
						}, 'Reset')
					])
				]),
				// tr([th('Vendor ID'), td(header.vendor_id)]),
				// tr([th('Product ID'), td(header.product_id)]),
				tr([th('Card Type'), td(header.card_type)]),
				tr([
					th('Card ID'),
					td({ className: 'button-row' }, [
						div(
							{ style: 'flex-grow: 1; margin-right: 16px;' },
							header.card_id
						),
						// button({
						// 	title: 'Edit Card ID', className: 'icon',
						// 	onclick: () => EditCardIDDialog.open()
						// }, EDIT_ICON)
					])
				]),
				tr([
					th('Build Date'),
					td({ className: 'button-row' }, [
						div(
							{ style: 'flex-grow: 1; margin-right: 16px;' },
							`${header.year}-${header.month}-${header.day} revision ${header.revision}`
						),
						button({
							title: 'Edit Build Date', className: 'icon',
							onclick: () => EditBuildDateDialog.open()
						}, EDIT_ICON)
					])
				]),
				// tr([th('Sector Count'), td(header.sector_count)]),
				// tr([th('Checksum'), td(header.checksum)]),
				// tr([th('MD5'), td(header.md5.map(x => x.toString(16).padStart(2, 0)).reduce((prev, curr) => `${prev}${curr}`))])
			])
		])
	])
}

const setupFirmwareHeader = () => {
	return div([
		div({className: 'table-title'}, 'Tamagotchi Smart Firmware'),
		div({className: 'toggle-container'}, [
			button({
				className: cardData.use_patch_header ? 'toggle on' : 'toggle off',
				onclick: () => tauri_invoke('set_patch_header', { enable: true }).then(() => {
					cardData.use_patch_header = true
					sections.header = setupFirmwareHeader()
					viewHeader()
				})
			}, 'Use Patch Header (for updating firmware via SD Card)'),
			button({
				className: cardData.use_patch_header ? 'toggle off' : 'toggle on',
				onclick: () => tauri_invoke('set_patch_header', { enable: false }).then(() => {
					cardData.use_patch_header = false
					sections.header = setupFirmwareHeader()
					viewHeader()
				})
			}, 'No Patch Header (for updating firmware via EEPROM Programmer)')
		])
	])
}

const viewHeader = () => {
	selectSection('header')
	contents.append(sections.header)
}
