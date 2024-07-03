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
		tbody(characters.map(character => tr({id: `character-${character.id.entity_id}`}, [
			th(character.id.entity_id),
			td(character.character_type),
			td(character.name),
			td(displayImageWithLink(character.profile_image_id, 0)),
			td(displayImageWithLink(character.icon_image_id, 0)),
			td(linkToFrame(character.composition_id)),
			td(formatEntityId(character.unknown1)),
			td(character.pronoun),
			td(character.statement),
			td(character.question1),
			td(character.question2),
			td(character.unknown2),
			td(character.unknown3),
			td(formatEntityId(character.global_id)),
			td(character.unknown4),
			td(character.unknown5),
			td('#' + formatHexCode(character.unknown6)),
			td(character.unknown7),
			td(character.gender)
		])))
	])
}

const viewCharacters = () => {
	selectSection('view-characters-button')
	contents.append(sections.characters)
}
