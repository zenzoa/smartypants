use std::collections::HashMap;

#[derive(serde::Serialize)]
pub struct DataView {
	pub data: Vec<u8>
}

impl DataView {
	pub fn new(data: &[u8]) -> DataView {
		DataView { data: data.to_owned() }
	}

	pub fn len(&self) -> usize {
		self.data.len()
	}

	pub fn chunk(&self, start: usize, size: usize) -> DataView {
		DataView { data: self.data[start..(start+size)].to_owned() }
	}

	pub fn get_u8(&self, i: usize) -> u8 {
		self.data[i]
	}

	pub fn get_u16(&self, i: usize) -> u16 {
		u16::from_le_bytes([self.data[i], self.data[i+1]])
	}

	pub fn get_i16(&self, i: usize) -> i16 {
		i16::from_le_bytes([self.data[i], self.data[i+1]])
	}

	pub fn get_u32(&self, i: usize) -> u32 {
		u32::from_le_bytes([self.data[i], self.data[i+1], self.data[i+2], self.data[i+3]])
	}

	pub fn get_bits(&self, i: usize, len: usize) -> Vec<u8> {
		let mut bits = Vec::new();
		for j in 0..len {
			let byte = self.data[i+j];
			for b in (0..8).rev() {
				let bit = (byte >> b) & 1;
				bits.push(bit);
			}
		}
		bits
	}

	pub fn get_encoded_string(&self, i: usize, len: usize) -> String {
		let mut value = String::new();
		for j in 0..len {
			let c = self.get_u16(i + j*2);
			if c > 0 {
				value.push_str(&get_encoded_char(c));
			}
		}
		value
	}

	pub fn find_bytes(&self, bytes: &[u8]) -> Option<usize> {
		for i in 0..self.data.len() {
			if self.data[i..].starts_with(bytes) {
				return Some(i);
			}
		}
		None
	}
}

pub fn words_to_bytes(words: &[u16]) -> Vec<u8> {
	let mut bytes: Vec<u8> = Vec::new();
	for word in words {
		for byte in u16::to_le_bytes(*word) {
			bytes.push(byte)
		}
	}
	bytes
}

pub fn get_encoded_char(word: u16) -> String {
	let map = get_encoding_map();
	match map.get(&word) {
		Some(s) => String::from(s),
		None => String::from("")
	}
}

pub fn get_char_encoding(ch: &str) -> Option<u16> {
	let map = get_encoding_map();
	for (key, val) in map.iter() {
		if val == ch {
			return Some(*key)
		}
	}
	println!("ERROR: Could not find encoding for '{}'", ch);
	None
}

pub fn encode_string(s: &str) -> Vec<u16> {
	let mut data: Vec<u16> = Vec::new();
	let mut var_name = String::new();
	for ch in s.chars() {
		match ch {
			'{' | '<' => {
				var_name.push(ch);
			},
			'}' | '>' => {
				var_name.push(ch);
				if let Some(word) = get_char_encoding(&var_name) {
					data.push(word);
				}
				var_name = String::new();
			},
			_ => {
				if var_name.is_empty() {
					if let Some(word) = get_char_encoding(&ch.to_string()) {
						data.push(word);
					}
				} else {
					var_name.push(ch);
				}
			}
		}
	}
	data
}

