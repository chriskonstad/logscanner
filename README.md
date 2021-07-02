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

![Screen Shot 2021-07-02 at 12 46 33 PM](https://user-images.githubusercontent.com/1539144/124322004-89675e00-db33-11eb-9a41-6cb044a1117e.png)
