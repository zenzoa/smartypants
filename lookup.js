const TABLE_NAMES = {
	6: 'Table 7: Dialog',
	10: 'Table 11: Items',
	11: 'Table 12: Characters',
	19: 'Table 20: Card ID'
}

const ITEM_TYPES = {
	0: 'meal',
	1: 'snack',
	2: 'item',
	3: 'accessory (clothing?)',
	6: 'accessory (balloon?)',
	7: 'room',
	8: 'game'
}

// from Studded's string converter
const TEXT_ENCODING = {
	0: "█",
	1: " ",
	2: "0",
	3: "1",
	4: "2",
	5: "3",
	6: "4",
	7: "5",
	8: "6",
	9: "7",
	10: "8",
	11: "9",
	12: "+",
	13: "-",
	14: "↵",
	15: "あ",
	16: "い",
	17: "う",
	18: "え",
	19: "お",
	20: "か",
	21: "き",
	22: "く",
	23: "け",
	24: "こ",
	25: "さ",
	26: "し",
	27: "す",
	28: "せ",
	29: "そ",
	30: "た",
	31: "ち",
	32: "つ",
	33: "て",
	34: "と",
	35: "な",
	36: "に",
	37: "ぬ",
	38: "ね",
	39: "の",
	40: "は",
	41: "ひ",
	42: "ふ",
	43: "へ",
	44: "ほ",
	45: "ま",
	46: "み",
	47: "む",
	48: "め",
	49: "も",
	50: "や",
	51: "ゆ",
	52: "よ",
	53: "ら",
	54: "り",
	55: "る",
	56: "れ",
	57: "ろ",
	58: "わ",
	59: "を",
	60: "ん",
	61: "ぁ",
	62: "ぃ",
	63: "ぅ",
	64: "ぇ",
	65: "ぉ",
	66: "っ",
	67: "ゃ",
	68: "ゅ",
	69: "ょ",
	70: "が",
	71: "ぎ",
	72: "ぐ",
	73: "げ",
	74: "ご",
	75: "ざ",
	76: "じ",
	77: "ず",
	78: "ぜ",
	79: "ぞ",
	80: "だ",
	81: "ぢ",
	82: "づ",
	83: "で",
	84: "ど",
	85: "ば",
	86: "び",
	87: "ぶ",
	88: "べ",
	89: "ぼ",
	90: "ぱ",
	91: "ぴ",
	92: "ぷ",
	93: "ぺ",
	94: "ぽ",
	95: "ア",
	96: "イ",
	97: "ウ",
	98: "エ",
	99: "オ",
	100: "カ",
	101: "キ",
	102: "ク",
	103: "ケ",
	104: "コ",
	105: "サ",
	106: "シ",
	107: "ス",
	108: "セ",
	109: "ソ",
	110: "タ",
	111: "チ",
	112: "ツ",
	113: "テ",
	114: "ト",
	115: "ナ",
	116: "ニ",
	117: "ヌ",
	118: "ネ",
	119: "ノ",
	120: "ハ",
	121: "ヒ",
	122: "フ",
	123: "ヘ",
	124: "ホ",
	125: "マ",
	126: "ミ",
	127: "ム",
	128: "メ",
	129: "モ",
	130: "ヤ",
	131: "ユ",
	132: "ヨ",
	133: "ラ",
	134: "リ",
	135: "ル",
	136: "レ",
	137: "ロ",
	138: "ワ",
	139: "ヲ",
	140: "ン",
	141: "ァ",
	142: "ィ",
	143: "ゥ",
	144: "ェ",
	145: "ォ",
	146: "ッ",
	147: "ャ",
	148: "ュ",
	149: "ョ",
	150: "ガ",
	151: "ギ",
	152: "グ",
	153: "ゲ",
	154: "ゴ",
	155: "ザ",
	156: "ジ",
	157: "ズ",
	158: "ゼ",
	159: "ゾ",
	160: "ダ",
	161: "ヂ",
	162: "ヅ",
	163: "デ",
	164: "ド",
	165: "バ",
	166: "ビ",
	167: "ブ",
	168: "ベ",
	169: "ボ",
	170: "パ",
	171: "ピ",
	172: "プ",
	173: "ペ",
	174: "ポ",
	175: "ヴ",
	176: "ー",
	177: "～",
	178: "…",
	179: "、",
	180: "。",
	181: "(",
	182: ")",
	183: "「",
	184: "」",
	185: ".",
	186: "•",
	187: "!",
	188: "?",
	189: "&",
	190: "○",
	191: "✕",
	192: "♥",
	193: "☀",
	194: "★",
	195: "꩜",
	196: "♪",
	197: "╬",
	198: "⤴",
	199: "⤵",
	200: "→",
	201: "←",
	202: "$",
	203: "%",
	204: "A",
	205: "B",
	206: "C",
	207: "D",
	208: "E",
	209: "F",
	210: "G",
	211: "H",
	212: "I",
	213: "J",
	214: "K",
	215: "L",
	216: "M",
	217: "N",
	218: "O",
	219: "P",
	220: "Q",
	221: "R",
	222: "S",
	223: "T",
	224: "U",
	225: "V",
	226: "W",
	227: "X",
	228: "Y",
	229: "Z",
	230: "¡",
	231: "_",
	232: "†",
	233: "😄",
	234: "😣",
	235: "😤",
	236: "😑",
	237: "😵",
	238: "😢",
	239: "🐱",
	240: "⏱",
	241: "🎂",
	242: "🎁",
	243: "📱",
	244: "🏢",
	245: "💼",
	246: "🍙",
	247: "🍰",
	248: "✨",
	249: "🟥",
	61440: "\n",
	61441: "⬜\n", //new page
	61442: "{username}",
	61443: "{charname}",
	61444: "{ndesu}",
	61445: "{ndesuka}",
	61446: "{desuka}",
	61447: "{lastfood}",
	61448: "{pronoun}",
	61449: "{nickname}",
	61450: "{friend}"
  }