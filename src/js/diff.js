import {ptrToPtrLenToString} from './utils.js';
import {contexts, idToTagName} from './api.js';

function added(ctx, nodeId, ptr) {
	const idParent = getValue(ptr, 'i32');
	const ty = getValue(ptr + 4, 'i32');
	const index = getValue(ptr + 8, 'i32');

	const elem = window.document.createElement(idToTagName[ty]);
	const parentElem = idParent === 0 ? ctx.rootNode : ctx.nodes[idParent];
	ctx.nodes[nodeId] = elem;
	parentElem.insertBefore(elem, parentElem.childNodes[index] || null);
}

function removed(ctx, nodeId, ptr) {
	const elem = ctx.nodes[nodeId];
	elem.parentNode.removeChild(elem);
	delete ctx.nodes[nodeId];
}

function reordered(ctx, nodeId, ptrIn) {
	const ptr = getValue(ptrIn, 'i32');
	const len = getValue(ptrIn + 4, 'i32');

	const elem = ctx.nodes[nodeId];
	const children = [];
	const childLen = elem.childNodes.length;
	for (let i = 0; i < childLen; i += 1) {
		children[i] = elem.childNodes[i];
	}

	let index = 0;
	for (let i = 0; i < len; i += 1) {
		const p = ptr + i * 8;
		const currIndex = getValue(p, 'i32');
		const lastIndex = getValue(p + 4, 'i32');

		while (index < currIndex) {
			elem.appendChild(children[index]);
			index += 1;
		}

		elem.appendChild(children[lastIndex]);
		index += 1;
	}
	while (index < childLen) {
		elem.appendChild(children[index]);
		index += 1;
	}
	_free(ptr);
}

function paramSet(ctx, nodeId, ptr) {
	const key = ptrToPtrLenToString(ptr);
	const val = ptrToPtrLenToString(ptr + 8);

	ctx.nodes[nodeId].setAttribute(key, val);
}

function paramSetToTrue(ctx, nodeId, ptr) {
	const key = ptrToPtrLenToString(ptr);

	ctx.nodes[nodeId].setAttribute(key, key);
}

function paramRemoved(ctx, nodeId, ptr) {
	const key = ptrToPtrLenToString(ptr);

	ctx.nodes[nodeId].removeAttribute(key);
}

function textAdded(ctx, nodeId, ptr) {
	const idParent = getValue(ptr, 'i32');
	const index = getValue(ptr + 4, 'i32');
	const text = ptrToPtrLenToString(ptr + 8);

	const elem = window.document.createTextNode(text);
	const parentElem = idParent === 0 ? ctx.rootNode : ctx.nodes[idParent];
	ctx.nodes[nodeId] = elem;
	parentElem.insertBefore(elem, parentElem.childNodes[index] || null);
}

function textSet(ctx, nodeId, ptr) {
	const text = ptrToPtrLenToString(ptr);

	ctx.nodes[nodeId].nodeValue = text;
}

export default function diff(ctxId, ptr, len) {
	const ctx = contexts[ctxId];

	for (let i = 0; i < len; i += 1) {
		const p = ptr + i * 24;
		const nodeId = getValue(p, 'i32');
		const tag = getValue(p + 4, 'i32');

		console.log('diff nodeId:', nodeId, '; tag:', tag);

		switch (tag) {
			case 0: // Added
				added(ctx, nodeId, p + 8);
				break;

			case 1: // Removed
				removed(ctx, nodeId, p + 8);
				break;

			case 2: // Reordered
				reordered(ctx, nodeId, p + 8);
				break;

			case 3: // ParamSet
				paramSet(ctx, nodeId, p + 8);
				break;

			case 4: // ParamSetToTrue
				paramSetToTrue(ctx, nodeId, p + 8);
				break;

			case 5: // ParamRemoved
				paramRemoved(ctx, nodeId, p + 8);
				break;

			case 6: // TextAdded
				textAdded(ctx, nodeId, p + 8);
				break;

			case 7: // TextSet
				textSet(ctx, nodeId, p + 8);
				break;

			default:
				throw new Error('unreachable');
		}
	}
}
