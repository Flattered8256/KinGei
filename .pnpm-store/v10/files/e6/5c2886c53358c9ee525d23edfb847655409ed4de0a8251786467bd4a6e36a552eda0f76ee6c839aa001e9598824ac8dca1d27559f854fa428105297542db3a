const decoder = new TextDecoder();
const toUTF8String = (input, start = 0, end = input.length) => decoder.decode(input.slice(start, end));
const toHexString = (input, start = 0, end = input.length) => input.slice(start, end).reduce((memo, i) => memo + ("0" + i.toString(16)).slice(-2), "");
const readInt16LE = (input, offset = 0) => {
	const val = input[offset] + input[offset + 1] * 256;
	return val >= 2 ** 15 ? val - 2 ** 16 : val;
};
const readUInt16BE = (input, offset = 0) => input[offset] * 256 + input[offset + 1];
const readUInt16LE = (input, offset = 0) => input[offset] + input[offset + 1] * 256;
const readUInt24LE = (input, offset = 0) => input[offset] + input[offset + 1] * 256 + input[offset + 2] * 2 ** 16;
const readInt32LE = (input, offset = 0) => {
	const val = readUInt32LE(input, offset);
	return val >= 2 ** 31 ? val - 2 ** 32 : val;
};
const readUInt32BE = (input, offset = 0) => input[offset] * 2 ** 24 + input[offset + 1] * 2 ** 16 + input[offset + 2] * 256 + input[offset + 3];
const readUInt32LE = (input, offset = 0) => input[offset] + input[offset + 1] * 256 + input[offset + 2] * 2 ** 16 + input[offset + 3] * 2 ** 24;
const methods = {
	readUInt16BE,
	readUInt16LE,
	readUInt32BE,
	readUInt32LE
};
function readUInt(input, bits, offset, isBigEndian) {
	offset = offset || 0;
	const endian = isBigEndian ? "BE" : "LE";
	const methodName = "readUInt" + bits + endian;
	return methods[methodName](input, offset);
}
const encoder = new TextEncoder();
const toTagCode = (tag) => readUInt32BE(encoder.encode(tag));
const BOX_HEADER_SIZE = 8;
function findBox(input, boxName, startOffset = 0, endOffset = input.length) {
	const boxType = toTagCode(boxName);
	let offset = startOffset;
	while (offset + BOX_HEADER_SIZE <= endOffset) {
		const size = readUInt32BE(input, offset);
		if (size < BOX_HEADER_SIZE || offset + size > endOffset) return;
		if (readUInt32BE(input, offset + 4) === boxType) return {
			offset,
			size
		};
		offset += size;
	}
}
const CORE_HEADER_SIZE = 12;
const BMP = {
	validate: (input) => toUTF8String(input, 0, 2) === "BM",
	calculate: (input) => {
		if (readUInt32LE(input, 14) === CORE_HEADER_SIZE) return {
			height: readUInt16LE(input, 20),
			width: readUInt16LE(input, 18)
		};
		return {
			height: Math.abs(readInt32LE(input, 22)),
			width: readInt32LE(input, 18)
		};
	}
};
const TYPE_ICON = 1;
const SIZE_HEADER$1 = 6;
const SIZE_IMAGE_ENTRY = 16;
function getSizeFromOffset(input, offset) {
	const value = input[offset];
	return value === 0 ? 256 : value;
}
function getImageSize$1(input, imageIndex) {
	const offset = SIZE_HEADER$1 + imageIndex * SIZE_IMAGE_ENTRY;
	return {
		height: getSizeFromOffset(input, offset + 1),
		width: getSizeFromOffset(input, offset)
	};
}
const ICO = {
	validate(input) {
		const reserved = readUInt16LE(input, 0);
		const imageCount = readUInt16LE(input, 4);
		if (reserved !== 0 || imageCount === 0) return false;
		return readUInt16LE(input, 2) === TYPE_ICON;
	},
	calculate(input) {
		const nbImages = readUInt16LE(input, 4);
		if (SIZE_HEADER$1 + nbImages * SIZE_IMAGE_ENTRY > input.length) throw new TypeError("Invalid ICO");
		const imageSize = getImageSize$1(input, 0);
		if (nbImages === 1) return imageSize;
		const imgs = [imageSize];
		for (let imageIndex = 1; imageIndex < nbImages; imageIndex += 1) imgs.push(getImageSize$1(input, imageIndex));
		return {
			height: imageSize.height,
			images: imgs,
			width: imageSize.width
		};
	}
};
const TYPE_CURSOR = 2;
const CUR = {
	validate(input) {
		const reserved = readUInt16LE(input, 0);
		const imageCount = readUInt16LE(input, 4);
		if (reserved !== 0 || imageCount === 0) return false;
		return readUInt16LE(input, 2) === TYPE_CURSOR;
	},
	calculate: (input) => ICO.calculate(input)
};
const DDS = {
	validate: (input) => readUInt32LE(input, 0) === 542327876,
	calculate: (input) => ({
		height: readUInt32LE(input, 12),
		width: readUInt32LE(input, 16)
	})
};
const gifRegexp = /^GIF8[79]a/;
const GIF = {
	validate: (input) => gifRegexp.test(toUTF8String(input, 0, 6)),
	calculate: (input) => ({
		height: readUInt16LE(input, 8),
		width: readUInt16LE(input, 6)
	})
};
const FTYP = toTagCode("ftyp");
const ISPE = toTagCode("ispe");
const CLAP = toTagCode("clap");
function larger(largest, image) {
	return !largest || image.width * image.height > largest.width * largest.height ? image : largest;
}
function readLargestImageSize(input) {
	const metaBox = findBox(input, "meta");
	const iprpBox = metaBox && findBox(input, "iprp", metaBox.offset + 12, metaBox.offset + metaBox.size);
	const ipcoBox = iprpBox && findBox(input, "ipco", iprpBox.offset + 8, iprpBox.offset + iprpBox.size);
	if (!ipcoBox) throw new TypeError("Invalid HEIF, no ipco box found");
	let largest;
	let image;
	const end = ipcoBox.offset + ipcoBox.size;
	let offset = ipcoBox.offset + 8;
	while (offset + 8 <= end) {
		const size = readUInt32BE(input, offset);
		if (size < 8 || offset + size > end) throw new TypeError("Invalid HEIF, corrupt ipco box");
		const type = readUInt32BE(input, offset + 4);
		if (type === ISPE) {
			if (size < 20) throw new TypeError("Invalid HEIF, corrupt ispe box");
			if (image) largest = larger(largest, image);
			image = {
				width: readUInt32BE(input, offset + 12),
				height: readUInt32BE(input, offset + 16)
			};
		}
		if (type === CLAP && size >= 24 && image) {
			const widthD = readUInt32BE(input, offset + 12);
			const heightD = readUInt32BE(input, offset + 20);
			if (widthD > 0 && heightD > 0) {
				image.width = Math.round(readUInt32BE(input, offset + 8) / widthD);
				image.height = Math.round(readUInt32BE(input, offset + 16) / heightD);
			}
		}
		offset += size;
	}
	if (!image) throw new TypeError("Invalid HEIF, no ispe box found");
	return larger(largest, image);
}
const AVIF_BRANDS = new Set(["avif", "avis"].map((brand) => toTagCode(brand)));
function detectHeifType(input) {
	const ftypSize = readUInt32BE(input, 0);
	if (readUInt32BE(input, 4) !== FTYP || ftypSize < 12 || ftypSize > input.length) return;
	if (AVIF_BRANDS.has(readUInt32BE(input, 8))) return "avif";
	const majorBrand = toUTF8String(input, 8, 12);
	if ([
		"mif1",
		"msf1",
		"miaf"
	].includes(majorBrand)) {
		for (let offset = 16; offset + 4 <= ftypSize; offset += 4) if (AVIF_BRANDS.has(readUInt32BE(input, offset))) return "avif";
	}
	return [
		"heic",
		"heix",
		"hevc",
		"hevx",
		"mif1",
		"msf1"
	].includes(majorBrand) ? "heic" : void 0;
}
const HEIC = {
	validate: (input) => detectHeifType(input) === "heic",
	calculate: (input) => readLargestImageSize(input)
};
const SIZE_HEADER = 8;
const FILE_LENGTH_OFFSET = 4;
const ENTRY_LENGTH_OFFSET = 4;
const ICON_TYPE_SIZE = {
	ICON: 32,
	"ICN#": 32,
	"icm#": 16,
	icm4: 16,
	icm8: 16,
	"ics#": 16,
	ics4: 16,
	ics8: 16,
	is32: 16,
	s8mk: 16,
	icp4: 16,
	ic04: 16,
	icsb: 18,
	sb24: 24,
	icl4: 32,
	icl8: 32,
	il32: 32,
	l8mk: 32,
	icp5: 32,
	ic05: 32,
	ic11: 32,
	icsB: 36,
	ich4: 48,
	ich8: 48,
	ih32: 48,
	h8mk: 48,
	SB24: 48,
	icp6: 64,
	ic12: 64,
	it32: 128,
	t8mk: 128,
	ic07: 128,
	ic08: 256,
	ic13: 256,
	ic09: 512,
	ic14: 512,
	ic10: 1024
};
const ICON_TYPES = new Map(Object.keys(ICON_TYPE_SIZE).map((type) => [toTagCode(type), type]));
function readImageHeader(input, imageOffset) {
	const imageLengthOffset = imageOffset + ENTRY_LENGTH_OFFSET;
	if (imageLengthOffset + 4 > input.length) throw new TypeError("Invalid ICNS");
	const entryLength = readUInt32BE(input, imageLengthOffset);
	if (entryLength < 8) throw new TypeError("Invalid ICNS");
	return [ICON_TYPES.get(readUInt32BE(input, imageOffset)), entryLength];
}
function getImageSize(type) {
	const size = ICON_TYPE_SIZE[type];
	return {
		width: size,
		height: size,
		type
	};
}
const ICNS = {
	validate: (input) => toUTF8String(input, 0, 4) === "icns",
	calculate(input) {
		const inputLength = input.length;
		const fileLength = readUInt32BE(input, FILE_LENGTH_OFFSET);
		let imageOffset = SIZE_HEADER;
		const images = [];
		const seenTypes = /* @__PURE__ */ new Set();
		while (imageOffset < fileLength && imageOffset < inputLength) {
			const [type, entryLength] = readImageHeader(input, imageOffset);
			imageOffset += entryLength;
			if (type !== void 0 && !seenTypes.has(type)) {
				seenTypes.add(type);
				images.push(getImageSize(type));
			}
		}
		if (images.length === 0) throw new TypeError("Invalid ICNS, no icons found");
		if (images.length === 1) return images[0];
		return {
			height: images[0].height,
			images,
			width: images[0].width
		};
	}
};
const J2C = {
	validate: (input) => readUInt32BE(input, 0) === 4283432785,
	calculate: (input) => ({
		height: readUInt32BE(input, 12) - readUInt32BE(input, 20),
		width: readUInt32BE(input, 8) - readUInt32BE(input, 16)
	})
};
const JP2 = {
	validate(input) {
		if (toUTF8String(input, 4, 8) !== "jP  ") return false;
		return findBox(input, "ftyp") !== void 0;
	},
	calculate(input) {
		const jp2hBox = findBox(input, "jp2h");
		const ihdrBox = jp2hBox && findBox(input, "ihdr", jp2hBox.offset + 8, jp2hBox.offset + jp2hBox.size);
		if (!ihdrBox || ihdrBox.size < 16) throw new TypeError("Invalid JPEG 2000, no ihdr box found");
		return {
			height: readUInt32BE(input, ihdrBox.offset + 8),
			width: readUInt32BE(input, ihdrBox.offset + 12)
		};
	}
};
const EXIF_MARKER = toTagCode("Exif");
const APP1_DATA_SIZE_BYTES = 2;
const EXIF_HEADER_BYTES = 6;
const BIG_ENDIAN_BYTE_ALIGN = 19789;
const LITTLE_ENDIAN_BYTE_ALIGN = 18761;
const TIFF_MAGIC = 42;
const TIFF_HEADER_BYTES = 8;
const IDF_ENTRY_BYTES = 12;
const NUM_DIRECTORY_ENTRIES_BYTES = 2;
function isEXIF(input) {
	return readUInt32BE(input, 2) === EXIF_MARKER;
}
function extractSize(input, index) {
	return {
		height: readUInt16BE(input, index),
		width: readUInt16BE(input, index + 2)
	};
}
function extractOrientation(exifBlock, isBigEndian) {
	if (readUInt(exifBlock, 16, 8, isBigEndian) !== TIFF_MAGIC) return;
	const idfOffset = readUInt(exifBlock, 32, 10, isBigEndian);
	const offset = EXIF_HEADER_BYTES + idfOffset;
	if (!(idfOffset >= TIFF_HEADER_BYTES && offset + NUM_DIRECTORY_ENTRIES_BYTES <= exifBlock.length)) return;
	const idfDirectoryEntries = readUInt(exifBlock, 16, offset, isBigEndian);
	for (let directoryEntryNumber = 0; directoryEntryNumber < idfDirectoryEntries; directoryEntryNumber++) {
		const start = offset + NUM_DIRECTORY_ENTRIES_BYTES + directoryEntryNumber * IDF_ENTRY_BYTES;
		const end = start + IDF_ENTRY_BYTES;
		if (start > exifBlock.length) return;
		const block = exifBlock.slice(start, end);
		if (readUInt(block, 16, 0, isBigEndian) === 274) {
			if (readUInt(block, 16, 2, isBigEndian) !== 3) return;
			if (readUInt(block, 32, 4, isBigEndian) !== 1) return;
			return readUInt(block, 16, 8, isBigEndian);
		}
	}
}
function validateExifBlock(input, index) {
	const exifBlock = input.slice(APP1_DATA_SIZE_BYTES, index);
	const byteAlign = readUInt16BE(exifBlock, EXIF_HEADER_BYTES);
	const isBigEndian = byteAlign === BIG_ENDIAN_BYTE_ALIGN;
	if (isBigEndian || byteAlign === LITTLE_ENDIAN_BYTE_ALIGN) return extractOrientation(exifBlock, isBigEndian);
}
function validateInput(input, index) {
	if (index > input.length) throw new TypeError("Corrupt JPG, exceeded buffer limits");
}
function findMarker(input, index) {
	let marker = input.indexOf(255, index);
	while (marker !== -1) {
		while (input[marker + 1] === 255) marker++;
		if (input[marker + 1] !== 0) return marker;
		marker = input.indexOf(255, marker + 2);
	}
	throw new TypeError("Invalid JPG, marker table corrupted");
}
const JPG = {
	validate: (input) => toHexString(input, 0, 2) === "ffd8",
	calculate(input) {
		input = input.subarray(2);
		let orientation;
		let next;
		while (input.length > 0) {
			const marker = findMarker(input, 0);
			next = input[marker + 1];
			if (next >= 192 && next <= 195 || next >= 201 && next <= 203) {
				const size = extractSize(input, marker + 5);
				if (!orientation) return size;
				return {
					height: size.height,
					orientation,
					width: size.width
				};
			}
			if (next === 218) break;
			input = input.subarray(marker + 2);
			const i = readUInt16BE(input, 0);
			validateInput(input, i);
			if (isEXIF(input)) orientation = validateExifBlock(input, i);
			input = input.subarray(i);
		}
		throw new TypeError("Invalid JPG, no size found");
	}
};
function createBitReader(input) {
	let bitOffset = 16;
	return (length) => {
		let result = 0;
		for (let i = 0; i < length; i++, bitOffset++) {
			const byte = input[bitOffset >> 3];
			if (byte === void 0) throw new TypeError("Invalid JXL, truncated codestream");
			result += (byte >> (bitOffset & 7) & 1) * 2 ** i;
		}
		return result;
	};
}
const RATIOS = [
	[1, 1],
	[12, 10],
	[4, 3],
	[3, 2],
	[16, 9],
	[5, 4],
	[2, 1]
];
function calculateCodestream(input) {
	const readBits = createBitReader(input);
	const readSize = (isSmall) => isSmall ? 8 * (1 + readBits(5)) : 1 + readBits([
		9,
		13,
		18,
		30
	][readBits(2)]);
	const isSmall = readBits(1) === 1;
	const height = readSize(isSmall);
	const ratio = readBits(3);
	if (ratio === 0) return {
		height,
		width: readSize(isSmall)
	};
	const [numerator, denominator] = RATIOS[ratio - 1];
	return {
		height,
		width: Math.floor(height * numerator / denominator)
	};
}
const CODESTREAM_PREFIX_BYTES = 32;
const JXLC = toTagCode("jxlc");
const JXLP = toTagCode("jxlp");
function extractCodestream(input) {
	const parts = [];
	let length = 0;
	let offset = 0;
	while (offset + 8 <= input.length && length < CODESTREAM_PREFIX_BYTES) {
		let size = readUInt32BE(input, offset);
		const type = readUInt32BE(input, offset + 4);
		let headerSize = 8;
		if (size === 1) {
			if (offset + 16 > input.length) break;
			size = readUInt32BE(input, offset + 8) * 2 ** 32 + readUInt32BE(input, offset + 12);
			headerSize = 16;
		} else if (size === 0) size = input.length - offset;
		if (type === JXLC || type === JXLP) {
			const start = offset + headerSize + (type === JXLP ? 4 : 0);
			if (size < start - offset) throw new TypeError(`Invalid JXL, corrupt ${type === JXLP ? "jxlp" : "jxlc"} box`);
			const end = Math.min(offset + size, input.length, start + CODESTREAM_PREFIX_BYTES - length);
			if (end > start) {
				parts.push(input.subarray(start, end));
				length += end - start;
			}
			if (type === JXLC) break;
		}
		if (size < headerSize) throw new TypeError("Invalid JXL, corrupt box");
		offset += size;
	}
	const codestream = new Uint8Array(length);
	let position = 0;
	for (const part of parts) {
		codestream.set(part, position);
		position += part.length;
	}
	return codestream;
}
const isCodestream = (input) => toHexString(input, 0, 2) === "ff0a";
const JXL = {
	validate: (input) => {
		if (isCodestream(input)) return true;
		if (toUTF8String(input, 4, 8) !== "JXL ") return false;
		const ftypBox = findBox(input, "ftyp");
		return ftypBox !== void 0 && toUTF8String(input, ftypBox.offset + 8, ftypBox.offset + 12) === "jxl ";
	},
	calculate: (input) => {
		const codestream = isCodestream(input) ? input : extractCodestream(input);
		if (!isCodestream(codestream)) throw new TypeError("Invalid JXL, no codestream found");
		return calculateCodestream(codestream);
	}
};
const LITTLE_ENDIAN = 67305985;
const BIG_ENDIAN = 16909060;
const KTX = {
	validate: (input) => {
		const signature = toUTF8String(input, 1, 7);
		return signature === "KTX 11" || signature === "KTX 20";
	},
	calculate: (input) => {
		if (input[5] === 50) return {
			height: readUInt32LE(input, 24),
			width: readUInt32LE(input, 20)
		};
		const endianness = readUInt32LE(input, 12);
		if (endianness !== LITTLE_ENDIAN && endianness !== BIG_ENDIAN) throw new TypeError("Invalid KTX, unknown endianness");
		const isBigEndian = endianness === BIG_ENDIAN;
		return {
			height: readUInt(input, 32, 40, isBigEndian),
			width: readUInt(input, 32, 36, isBigEndian)
		};
	}
};
const pngSignature = "PNG\r\n\n";
const pngImageHeaderChunkName = "IHDR";
const pngFriedChunkName = "CgBI";
const PNG = {
	validate(input) {
		if (pngSignature === toUTF8String(input, 1, 8)) {
			let chunkName = toUTF8String(input, 12, 16);
			if (chunkName === pngFriedChunkName) chunkName = toUTF8String(input, 28, 32);
			if (chunkName !== pngImageHeaderChunkName) throw new TypeError("Invalid PNG");
			return true;
		}
		return false;
	},
	calculate(input) {
		if (toUTF8String(input, 12, 16) === pngFriedChunkName) return {
			height: readUInt32BE(input, 36),
			width: readUInt32BE(input, 32)
		};
		return {
			height: readUInt32BE(input, 20),
			width: readUInt32BE(input, 16)
		};
	}
};
const PNMTypes = {
	P1: "pbm/ascii",
	P2: "pgm/ascii",
	P3: "ppm/ascii",
	P4: "pbm",
	P5: "pgm",
	P6: "ppm",
	P7: "pam",
	PF: "pfm",
	Pf: "pfm"
};
const isLineBreak = (byte) => byte === 10 || byte === 13;
const isWhitespace = (byte) => byte === 32 || byte >= 9 && byte <= 13;
const toInteger = (value = "") => /^\d+$/.test(value) ? Number(value) : NaN;
const PAMLine = /^\s*(width|height|endhdr)(?!\S)\s*(\S*)/i;
const handlers = {
	default: (input) => {
		const [width, height] = readHeader(input, "tokens");
		return {
			height: toInteger(height),
			width: toInteger(width)
		};
	},
	pam: (input) => {
		const size = {};
		for (const line of readHeader(input, "lines")) {
			const [, tag = "", value] = PAMLine.exec(line) || [];
			const key = tag.toLowerCase();
			if (key === "width" || key === "height") size[key] = toInteger(value);
			if (key === "endhdr" || "width" in size && "height" in size) break;
		}
		return {
			height: size.height,
			width: size.width
		};
	}
};
function* readHeader(input, unit) {
	const isSeparator = unit === "lines" ? isLineBreak : isWhitespace;
	const values = unit === "lines" ? /[^\n\r]+/g : /[^\t-\r ]+/g;
	let start = 2;
	while (start < input.length) {
		let end = start;
		let cut = start;
		let inComment = false;
		while (end < input.length && (end - start < 1024 || cut === start)) {
			const byte = input[end++];
			inComment = byte === 35 || inComment && !isLineBreak(byte);
			if (!inComment && isSeparator(byte)) cut = end;
		}
		if (end === input.length) cut = end;
		yield* toUTF8String(input, start, cut).replace(/#[^\n\r]*/g, "\n").match(values) || [];
		start = cut;
	}
}
const PNM = {
	validate: (input) => toUTF8String(input, 0, 2) in PNMTypes,
	calculate(input) {
		const signature = toUTF8String(input, 0, 2);
		const type = PNMTypes[signature];
		return (handlers[type] || handlers.default)(input);
	}
};
const PSD = {
	validate: (input) => toUTF8String(input, 0, 4) === "8BPS",
	calculate: (input) => ({
		height: readUInt32BE(input, 14),
		width: readUInt32BE(input, 18)
	})
};
const svgReg = /<svg\s([^"'>]|"[^"]*"|'[^']*')*>/;
const commentReg = /<!--[\s\S]*?(?:-->|$)/g;
const extractorRegExps = {
	height: /\sheight=(["'])([^%]+?)\1/,
	viewbox: /\sviewbox=(["'])([\s\S]+?)\1/i,
	width: /\swidth=(["'])([^%]+?)\1/
};
const TAG_START = 60;
const TAG_END = 62;
const SVG_TAG = [
	TAG_START,
	115,
	118,
	103
];
const DOUBLE_QUOTE = 34;
const SINGLE_QUOTE = 39;
const unicodeSpaceReg = /^(?:\u00C2\u00A0|\u00E1\u009A\u0080|\u00E2\u0080[\u0080-\u008A\u00A8\u00A9\u00AF]|\u00E2\u0081\u009F|\u00E3\u0080\u0080|\u00EF\u00BB\u00BF)/;
function isSpaceAt(input, offset) {
	const byte = input[offset];
	if (byte < 128) return byte === 32 || byte >= 9 && byte <= 13;
	const bytes = String.fromCharCode(byte, input[offset + 1], input[offset + 2]);
	return unicodeSpaceReg.test(bytes);
}
const isCommentAt = (input, offset) => input[offset] === TAG_START && offset + 3 < input.length && input[offset + 1] === 33 && input[offset + 2] === 45 && input[offset + 3] === 45;
function skipComment(input, offset) {
	for (let end = offset + 4; end + 2 < input.length; end++) if (input[end] === 45 && input[end + 1] === 45 && input[end + 2] === TAG_END) return end + 3;
	return input.length;
}
function extractRoot(input) {
	const length = input.length;
	let start = 0;
	let offset = 0;
	let matched = 0;
	while (offset < length) {
		const byte = input[offset];
		if (byte === TAG_START && isCommentAt(input, offset)) {
			offset = skipComment(input, offset);
			continue;
		}
		if (matched === SVG_TAG.length && isSpaceAt(input, offset)) break;
		if (byte === TAG_START) {
			start = offset;
			matched = 1;
		} else if (matched < SVG_TAG.length && byte === SVG_TAG[matched]) matched++;
		else matched = 0;
		offset++;
	}
	let quote = 0;
	for (offset++; offset < length; offset++) {
		const byte = input[offset];
		if (byte === TAG_START && isCommentAt(input, offset)) {
			offset = skipComment(input, offset) - 1;
			continue;
		}
		if (quote) {
			if (byte === quote) quote = 0;
		} else if (byte === DOUBLE_QUOTE || byte === SINGLE_QUOTE) quote = byte;
		else if (byte === TAG_END) break;
	}
	if (offset >= length) return;
	try {
		return toUTF8String(input, start, offset + 1).replace(commentReg, "");
	} catch {
		return;
	}
}
const INCH_CM = 2.54;
const units = {
	in: 96,
	cm: 96 / INCH_CM,
	em: 16,
	ex: 8,
	m: 96 / INCH_CM * 100,
	mm: 96 / INCH_CM / 10,
	pc: 16,
	pt: 96 / 72,
	px: 1
};
const unitsReg = new RegExp(`^([0-9.]+(?:e\\d+)?)(${Object.keys(units).join("|")})?$`);
function parseLength(len) {
	const m = unitsReg.exec(len);
	if (!m) return;
	return Number(m[1]) * (units[m[2]] || 1);
}
function parseViewbox(viewbox) {
	const bounds = viewbox.trim().split(/[\s,]+/, 4);
	return {
		height: parseLength(bounds[3]),
		width: parseLength(bounds[2])
	};
}
function parseAttributes(root) {
	const width = root.match(extractorRegExps.width);
	const height = root.match(extractorRegExps.height);
	const viewbox = root.match(extractorRegExps.viewbox);
	return {
		height: height && Math.round(parseLength(height[2])),
		viewbox: viewbox && parseViewbox(viewbox[2]),
		width: width && Math.round(parseLength(width[2]))
	};
}
function calculateByDimensions(attrs) {
	return {
		height: attrs.height,
		width: attrs.width
	};
}
function toPixels(length) {
	return Math.max(1, Math.round(length));
}
function calculateByViewbox(attrs, viewbox) {
	const vbWidth = viewbox.width;
	const vbHeight = viewbox.height;
	if (attrs.width) return {
		height: toPixels(attrs.width * vbHeight / vbWidth),
		width: attrs.width
	};
	if (attrs.height) return {
		height: attrs.height,
		width: toPixels(attrs.height * vbWidth / vbHeight)
	};
	return {
		height: toPixels(vbHeight),
		width: toPixels(vbWidth)
	};
}
const SVG = {
	validate: (input) => svgReg.test(toUTF8String(input, 0, 1e3)),
	calculate(input) {
		const root = extractRoot(input);
		if (root) {
			const attrs = parseAttributes(root);
			if (attrs.width && attrs.height) return calculateByDimensions(attrs);
			if (attrs.viewbox?.width && attrs.viewbox.height) return calculateByViewbox(attrs, attrs.viewbox);
		}
		throw new TypeError("Invalid SVG");
	}
};
const TGA = {
	validate(input) {
		return readUInt16LE(input, 0) === 0 && readUInt16LE(input, 4) === 0;
	},
	calculate(input) {
		return {
			height: readUInt16LE(input, 14),
			width: readUInt16LE(input, 12)
		};
	}
};
const TAG_WIDTH = 256;
const TAG_HEIGHT = 257;
const TYPE_SHORT = 3;
const TYPE_LONG = 4;
const TYPE_LONG8 = 16;
const MAX_ENTRIES = 4096;
function readUInt64(input, offset, isBigEndian) {
	const high = readUInt(input, 32, offset + (isBigEndian ? 0 : 4), isBigEndian);
	const low = readUInt(input, 32, offset + (isBigEndian ? 4 : 0), isBigEndian);
	return high * 2 ** 32 + low;
}
function readValue(input, type, offset, isBigEndian, isBigTiff) {
	switch (type) {
		case TYPE_SHORT: return readUInt(input, 16, offset, isBigEndian);
		case TYPE_LONG: return readUInt(input, 32, offset, isBigEndian);
		case TYPE_LONG8: {
			if (!isBigTiff) return;
			const value = readUInt64(input, offset, isBigEndian);
			if (!Number.isSafeInteger(value)) throw new TypeError("Invalid Tiff. Value too large");
			return value;
		}
	}
}
const signatures = /* @__PURE__ */ new Set([
	"49492a00",
	"4d4d002a",
	"49492b00",
	"4d4d002b"
]);
const TIFF = {
	validate: (input) => signatures.has(toHexString(input, 0, 4)),
	calculate(input) {
		const isBigEndian = toUTF8String(input, 0, 2) === "MM";
		const isBigTiff = readUInt(input, 16, 2, isBigEndian) === 43;
		let entryCount;
		let entriesOffset;
		let entrySize;
		if (isBigTiff) {
			const byteSize = readUInt(input, 16, 4, isBigEndian);
			const reserved = readUInt(input, 16, 6, isBigEndian);
			if (byteSize !== 8 || reserved !== 0) throw new TypeError("Invalid BigTIFF header");
			const ifdOffset = readUInt64(input, 8, isBigEndian);
			if (!(ifdOffset + 8 <= input.length)) throw new TypeError("Invalid Tiff. IFD offset out of bounds");
			entryCount = readUInt64(input, ifdOffset, isBigEndian);
			entriesOffset = ifdOffset + 8;
			entrySize = 20;
		} else {
			const ifdOffset = readUInt(input, 32, 4, isBigEndian);
			if (!(ifdOffset + 2 <= input.length)) throw new TypeError("Invalid Tiff. IFD offset out of bounds");
			entryCount = readUInt(input, 16, ifdOffset, isBigEndian);
			entriesOffset = ifdOffset + 2;
			entrySize = 12;
		}
		const tags = {};
		for (let index = 0; index < Math.min(entryCount, MAX_ENTRIES); index++) {
			const offset = entriesOffset + index * entrySize;
			if (offset + entrySize > input.length) break;
			const code = readUInt(input, 16, offset, isBigEndian);
			const type = readUInt(input, 16, offset + 2, isBigEndian);
			if ((isBigTiff ? readUInt64(input, offset + 4, isBigEndian) : readUInt(input, 32, offset + 4, isBigEndian)) === 1) tags[code] = readValue(input, type, offset + (isBigTiff ? 12 : 8), isBigEndian, isBigTiff);
			if (tags[TAG_WIDTH] && tags[TAG_HEIGHT]) break;
		}
		const width = tags[TAG_WIDTH];
		const height = tags[TAG_HEIGHT];
		if (!width || !height) throw new TypeError("Invalid Tiff. Missing tags");
		return {
			height,
			width
		};
	}
};
function calculateExtended(input) {
	return {
		height: 1 + readUInt24LE(input, 7),
		width: 1 + readUInt24LE(input, 4)
	};
}
function calculateLossless(input) {
	return {
		height: 1 + ((input[4] & 15) << 10 | input[3] << 2 | (input[2] & 192) >> 6),
		width: 1 + ((input[2] & 63) << 8 | input[1])
	};
}
function calculateLossy(input) {
	return {
		height: readInt16LE(input, 8) & 16383,
		width: readInt16LE(input, 6) & 16383
	};
}
const typeHandlers = {
	bmp: BMP,
	cur: CUR,
	dds: DDS,
	gif: GIF,
	heic: HEIC,
	icns: ICNS,
	ico: ICO,
	j2c: J2C,
	jp2: JP2,
	jpg: JPG,
	jxl: JXL,
	ktx: KTX,
	png: PNG,
	pnm: PNM,
	psd: PSD,
	svg: SVG,
	tga: TGA,
	tiff: TIFF,
	webp: {
		validate(input) {
			const riffHeader = toUTF8String(input, 0, 4) === "RIFF";
			const webpHeader = toUTF8String(input, 8, 12) === "WEBP";
			const vp8Header = toUTF8String(input, 12, 15) === "VP8";
			return riffHeader && webpHeader && vp8Header;
		},
		calculate(input) {
			if (input.length < 30) throw new TypeError("Invalid WebP, truncated input");
			const chunkHeader = toUTF8String(input, 12, 16);
			input = input.slice(20, 30);
			if (chunkHeader === "VP8X") {
				const extendedHeader = input[0];
				const validStart = (extendedHeader & 192) === 0;
				const validEnd = (extendedHeader & 1) === 0;
				if (validStart && validEnd) return calculateExtended(input);
				else throw new TypeError("Invalid WebP");
			}
			if (chunkHeader === "VP8 " && input[0] !== 47) return calculateLossy(input);
			const signature = toHexString(input, 3, 6);
			if (chunkHeader === "VP8L" && signature !== "9d012a") return calculateLossless(input);
			throw new TypeError("Invalid WebP");
		}
	},
	avif: {
		validate: (input) => detectHeifType(input) === "avif",
		calculate: (input) => HEIC.calculate(input)
	}
};
const keys = Object.keys(typeHandlers);
const firstBytes = {
	56: "psd",
	66: "bmp",
	68: "dds",
	71: "gif",
	73: "tiff",
	77: "tiff",
	82: "webp",
	105: "icns",
	137: "png",
	255: "jpg"
};
function detector(input) {
	const byte = input[0];
	if (byte in firstBytes) {
		const type = firstBytes[byte];
		if (type && typeHandlers[type].validate(input)) return type;
	}
	return keys.find((key) => typeHandlers[key].validate(input));
}
function imageMeta(input) {
	if (!(input instanceof Uint8Array)) throw new TypeError("Input should be a Uint8Array");
	const type = detector(input);
	if (type !== void 0 && type in typeHandlers) {
		const size = typeHandlers[type].calculate(input);
		if (size !== void 0) {
			if (size.images && size.images.length > 1) {
				for (const image of size.images) if (image.width * image.height > size.width * size.height) {
					size.width = image.width;
					size.height = image.height;
				}
			}
			if (!Number.isSafeInteger(size.width) || !Number.isSafeInteger(size.height)) throw new TypeError(`Invalid ${type}, truncated or corrupt input`);
			if (size.width <= 0 || size.height < 0) throw new TypeError(`Invalid ${type}, zero or negative size`);
			size.type = type;
			return size;
		}
	}
	throw new TypeError(`Unsupported file type: ${type}`);
}
export { imageMeta };
