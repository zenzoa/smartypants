const setupCharacters = () => {
	const characters = cardData.data_pack.characters
	return table([
		thead([tr([
			th('ID'),
			th('Type'),
			th('Name'),
			th('Profile Image ID'),
			th('Icon Image ID'),
			th('Frame ID'),
			th('Unknown ID'),
			th('Pronoun'),
			th('Statement Ending'),
			th('Question Ending 1'),
			th('Question Ending 2'),
			th('Unknown 2'),
			th('Unknown 3'),
			th('Global ID'),
			th('Unknown 4'),
			th('Unknown 5'),
			th('Unknown 6'),
			th('Unknown 7'),
			th('Gender')
		])]),
		tbody(characters.map((character, i) => renderCharacter(i, character)))
	])
}

const renderCharacter = (i, character) => {
	return tr({id: `character-${character.id.entity_id}`}, [
		th(character.id.entity_id),
		td(character.character_type),
		td([
			span(character.name.string),
			button({
				className: 'edit',
				onclick: editCharacterString.bind(this, i, 'Name', 'name')
			}, '✏️')
		]),
		td(displayImageWithLink(character.profile_image_id, 0)),
		td(displayImageWithLink(character.icon_image_id, 0)),
		td(linkToFrame(character.composition_id)),
		td(formatEntityId(character.unknown1)),
		td([
			span(character.pronoun.string),
			button({
				className: 'edit',
				onclick: editCharacterString.bind(this, i, 'Pronoun', 'pronoun')
			}, '✏️')
		]),
		td([
			span(character.statement.string),
			button({
				className: 'edit',
				onclick: editCharacterString.bind(this, i, 'Statement', 'statement')
			}, '✏️')
		]),
		td([
			span(character.question1.string),
			button({
				className: 'edit',
				onclick: editCharacterString.bind(this, i, 'Question 1', 'question1')
			}, '✏️')
		]),
		td([
			span(character.question2.string),
			button({
				className: 'edit',
				onclick: editCharacterString.bind(this, i, 'Question 2', 'question2')
			}, '✏️')
		]),
		td(character.unknown2),
		td(character.unknown3),
		td(formatEntityId(character.global_id)),
		td(character.unknown4),
		td(character.unknown5),
		td('#' + formatHexCode(character.unknown6)),
		td(character.unknown7),
		td(character.gender)
	])
}

const viewCharacters = () => {
	selectSection('characters')
	contents.append(sections.characters)
}

const editCharacterString = (i, title, key) => {
	const character = cardData.data_pack.characters[i]
	EditDialog.openStringEditor(
		`Edit Character ${i}: ${title}`,
		`${title}:`,
		character[key].string,
		(newValue) => {
			tauri_invoke('update_character', {
				index: i,
				key,
				newValue
			}).then(result => {
				if (result != null) {
					character[key] = result
					const characterEl = document.getElementById(`character-${i}`)
					if (characterEl != null) characterEl.replaceWith(renderCharacter(i, character))
				}
			})
			EditDialog.close()
		},
		8
	)
}
