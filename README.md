# logscanner
Quickly scan logs for numbers, and highlight interesting numbers.

```
# either pass -i or pipe data through it
$ cargo run -- 'Took (\d+)ms' --bold -i test.data
Took 100ms
Took 950ms
Took 50ms
Took 10ms
Took 5ms
Not a result
Took 1000ms
Took 101ms
Took 951ms
Took 51ms
Took 11ms
Took 6ms
Not a result
10x not a result
Neither is 10ms
Took 1001ms
```
The numbers will be highlighted based on if p50+, p90+, or p99+.

You can pass `--highlight` to highlight all numbers yellow instead.

You can also pass `--sort asc` or `--sort desc` to sort the lines by the integer
captured by your regex, although this will drop non-matching lines.

You can pass `--matching` to drop all non-matching lines without sorting.
