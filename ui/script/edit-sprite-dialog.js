class EditSpriteDialog extends EditDialog {
	static open(i, imageDef, uniformOffsets) {
		document.getElementById('edit-dialog-title').innerText = `Edit Image ${i}`

		if (cardData.lock_colors) {
			EditDialog.addIntInput('First Palette ID', 'first-palette-index', imageDef.first_palette_index, 0, U16_MAX)
		}

		let firstOffsetX = imageDef.subimage_defs[0].offset_x
		let firstOffsetY = imageDef.subimage_defs[0].offset_y

		if (uniformOffsets == null) {
			uniformOffsets = true
			imageDef.subimage_defs.forEach(subimageDef => {
				if (subimageDef.offset_x !== firstOffsetX || subimageDef.offset_y !== firstOffsetY) {
					uniformOffsets = false
				}
			})
		}

		if (imageDef.subimage_defs.length > 1) {
			EditDialog.addCheckbox('Uniform Offsets', `uniform-offsets`, uniformOffsets)
				.addEventListener('click', () => {
					EditDialog.close()
					EditSpriteDialog.open(i, imageDef, !uniformOffsets)
				})
		}

		const halfWidth = Math.floor(imageDef.width / 2)
		const halfHeight = Math.floor(imageDef.height / 2)

		if (uniformOffsets) {
			const adjustedOffsetX = firstOffsetX + halfWidth
			const adjustedOffsetY = firstOffsetY + halfHeight
			EditDialog.addIntInput('X-Offset', `offset-x-0`, adjustedOffsetX, -128, 128)
			EditDialog.addIntInput('Y-Offset', `offset-y-0`, adjustedOffsetY, -128, 128)
		} else {
			imageDef.subimage_defs.forEach((subimageDef, subimageIndex) => {
				EditDialog.addSectionTitle(`Subimage ${subimageIndex}`)
				document.getElementById('edit-dialog-body').append(
					div([img({ src: convertFileSrc(`${timestamp}-${i}-${subimageIndex}`, 'getimage') })])
				)
				const adjustedOffsetX = subimageDef.offset_x + halfWidth
				const adjustedOffsetY = subimageDef.offset_y + halfHeight
				EditDialog.addIntInput('X-Offset', `offset-x-${subimageIndex}`, adjustedOffsetX, -128, 128)
				EditDialog.addIntInput('Y-Offset', `offset-y-${subimageIndex}`, adjustedOffsetY, -128, 128)
			})
		}


		document.getElementById('edit-dialog-actions').append(
			button({ id: 'edit-cancel-button', className: 'text', title: 'Cancel', onclick: EditSpriteDialog.close }, 'Cancel'),
		)

		document.getElementById('edit-dialog-actions').append(
			button({ id: 'edit-ok-button', className: 'text', title: 'Ok', onclick: () => EditSpriteDialog.submit(i, imageDef, uniformOffsets) }, 'Ok')
		)

		document.getElementById('edit-dialog').classList.add('open')
	}

	static submit(i, imageDef, uniformOffsets) {
		let invalid = false
		if (cardData.lock_colors && !document.getElementById('edit-first-palette-index').checkValidity()) {
			invalid = true
		}
		imageDef.subimage_defs.forEach((_, subimageIndex) => {
			let offsetXEl = document.getElementById(`edit-offset-x-${subimageIndex}`)
			let offsetYEl = document.getElementById(`edit-offset-y-${subimageIndex}`)
			if ((offsetXEl && !offsetXEl.checkValidity()) ||
				(offsetYEl && !offsetYEl.checkValidity())
			) {
				invalid = true
			}
		})

		if (!invalid) {
			const firstPaletteIndex = cardData.lock_colors ? parseInt(document.getElementById('edit-first-palette-index').value) : null

			let offsetsX = []
			let offsetsY = []

			const halfWidth = Math.floor(imageDef.width / 2)
			const halfHeight = Math.floor(imageDef.height / 2)

			if (uniformOffsets) {
				let offsetX = parseInt(document.getElementById('edit-offset-x-0').value) - halfWidth
				let offsetY = parseInt(document.getElementById('edit-offset-y-0').value) - halfHeight
				offsetsX = imageDef.subimage_defs.map(_ => offsetX)
				offsetsY = imageDef.subimage_defs.map(_ => offsetY)
			} else {
				offsetsX = imageDef.subimage_defs.map((_, subimageIndex) =>
					parseInt(document.getElementById(`edit-offset-x-${subimageIndex}`).value) - halfWidth
				)
				offsetsY = imageDef.subimage_defs.map((_, subimageIndex) =>
					parseInt(document.getElementById(`edit-offset-y-${subimageIndex}`).value) - halfHeight
				)
			}

			tauri_invoke('update_image_def', { index: i, offsetsX, offsetsY, firstPaletteIndex }).then(result => {
				if (result != null) {
					cardData.sprite_pack.image_defs[i] = result
					sections.frames = setupFrames()
					sections.sprites = setupSprites()
					viewSprites()
					const imageEl = document.getElementById(`image-${i}`)
					if (imageEl != null) {
						imageEl.scrollIntoView()
					}
				}
			})

			EditSpriteDialog.close()
		}
	}
}
