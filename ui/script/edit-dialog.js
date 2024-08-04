class EditDialog {
	static isOpen() {
		return document.getElementById('edit-dialog').classList.contains('open')
	}

	static open(title, inputLabel, value, onchange) {
		document.getElementById('edit-dialog').classList.add('open')
		document.getElementById('edit-dialog-title').innerText = title

		const inputEl = input({ id: 'edit-dialog-input', value })
		inputEl.addEventListener('keydown', (event) => {
			if (event.key === 'Enter') {
				onchange(inputEl.value)
			} else if (event.key === 'Escape') {
				EditDialog.close()
			} else {
				inputEl.classList.remove('invalid')
			}
		})


		EditDialog.callback = () => onchange(inputEl.value)

		document.getElementById('edit-dialog-body').append(
			label([
				span(inputLabel),
				inputEl,
				span({ className: 'validation-error' }, 'invalid')
			])
		)

		inputEl.focus()

		return inputEl
	}

	static openNumberEditor(title, inputLabel, value, fn, min, max) {
		const onchange = (newValue) => {
			const intValue = parseInt(newValue)
			if (intValue === parseFloat(newValue) && intValue >= min && intValue <= max) {
				fn(intValue)
			}
		}

		const inputEl = EditDialog.open(title, inputLabel, value, onchange)

		inputEl.setAttribute('type', 'number')
		inputEl.setAttribute('min', min)
		inputEl.setAttribute('max', max)
		inputEl.setAttribute('step', 1)
	}

	static openStringEditor(title, inputLabel, value, fn, maxLength) {
		const onchange = (newValue) => {
			const inputEl = document.getElementById('edit-dialog-input')
			tauri_invoke('validate_string', { string: newValue, maxLength }).then(result => {
				if (result[0]) {
					fn(result[1])
				} else {
					inputEl.classList.add('invalid')
				}
			})
		}

		const inputEl = EditDialog.open(title, inputLabel, value, onchange)

		inputEl.setAttribute('type', 'text')
		inputEl.setAttribute('valid', false)

		const previewEl = div({ className: 'string-preview-container' }, [
			'Preview:',
			div({ id: 'preview-small-text', className: 'string-preview' }),
			div({ id: 'preview-large-text', className: 'string-preview' })
		])

		document.getElementById('edit-dialog-body').append(previewEl)

		const updatePreview = () => {
			const inputEl = document.getElementById('edit-dialog-input')
			const smallPreviewEl = document.getElementById('preview-small-text')
			const largePreviewEl = document.getElementById('preview-large-text')
			tauri_invoke('decode_string_js', { string: inputEl.value }).then(result => {
				smallPreviewEl.replaceChildren()
				largePreviewEl.replaceChildren()
				result.forEach(i => {
					smallPreviewEl.append(div({ className: 'preview-letter' }, [displayImage('smallfont', i-1)]))
					largePreviewEl.append(div({ className: 'preview-letter' }, [displayImage('largefont', i-1)]))
				})
			})
		}

		inputEl.addEventListener('keyup', updatePreview)

		updatePreview()
	}

	static close() {
		document.getElementById('edit-dialog').classList.remove('open')
		document.getElementById('edit-dialog-title').replaceChildren()
		document.getElementById('edit-dialog-body').replaceChildren()
		EditDialog.callback = () => {}
	}

	static setup() {
		document.getElementById('edit-close-button')
			.addEventListener('click', EditDialog.close)

		document.getElementById('edit-cancel-button')
			.addEventListener('click', EditDialog.close)

		document.getElementById('edit-ok-button')
			.addEventListener('click', () => EditDialog.callback())
	}

	static callback() {}
}

// const ALLOWED_CHARACTERS = [
// 	" ", "0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "+", "-", "↵",
// 	"あ", "い", "う", "え", "お", "か", "き", "く", "け", "こ",
// 	"さ", "し", "す", "せ", "そ", "た", "ち", "つ", "て", "と",
// 	"な", "に", "ぬ", "ね", "の", "は", "ひ", "ふ", "へ", "ほ",
// 	"ま", "み", "む", "め", "も", "や", "ゆ", "よ",
// 	"ら", "り", "る", "れ", "ろ", "わ", "を", "ん",
// 	"ぁ", "ぃ", "ぅ", "ぇ", "ぉ", "っ", "ゃ", "ゅ", "ょ",
// 	"が", "ぎ", "ぐ", "げ", "ご", "ざ", "じ", "ず", "ぜ", "ぞ",
// 	"だ", "ぢ", "づ", "で", "ど", "ば", "び", "ぶ", "べ", "ぼ",
// 	"ぱ", "ぴ", "ぷ", "ぺ", "ぽ",
// 	"ア", "イ", "ウ", "エ", "オ", "カ", "キ", "ク", "ケ", "コ",
// 	"サ", "シ", "ス", "セ", "ソ", "タ", "チ", "ツ", "テ", "ト",
// 	"ナ", "ニ", "ヌ", "ネ", "ノ", "ハ", "ヒ", "フ", "ヘ", "ホ",
// 	"マ", "ミ", "ム", "メ", "モ", "ヤ", "ユ", "ヨ",
// 	"ラ", "リ", "ル", "レ", "ロ", "ワ", "ヲ", "ン",
// 	"ァ", "ィ", "ゥ", "ェ", "ォ", "ッ", "ャ", "ュ", "ョ",
// 	"ガ", "ギ", "グ", "ゲ", "ゴ", "ザ", "ジ", "ズ", "ゼ", "ゾ",
// 	"ダ", "ヂ", "ヅ", "デ", "ド", "バ", "ビ", "ブ", "ベ", "ボ",
// 	"パ", "ピ", "プ", "ペ", "ポ", "ヴ",
// 	"ー", "～", "…", "、", "。", "(", ")", "「", "」", ".", "•", "!", "?", "&",
// 	"○", "✕", "♥", "☼", "★", "🌀", "♪", "💢", "⤴", "⤵", "→", "←", "$", "%",
// 	"A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L", "M",
// 	"N", "O", "P", "Q", "R", "S", "T", "U", "V", "W", "X", "Y", "Z",
// 	"¡", "_", "†", "😄", "😣", "😤", "😑", "😵", "😢", "🐱",
// 	"⏱", "🎂", "🎁", "📱", "🏢", "💼", "🍙", "🍰", "✨", "🟥",
// 	"'"
// ]
