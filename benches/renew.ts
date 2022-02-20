import { BenchmarkTimer, bench, runBenchmarks } from '../deno/test_util/std/testing/bench.ts';

const gotcha = Deno.dlopen('./libgotcha.so', {
	libgotcha_group_limit: {
		parameters: [],
		result: 'u64',
	},
	libgotcha_group_renew: {
		parameters: ['i64'],
		result: 'void',
	},
});

const count = gotcha.symbols.libgotcha_group_limit() as number;
let iter = 0;
bench({
	name: 'renew',
	func: function(timer: BenchmarkTimer) {
		timer.start();
		gotcha.symbols.libgotcha_group_renew(iter++ % count + 1);
		timer.stop();
	},
	runs: count * 100,
});

runBenchmarks();
