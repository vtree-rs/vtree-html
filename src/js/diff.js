import {utf8ToString} from './utils.js';
import {contexts, idToTagName} from './api.js';

function added(ctx, nodeId, ptr) {
	const idParent = getValue(ptr, 'i32');
	const ty = getValue(ptr + 4, 'i32');
	const index = getValue(ptr + 8, 'i32');

	console.log('added - nodeId:', nodeId, 'idParent:', idParent, 'ty:', ty, 'index:', index);

	let elem;
	if (ty === 0) {
		elem = window.document.createTextNode('');
	} else {
		elem = window.document.createElement(idToTagName[ty]);
	}
	const parentElem = idParent === 0 ? ctx.rootNode : ctx.nodes[idParent].elem;
	ctx.nodes[nodeId] = {
		elem: elem,
		type: ty,
	};
	if (parentElem.childNodes.length < index) {
		parentElem.appendChild(elem);
	} else {
		parentElem.insertBefore(elem, parentElem.childNodes[index]);
	}
}

function removed(ctx, nodeId, ptr) {
	console.log('removed - nodeId:', nodeId);
	if (!ctx.nodes[nodeId]) return;
	const elem = ctx.nodes[nodeId].elem;
	elem.parentNode.removeChild(elem);
	delete ctx.nodes[nodeId];
}

function reordered(ctx, nodeId, ptr) {
	const lastIndex = getValue(ptr, 'i32');
	const currIndex = getValue(ptr + 4, 'i32');
}

function paramSet(ctx, nodeId, ptr) {
	const valPtr = getValue(ptr + 8, '*');
	const valLen = getValue(ptr + 12, 'i32');
	const val = utf8ToString(valPtr, valLen);

	console.log('paramSet - nodeId:', nodeId, 'val:', val);

	const node = ctx.nodes[nodeId];
	if (node.type === 0) {
		node.elem.nodeValue = val;
	} else {
		const keyPtr = getValue(ptr, '*');
		const keyLen = getValue(ptr + 4, 'i32');
		const key = utf8ToString(keyPtr, keyLen);

		node.elem.setAttribute(key, val);
	}
}

function paramSetToTrue(ctx, nodeId, ptr) {
	const keyPtr = getValue(ptr, '*');
	const keyLen = getValue(ptr + 4, 'i32');
	const key = utf8ToString(keyPtr, keyLen);

	const node = ctx.nodes[nodeId];
	if (node.type === 0) throw new Error('illegal node type "text"');
	node.elem.setAttribute(key, key);
}

function paramRemoved(ctx, nodeId, ptr) {
	const keyPtr = getValue(ptr, '*');
	const keyLen = getValue(ptr + 4, 'i32');
	const key = utf8ToString(keyPtr, keyLen);

	const node = ctx.nodes[nodeId];
	if (node.type === 0) throw new Error('illegal node type "text"');
	node.elem.removeAttribute(key);
}

export default function diff(ctxId, ptr, len) {
	console.log('diff ptr:', ptr, '; len:', len);

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

			default:
				throw new Error('unreachable');
		}
	}
}
