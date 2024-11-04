class EditCharacterDialog extends EditDialog {
	static open(i, character) {
		document.getElementById('edit-dialog-title').innerText = `Edit Character ${i}`

		// EditDialog.addIntInput('Character Type', 'character-type', character.character_type, 0, U16_MAX)
		EditDialog.addStrInput('Name', 'name', character.name.string, 9)
		EditDialog.addIdInput('Profile Image ID', 'profile-image-id', character.profile_image_id)
		EditDialog.addIdInput('Icon Image ID', 'icon-image-id', character.icon_image_id)
		EditDialog.addIntInput('Frame Group ID', 'composition-id', character.composition_id == null ? -1 : character.composition_id.entity_id, -1, U16_MAX)
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
		if (EditDialog.checkStrValue('name') &&
			EditDialog.checkIdValue('profile-image-id') &&
			EditDialog.checkIdValue('icon-image-id') &&
			EditDialog.checkIntValue('composition-id') &&
			EditDialog.checkStrValue('pronoun') &&
			EditDialog.checkStrValue('statement') &&
			EditDialog.checkStrValue('question1') &&
			EditDialog.checkStrValue('question2') &&
			EditDialog.checkIntValue('unknown2') &&
			EditDialog.checkIntValue('unknown3') &&
			EditDialog.checkIntValue('unknown4') &&
			EditDialog.checkIntValue('unknown5') &&
			EditDialog.checkIntValue('unknown6') &&
			EditDialog.checkIntValue('unknown7')
		) {
			const compositionEntityId = EditDialog.getIntValue('composition-id')

			const newCharacter = {
				id: character.id,
				character_type: character.character_type,
				name: EditDialog.getStrValue('name'),
				profile_image_id: EditDialog.getIdValue('profile-image-id'),
				icon_image_id: EditDialog.getIdValue('icon-image-id'),
				composition_id: compositionEntityId < 0 ? null : {
					card_id: character.id.card_id,
					entity_id: compositionEntityId
				},
				unknown1: character.unknown1,
				pronoun: EditDialog.getStrValue('pronoun'),
				statement: EditDialog.getStrValue('statement'),
				question1: EditDialog.getStrValue('question1'),
				question2: EditDialog.getStrValue('question2'),
				unknown2: EditDialog.getIntValue('unknown2'),
				unknown3: EditDialog.getIntValue('unknown3'),
				global_id: character.global_id,
				unknown4: EditDialog.getIntValue('unknown4'),
				unknown5: EditDialog.getIntValue('unknown5'),
				unknown6: EditDialog.getIntValue('unknown6'),
				unknown7: EditDialog.getIntValue('unknown7'),
				gender: EditDialog.getDropdownValue('gender')
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
