// https://github.com/kripken/emscripten/blob/6dc4ac5f9e4d8484e273e4dcc554f809738cedd6/src/preamble.js#L532-L580
export function utf8ToString(ptr, len) {
	const u8Array = HEAPU8;
	const endPtr = ptr + len;

	if (endPtr - ptr > 16 && u8Array.subarray && UTF8Decoder) {
		return UTF8Decoder.decode(u8Array.subarray(ptr, endPtr));
	}

	let idx = ptr;
	let str = '';
	while (idx < endPtr) {
		// For UTF8 byte structure, see http://en.wikipedia.org/wiki/UTF-8#Description and
		// https://www.ietf.org/rfc/rfc2279.txt and https://tools.ietf.org/html/rfc3629
		let u0 = u8Array[idx++];
		if (!(u0 & 0x80)) {
			str += String.fromCharCode(u0);
			continue;
		}
		const u1 = u8Array[idx++] & 63;
		if ((u0 & 0xE0) === 0xC0) {
			str += String.fromCharCode(((u0 & 31) << 6) | u1);
			continue;
		}
		const u2 = u8Array[idx++] & 63;
		if ((u0 & 0xF0) === 0xE0) {
			u0 = ((u0 & 15) << 12) | (u1 << 6) | u2;
		} else {
			const u3 = u8Array[idx++] & 63;
			if ((u0 & 0xF8) === 0xF0) {
				u0 = ((u0 & 7) << 18) | (u1 << 12) | (u2 << 6) | u3;
			} else {
				const u4 = u8Array[idx++] & 63;
				if ((u0 & 0xFC) === 0xF8) {
					u0 = ((u0 & 3) << 24) | (u1 << 18) | (u2 << 12) | (u3 << 6) | u4;
				} else {
					const u5 = u8Array[idx++] & 63;
					u0 = ((u0 & 1) << 30) | (u1 << 24) | (u2 << 18) | (u3 << 12) | (u4 << 6) | u5;
				}
			}
		}
		if (u0 < 0x10000) {
			str += String.fromCharCode(u0);
		} else {
			const ch = u0 - 0x10000;
			str += String.fromCharCode(0xD800 | (ch >> 10), 0xDC00 | (ch & 0x3FF));
		}
	}
	return str;
}
