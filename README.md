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

You can also pass `--sorting asc` or `--sorting desc` to sort the lines by the integer
captured by your regex, although this will drop non-matching lines.

You can pass `--matching` to drop all non-matching lines without sorting.

![Screen Shot 2021-07-02 at 12 46 33 PM](https://user-images.githubusercontent.com/1539144/124322004-89675e00-db33-11eb-9a41-6cb044a1117e.png)
![Screen Shot 2021-07-02 at 12 48 48 PM](https://user-images.githubusercontent.com/1539144/124322184-db0fe880-db33-11eb-99c7-1f356e23fce4.png)

## Performance vs Grep
```
$ wc -cl ../large.data
3575880 686568960 ../large.data

$ hyperfine --warmup 3 "cargo run --release -- 'Took (\d+)ms' -b -i ../large.data" "grep -e 'Took [[:digit:]]\+ms' ../large.data"
Benchmark #1: cargo run --release -- 'Took (\d+)ms' -b -i ../large.data
  Time (mean ± σ):      2.898 s ±  0.163 s    [User: 5.575 s, System: 0.553 s]
  Range (min … max):    2.743 s …  3.184 s    10 runs

Benchmark #2: grep -e 'Took [[:digit:]]\+ms' ../large.data
  Time (mean ± σ):      2.987 s ±  0.050 s    [User: 2.870 s, System: 0.108 s]
  Range (min … max):    2.947 s …  3.112 s    10 runs

Summary
  'cargo run --release -- 'Took (\d+)ms' -b -i ../large.data' ran
    1.03 ± 0.06 times faster than 'grep -e 'Took [[:digit:]]\+ms' ../large.data'
```
