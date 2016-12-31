import {utf8ToString} from './utils.js';

export const idToTagName = [
	'text', // 0
	'div', // 1
	'span', // 2
];

export let lastCtxId = 0;
export const contexts = {};

export function createContext(nodeIdPtr, nodeIdLen) {
	const nodeId = utf8ToString(nodeIdPtr, nodeIdLen);
	const ctxId = lastCtxId;
	lastCtxId += 1;
	contexts[ctxId] = {
		rootNode: window.document.getElementById(nodeId),
		nodes: {},
	};
	return ctxId;
}

export function removeContext(ctxId) {
	console.log(`remove_context: ${ctxId}`);

	const ctx = contexts[ctxId];
	const rootNode = ctx.rootNode;
	while (rootNode.firstChild) {
		rootNode.removeChild(rootNode.firstChild);
	}
	delete contexts[ctxId];
}

export {default as diff} from './diff.js';