fn get_encoding_map() -> HashMap<u16, String> {
	HashMap::from([
		(0u16, String::from("█")),
		(1u16, String::from(" ")),
		(2u16, String::from("0")),
		(3u16, String::from("1")),
		(4u16, String::from("2")),
		(5u16, String::from("3")),
		(6u16, String::from("4")),
		(7u16, String::from("5")),
		(8u16, String::from("6")),
		(9u16, String::from("7")),
		(10u16, String::from("8")),
		(11u16, String::from("9")),
		(12u16, String::from("+")),
		(13u16, String::from("-")),
		(14u16, String::from("↵")),
		(15u16, String::from("あ")),
		(16u16, String::from("い")),
		(17u16, String::from("う")),
		(18u16, String::from("え")),
		(19u16, String::from("お")),
		(20u16, String::from("か")),
		(21u16, String::from("き")),
		(22u16, String::from("く")),
		(23u16, String::from("け")),
		(24u16, String::from("こ")),
		(25u16, String::from("さ")),
		(26u16, String::from("し")),
		(27u16, String::from("す")),
		(28u16, String::from("せ")),
		(29u16, String::from("そ")),
		(30u16, String::from("た")),
		(31u16, String::from("ち")),
		(32u16, String::from("つ")),
		(33u16, String::from("て")),
		(34u16, String::from("と")),
		(35u16, String::from("な")),
		(36u16, String::from("に")),
		(37u16, String::from("ぬ")),
		(38u16, String::from("ね")),
		(39u16, String::from("の")),
		(40u16, String::from("は")),
		(41u16, String::from("ひ")),
		(42u16, String::from("ふ")),
		(43u16, String::from("へ")),
		(44u16, String::from("ほ")),
		(45u16, String::from("ま")),
		(46u16, String::from("み")),
		(47u16, String::from("む")),
		(48u16, String::from("め")),
		(49u16, String::from("も")),
		(50u16, String::from("や")),
		(51u16, String::from("ゆ")),
		(52u16, String::from("よ")),
		(53u16, String::from("ら")),
		(54u16, String::from("り")),
		(55u16, String::from("る")),
		(56u16, String::from("れ")),
		(57u16, String::from("ろ")),
		(58u16, String::from("わ")),
		(59u16, String::from("を")),
		(60u16, String::from("ん")),
		(61u16, String::from("ぁ")),
		(62u16, String::from("ぃ")),
		(63u16, String::from("ぅ")),
		(64u16, String::from("ぇ")),
		(65u16, String::from("ぉ")),
		(66u16, String::from("っ")),
		(67u16, String::from("ゃ")),
		(68u16, String::from("ゅ")),
		(69u16, String::from("ょ")),
		(70u16, String::from("が")),
		(71u16, String::from("ぎ")),
		(72u16, String::from("ぐ")),
		(73u16, String::from("げ")),
		(74u16, String::from("ご")),
		(75u16, String::from("ざ")),
		(76u16, String::from("じ")),
		(77u16, String::from("ず")),
		(78u16, String::from("ぜ")),
		(79u16, String::from("ぞ")),
		(80u16, String::from("だ")),
		(81u16, String::from("ぢ")),
		(82u16, String::from("づ")),
		(83u16, String::from("で")),
		(84u16, String::from("ど")),
		(85u16, String::from("ば")),
		(86u16, String::from("び")),
		(87u16, String::from("ぶ")),
		(88u16, String::from("べ")),
		(89u16, String::from("ぼ")),
		(90u16, String::from("ぱ")),
		(91u16, String::from("ぴ")),
		(92u16, String::from("ぷ")),
		(93u16, String::from("ぺ")),
		(94u16, String::from("ぽ")),
		(95u16, String::from("ア")),
		(96u16, String::from("イ")),
		(97u16, String::from("ウ")),
		(98u16, String::from("エ")),
		(99u16, String::from("オ")),
		(100u16, String::from("カ")),
		(101u16, String::from("キ")),
		(102u16, String::from("ク")),
		(103u16, String::from("ケ")),
		(104u16, String::from("コ")),
		(105u16, String::from("サ")),
		(106u16, String::from("シ")),
		(107u16, String::from("ス")),
		(108u16, String::from("セ")),
		(109u16, String::from("ソ")),
		(110u16, String::from("タ")),
		(111u16, String::from("チ")),
		(112u16, String::from("ツ")),
		(113u16, String::from("テ")),
		(114u16, String::from("ト")),
		(115u16, String::from("ナ")),
		(116u16, String::from("ニ")),
		(117u16, String::from("ヌ")),
		(118u16, String::from("ネ")),
		(119u16, String::from("ノ")),
		(120u16, String::from("ハ")),
		(121u16, String::from("ヒ")),
		(122u16, String::from("フ")),
		(123u16, String::from("ヘ")),
		(124u16, String::from("ホ")),
		(125u16, String::from("マ")),
		(126u16, String::from("ミ")),
		(127u16, String::from("ム")),
		(128u16, String::from("メ")),
		(129u16, String::from("モ")),
		(130u16, String::from("ヤ")),
		(131u16, String::from("ユ")),
		(132u16, String::from("ヨ")),
		(133u16, String::from("ラ")),
		(134u16, String::from("リ")),
		(135u16, String::from("ル")),
		(136u16, String::from("レ")),
		(137u16, String::from("ロ")),
		(138u16, String::from("ワ")),
		(139u16, String::from("ヲ")),
		(140u16, String::from("ン")),
		(141u16, String::from("ァ")),
		(142u16, String::from("ィ")),
		(143u16, String::from("ゥ")),
		(144u16, String::from("ェ")),
		(145u16, String::from("ォ")),
		(146u16, String::from("ッ")),
		(147u16, String::from("ャ")),
		(148u16, String::from("ュ")),
		(149u16, String::from("ョ")),
		(150u16, String::from("ガ")),
		(151u16, String::from("ギ")),
		(152u16, String::from("グ")),
		(153u16, String::from("ゲ")),
		(154u16, String::from("ゴ")),
		(155u16, String::from("ザ")),
		(156u16, String::from("ジ")),
		(157u16, String::from("ズ")),
		(158u16, String::from("ゼ")),
		(159u16, String::from("ゾ")),
		(160u16, String::from("ダ")),
		(161u16, String::from("ヂ")),
		(162u16, String::from("ヅ")),
		(163u16, String::from("デ")),
		(164u16, String::from("ド")),
		(165u16, String::from("バ")),
		(166u16, String::from("ビ")),
		(167u16, String::from("ブ")),
		(168u16, String::from("ベ")),
		(169u16, String::from("ボ")),
		(170u16, String::from("パ")),
		(171u16, String::from("ピ")),
		(172u16, String::from("プ")),
		(173u16, String::from("ペ")),
		(174u16, String::from("ポ")),
		(175u16, String::from("ヴ")),
		(176u16, String::from("ー")),
		(177u16, String::from("～")),
		(178u16, String::from("…")),
		(179u16, String::from("、")),
		(180u16, String::from("。")),
		(181u16, String::from("(")),
		(182u16, String::from(")")),
		(183u16, String::from("「")),
		(184u16, String::from("」")),
		(185u16, String::from(".")),
		(186u16, String::from("•")),
		(187u16, String::from("!")),
		(188u16, String::from("?")),
		(189u16, String::from("&")),
		(190u16, String::from("○")),
		(191u16, String::from("✕")),
		(192u16, String::from("♥")),
		(193u16, String::from("☼")),
		(194u16, String::from("★")),
		(195u16, String::from("🌀")),
		(196u16, String::from("♪")),
		(197u16, String::from("💢")),
		(198u16, String::from("⤴")),
		(199u16, String::from("⤵")),
		(200u16, String::from("→")),
		(201u16, String::from("←")),
		(202u16, String::from("$")),
		(203u16, String::from("%")),
		(204u16, String::from("A")),
		(205u16, String::from("B")),
		(206u16, String::from("C")),
		(207u16, String::from("D")),
		(208u16, String::from("E")),
		(209u16, String::from("F")),
		(210u16, String::from("G")),
		(211u16, String::from("H")),
		(212u16, String::from("I")),
		(213u16, String::from("J")),
		(214u16, String::from("K")),
		(215u16, String::from("L")),
		(216u16, String::from("M")),
		(217u16, String::from("N")),
		(218u16, String::from("O")),
		(219u16, String::from("P")),
		(220u16, String::from("Q")),
		(221u16, String::from("R")),
		(222u16, String::from("S")),
		(223u16, String::from("T")),
		(224u16, String::from("U")),
		(225u16, String::from("V")),
		(226u16, String::from("W")),
		(227u16, String::from("X")),
		(228u16, String::from("Y")),
		(229u16, String::from("Z")),
		(230u16, String::from("¡")),
		(231u16, String::from("_")),
		(232u16, String::from("†")),
		(233u16, String::from("😄")),
		(234u16, String::from("😣")),
		(235u16, String::from("😤")),
		(236u16, String::from("😑")),
		(237u16, String::from("😵")),
		(238u16, String::from("😢")),
		(239u16, String::from("🐱")),
		(240u16, String::from("⏱")),
		(241u16, String::from("🎂")),
		(242u16, String::from("🎁")),
		(243u16, String::from("📱")),
		(244u16, String::from("🏢")),
		(245u16, String::from("💼")),
		(246u16, String::from("🍙")),
		(247u16, String::from("🍰")),
		(248u16, String::from("✨")),
		(249u16, String::from("🟥")),
		(250u16, String::from("'")),
		(61440u16, String::from("<br>")),
		(61441u16, String::from("<hr>")), //new page
		(61442u16, String::from("{username}")),
		(61443u16, String::from("{charname}")),
		(61444u16, String::from("{ndesu}")),
		(61445u16, String::from("{ndesuka}")),
		(61446u16, String::from("{desuka}")),
		(61447u16, String::from("{variable}")),
		(61448u16, String::from("{pronoun}")),
		(61449u16, String::from("{nickname}")),
		(61450u16, String::from("{friend}"))
	])
}
