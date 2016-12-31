const rollupEmscripten = require('rollup-emscripten').default;
const babel = require('rollup-plugin-babel');

rollupEmscripten({
	entry: './src/js/main.js',
	localPrefix: 'vtree_html',
	plugins: [
		babel({
			presets: [
				['es2015', {modules: false}],
			]
		}),
	]
})
	.then(res => res.write('./target/libs/vtree_html.js'))
	.catch(e => {
		setTimeout(() => {throw e;}, 0);
	});
