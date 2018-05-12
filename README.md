# wordcountrs

It counts your words.

**POST** */words*

```
{
    "word" : ["word", "word", "word"]
}
```

Response:

```
{
    "counts" {
        "word": 3
    }
}
```

To make it scoot use the lua scripts with *wrk*:

```
$ wrk -c 8 -t 8 -d 60s --latency --timeout 60s -s lulz.lua http://localhost:8000/words
```
or
```
$ wrk -c 8 -t 8 -d 60s --latency --timeout 60s -s perf_test.lua http://localhost:8000/words
```


To make a flamegraph use *perf*:

```
$ perf record -a --call-graph dwarf -- target/release/wordcountrs
$ perf script -i perf.data | stackcollapse-perf.pl | flamegraph.pl > flame.svg
```
