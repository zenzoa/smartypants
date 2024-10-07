class EditItemDialog extends EditDialog {
	static open(i, item) {
		document.getElementById('edit-dialog-title').innerText = `Edit Item ${i}`

		const maxImageId = cardData.sprite_pack.image_defs.length - 1

		EditDialog.addDropdown('Type', 'item-type', item.item_type, [
			{ title: 'Meal', value: 'Meal' },
			{ title: 'Snack', value: 'Snack' },
			{ title: 'Toy', value: 'Toy' },
			{ title: 'Accessory/Head', value: 'AccessoryHead' },
			{ title: 'Accessory/Face', value: 'AccessoryFace' },
			{ title: 'Accessory/Body', value: 'AccessoryBody' },
			{ title: 'Accessory/Hand', value: 'AccessoryHand' },
			{ title: 'Room', value: 'Room' },
			{ title: 'Game', value: 'Game' }
		])
		EditDialog.addStrInput('Name', 'name', item.name.string, 9)
		EditDialog.addIntInput('Image ID', 'image-id', item.image_id ? item.image_id.entity_id : 0, 0, maxImageId)
		EditDialog.addIntInput('Worn Image ID', 'worn-image-id', item.worn_image_id ? item.worn_image_id.entity_id : 0, 0, maxImageId)
		EditDialog.addIntInput('Close Image ID', 'close-image-id', item.close_image_id ? item.close_image_id.entity_id : 0, 0, maxImageId)
		EditDialog.addIntInput('Animation ID ?', 'animation-id', item.animation_id ? item.animation_id.entity_id : 0, 0, U16_MAX)
		EditDialog.addIntInput('Price', 'price', item.price, 0, U16_MAX)
		EditDialog.addIntInput('Unkown 1', 'unknown1', item.unknown1, 0, U16_MAX)
		EditDialog.addIntInput('Unkown 2', 'unknown2', item.unknown2, 0, U16_MAX)
		EditDialog.addIntInput('Unkown 3', 'unknown3', item.unknown3, 0, U16_MAX)
		EditDialog.addDropdown('Game Type', 'game-type', item.game_type, [
			{ title: 'GuessingGame', value: 'GuessingGame' },
			{ title: 'TimingGame', value: 'TimingGame' },
			{ title: 'MemoryGame', value: 'MemoryGame' },
			{ title: 'DodgingGame', value: 'DodgingGame' },
			{ title: 'ShakingGame', value: 'ShakingGame' },
			{ title: 'SwipingGame', value: 'SwipingGame' }
		])
		EditDialog.addDropdown('Unlocked Character', 'unlocked-character', item.unlocked_character,
			[{ title: '-', value: 0 }].concat(
				cardData.data_pack.characters
					.filter(char => char.unknown1.entity_id > 0)
					.map(char => {
						return { title: char.name.string, value: char.id.entity_id }
					})
			)
		)

		EditItemDialog.updateItemType(item.item_type, i, item)
		document.getElementById('edit-item-type')
			.addEventListener('change', (event) => EditItemDialog.updateItemType(event.target.value, i, item))

		document.getElementById('edit-dialog-actions').append(
			button({ id: 'edit-cancel-button', className: 'text', title: 'Cancel', onclick: EditItemDialog.close }, 'Cancel'),
		)

		document.getElementById('edit-dialog-actions').append(
			button({ id: 'edit-ok-button', className: 'text', title: 'Ok', onclick: () => EditItemDialog.submit(i, item) }, 'Ok')
		)

		document.getElementById('edit-dialog').classList.add('open')
	}

	static updateItemType(item_type, i, item) {
		document.getElementById('label-game-type').classList.add('hidden')
		document.getElementById('label-image-id').classList.add('hidden')
		document.getElementById('label-worn-image-id').classList.add('hidden')
		document.getElementById('label-close-image-id').classList.add('hidden')
		document.getElementById('label-animation-id').classList.add('hidden')
		document.getElementById('label-unlocked-character').classList.add('hidden')

		if (item_type === 'Game') {
			document.getElementById('label-game-type').classList.remove('hidden')

		} else {
			document.getElementById('label-image-id').classList.remove('hidden')

			if (item_type === 'Meal' || item_type === 'Snack' || item_type === 'Toy') {
				document.getElementById('label-unlocked-character').classList.remove('hidden')
			}

			if (item_type === 'Toy') {
				document.getElementById('label-animation-id').classList.remove('hidden')
			} else if (item_type.startsWith('Accessory')) {
				document.getElementById('label-worn-image-id').classList.remove('hidden')
				document.getElementById('label-close-image-id').classList.remove('hidden')
			}
		}
	}

	static submit(i, item) {
		if (!document.getElementById('edit-name').classList.contains('invalid')) {
			const newItem = {
				id: item.id,
				item_type: document.getElementById('edit-item-type').value,
				name: { data: [], string: document.getElementById('edit-name').value },
				image_id: null,
				worn_image_id: null,
				close_image_id: null,
				animation_id: null,
				price: parseInt(document.getElementById('edit-price').value),
				unknown1: parseInt(document.getElementById('edit-unknown1').value),
				unknown2: parseInt(document.getElementById('edit-unknown2').value),
				unknown3: parseInt(document.getElementById('edit-unknown3').value),
				unlocked_character: null,
				game_type: null
			}

			if (newItem.item_type === 'Game') {
				newItem.game_type = document.getElementById('edit-game-type').value;

			} else {
				newItem.image_id = {
					card_id: newItem.id.card_id,
					entity_id: parseInt(document.getElementById('edit-image-id').value)
				}

				if (newItem.item_type === 'Meal' || newItem.item_type === 'Snack' || newItem.item_type === 'Toy') {
					let newUnlockedCharacter = parseInt(document.getElementById('edit-unlocked-character').value)
					if (newUnlockedCharacter > 0) newItem.unlocked_character = newUnlockedCharacter
				}

				if (newItem.item_type === 'Toy') {
					newItem.animation_id = {
						card_id: newItem.id.card_id,
						entity_id: parseInt(document.getElementById('edit-animation-id').value)
					}
				} else if (newItem.item_type.startsWith('Accessory')) {
					newItem.worn_image_id = {
						card_id: newItem.id.card_id,
						entity_id: parseInt(document.getElementById('edit-worn-image-id').value)
					}
					newItem.close_image_id = {
						card_id: newItem.id.card_id,
						entity_id: parseInt(document.getElementById('edit-close-image-id').value)
					}
				}
			}

			tauri_invoke('update_item', { index: i, newItem }).then(result => {
				if (result != null) {
					cardData.data_pack.items[i] = result
					sections.items = setupItems()
					viewItems()
				}
			})

			EditItemDialog.close()
		}
	}
}
