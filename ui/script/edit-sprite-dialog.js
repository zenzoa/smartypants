class EditSpriteDialog extends EditDialog {
	static open(i, imageSet, uniformOffsets) {
		document.getElementById('edit-dialog-title').innerText = `Edit Image ${i}`

		let firstOffsetX = imageSet.subimages[0].offset_x
		let firstOffsetY = imageSet.subimages[0].offset_y

		if (uniformOffsets == null) {
			uniformOffsets = true
			imageSet.subimages.forEach(subimage => {
				if (subimage.offset_x !== firstOffsetX || subimage.offset_y !== firstOffsetY) {
					uniformOffsets = false
				}
			})
		}

		if (imageSet.subimages.length > 1) {
			EditDialog.addCheckbox('Uniform Offsets', 'uniform-offsets', uniformOffsets)
				.addEventListener('click', () => {
					EditDialog.close()
					EditSpriteDialog.open(i, imageSet, !uniformOffsets)
				})
		}

		const halfWidth = Math.floor(imageSet.width / 2)
		const halfHeight = Math.floor(imageSet.height / 2)

		if (uniformOffsets) {
			const adjustedOffsetX = firstOffsetX + halfWidth
			const adjustedOffsetY = firstOffsetY + halfHeight
			EditDialog.addIntInput('X-Offset', `offset-x-0`, adjustedOffsetX, -128, 128)
			EditDialog.addIntInput('Y-Offset', `offset-y-0`, adjustedOffsetY, -128, 128)
		} else {
			imageSet.subimages.forEach((subimage, subimageIndex) => {
				EditDialog.addSectionTitle(`Subimage ${subimageIndex}`)
				document.getElementById('edit-dialog-body').append(
					div([img({ src: convertFileSrc(`${timestamp}-${i}-${subimageIndex}`, 'getimage') })])
				)
				const adjustedOffsetX = subimage.offset_x + halfWidth
				const adjustedOffsetY = subimage.offset_y + halfHeight
				EditDialog.addIntInput('X-Offset', `offset-x-${subimageIndex}`, adjustedOffsetX, -128, 128)
				EditDialog.addIntInput('Y-Offset', `offset-y-${subimageIndex}`, adjustedOffsetY, -128, 128)
			})
		}


		document.getElementById('edit-dialog-actions').append(
			button({ id: 'edit-cancel-button', className: 'text', title: 'Cancel', onclick: EditSpriteDialog.close }, 'Cancel'),
		)

		document.getElementById('edit-dialog-actions').append(
			button({ id: 'edit-ok-button', className: 'text', title: 'Ok', onclick: () => EditSpriteDialog.submit(i, imageSet, uniformOffsets) }, 'Ok')
		)

		document.getElementById('edit-dialog').classList.add('open')
	}

	static submit(i, imageSet, uniformOffsets) {
		let invalid = false
		imageSet.subimages.forEach((_, subimageIndex) => {
			let offsetXEl = document.getElementById(`edit-offset-x-${subimageIndex}`)
			let offsetYEl = document.getElementById(`edit-offset-y-${subimageIndex}`)
			if ((offsetXEl && !offsetXEl.checkValidity()) ||
				(offsetYEl && !offsetYEl.checkValidity())
			) {
				invalid = true
			}
		})

		if (!invalid) {
			let offsetsX = []
			let offsetsY = []

			const halfWidth = Math.floor(imageSet.width / 2)
			const halfHeight = Math.floor(imageSet.height / 2)

			if (uniformOffsets) {
				let offsetX = parseInt(document.getElementById('edit-offset-x-0').value) - halfWidth
				let offsetY = parseInt(document.getElementById('edit-offset-y-0').value) - halfHeight
				offsetsX = imageSet.subimages.map(_ => offsetX)
				offsetsY = imageSet.subimages.map(_ => offsetY)
			} else {
				offsetsX = imageSet.subimages.map((_, subimageIndex) =>
					parseInt(document.getElementById(`edit-offset-x-${subimageIndex}`).value) - halfWidth
				)
				offsetsY = imageSet.subimages.map((_, subimageIndex) =>
					parseInt(document.getElementById(`edit-offset-y-${subimageIndex}`).value) - halfHeight
				)
			}

			tauri_invoke('update_image_set', { index: i, offsetsX, offsetsY }).then(result => {
				if (result != null) {
					cardData.image_sets[i] = result
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
