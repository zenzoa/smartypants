class AboutDialog {
	static isOpen() {
		return document.getElementById('about-dialog').classList.contains('open')
	}

	static open() {
		document.getElementById('about-dialog').classList.add('open')
		document.getElementById('about-ok-button').focus()
	}

	static close() {
		document.getElementById('about-dialog').classList.remove('open')
	}

	static setup() {
		document.getElementById('about-close-button')
			.addEventListener('click', AboutDialog.close)

		document.getElementById('about-ok-button')
			.addEventListener('click', AboutDialog.close)

		Tauri.app.getVersion().then(version => {
			document.getElementById('about-version').innerText = version
		})

		tauri_listen('show_about_dialog', AboutDialog.open)
	}
}
