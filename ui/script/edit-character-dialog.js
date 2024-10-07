class EditCharacterDialog extends EditDialog {
	static open(i, character) {
		document.getElementById('edit-dialog-title').innerText = `Edit Character ${i}`

		const maxImageId = cardData.sprite_pack.image_defs.length - 1
		const maxCompositionId = cardData.data_pack.frame_groups.length - 1

		// EditDialog.addIntInput('Character Type', 'character-type', character.character_type, 0, U16_MAX)
		EditDialog.addStrInput('Name', 'name', character.name.string, 9)
		EditDialog.addIntInput('Profile Image ID', 'profile-image-id', character.profile_image_id ? character.profile_image_id.entity_id : 0, 0, maxImageId)
		EditDialog.addIntInput('Icon Image ID', 'icon-image-id', character.icon_image_id ? character.icon_image_id.entity_id : 0, 0, maxImageId)
		EditDialog.addIntInput('Frame Group ID', 'composition-id', character.composition_id ? character.composition_id.entity_id : 0, 0, maxCompositionId)
		// EditDialog.addIntInput('Unknown ID', 'unknown1', character.unknown1 ? character.unknown1.entity_id : 0, 0, U16_MAX)
		EditDialog.addStrInput('Pronoun', 'pronoun', character.pronoun.string, 5)
		EditDialog.addStrInput('Statement Ending', 'statement', character.statement.string, 5)
		EditDialog.addStrInput('Question Ending 1', 'question1', character.question1.string, 5)
		EditDialog.addStrInput('Question Ending 2', 'question2', character.question2.string, 5)
		EditDialog.addIntInput('Unkown 2', 'unknown2', character.unknown2, 0, U16_MAX)
		EditDialog.addIntInput('Unkown 3', 'unknown3', character.unknown3, 0, U16_MAX)
		// EditDialog.addIntInput('Global ID', 'global-id', character.global_id ? character.global_id.entity_id : 0, 0, U16_MAX)
		EditDialog.addIntInput('Unkown 4', 'unknown4', character.unknown4, 0, U16_MAX)
		EditDialog.addIntInput('Unkown 5', 'unknown5', character.unknown5, 0, U16_MAX)
		EditDialog.addIntInput('Unkown 6', 'unknown6', character.unknown6, 0, U16_MAX)
		EditDialog.addIntInput('Unkown 7', 'unknown7', character.unknown7, 0, U16_MAX)
		EditDialog.addDropdown('Gender', 'gender', character.gender, [
			{ title: 'Female', value: 'Female' },
			{ title: 'Male', value: 'Male' }
		])

		document.getElementById('edit-dialog-actions').append(
			button({ id: 'edit-cancel-button', className: 'text', title: 'Cancel', onclick: EditCharacterDialog.close }, 'Cancel'),
		)

		document.getElementById('edit-dialog-actions').append(
			button({ id: 'edit-ok-button', className: 'text', title: 'Ok', onclick: () => EditCharacterDialog.submit(i, character) }, 'Ok')
		)

		document.getElementById('edit-dialog').classList.add('open')
	}

	static submit(i, character) {
		if (!document.getElementById('edit-name').classList.contains('invalid') &&
			!document.getElementById('edit-pronoun').classList.contains('invalid') &&
			!document.getElementById('edit-statement').classList.contains('invalid') &&
			!document.getElementById('edit-question1').classList.contains('invalid') &&
			!document.getElementById('edit-question2').classList.contains('invalid')
		) {
			const newCharacter = {
				id: character.id,
				character_type: character.character_type,
				name: { data: [], string: document.getElementById('edit-name').value },
				profile_image_id: {
					card_id: character.id.card_id,
					entity_id: parseInt(document.getElementById('edit-profile-image-id').value)
				},
				icon_image_id: {
					card_id: character.id.card_id,
					entity_id: parseInt(document.getElementById('edit-icon-image-id').value)
				},
				composition_id: {
					card_id: character.id.card_id,
					entity_id: parseInt(document.getElementById('edit-composition-id').value)
				},
				unknown1: character.unknown1,
				pronoun: { data: [], string: document.getElementById('edit-pronoun').value },
				statement: { data: [], string: document.getElementById('edit-statement').value },
				question1: { data: [], string: document.getElementById('edit-question1').value },
				question2: { data: [], string: document.getElementById('edit-question2').value },
				unknown2: parseInt(document.getElementById('edit-unknown2').value),
				unknown3: parseInt(document.getElementById('edit-unknown3').value),
				global_id: character.global_id,
				unknown4: parseInt(document.getElementById('edit-unknown4').value),
				unknown5: parseInt(document.getElementById('edit-unknown5').value),
				unknown6: parseInt(document.getElementById('edit-unknown6').value),
				unknown7: parseInt(document.getElementById('edit-unknown7').value),
				gender: document.getElementById('edit-gender').value
			}

			tauri_invoke('update_character', { index: i, newCharacter }).then(result => {
				if (result != null) {
					cardData.data_pack.characters[i] = result
					sections.characters = setupCharacters()
					viewCharacters()
				}
			})

			EditCharacterDialog.close()
		}
	}
}
