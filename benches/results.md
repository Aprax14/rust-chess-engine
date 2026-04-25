## Benchmark results

**Commit:** `33e97d2`  
**Date:** 2026-04-25 10:34 UTC  
**CPU:** AMD Ryzen 7 6800H with Radeon Graphics (16 cores)  
**RAM:** 14Gi  
**OS:** Linux 6.18.24-1-lts

```
move_generation/start   time:   [2.2132 µs 2.2204 µs 2.2298 µs]
                        change: [-1.2890% -1.0590% -0.8398%] (p = 0.00 < 0.05)
                        Change within noise threshold.
Found 16 outliers among 100 measurements (16.00%)
  16 (16.00%) high severe
move_generation/mid_game
                        time:   [2.9552 µs 2.9594 µs 2.9659 µs]
                        change: [-1.6435% -1.3874% -1.0578%] (p = 0.00 < 0.05)
                        Performance has improved.
Found 13 outliers among 100 measurements (13.00%)
  2 (2.00%) low severe
  4 (4.00%) low mild
  2 (2.00%) high mild
  5 (5.00%) high severe
move_generation/endgame time:   [1.5958 µs 1.6017 µs 1.6091 µs]
                        change: [-1.2611% -0.8732% -0.4679%] (p = 0.00 < 0.05)
                        Change within noise threshold.
Found 19 outliers among 100 measurements (19.00%)
  3 (3.00%) low mild
  1 (1.00%) high mild
  15 (15.00%) high severe
move_generation/tactics time:   [3.2755 µs 3.2827 µs 3.2910 µs]
                        change: [-2.1130% -1.8761% -1.6321%] (p = 0.00 < 0.05)
                        Performance has improved.
Found 12 outliers among 100 measurements (12.00%)
  1 (1.00%) low mild
  4 (4.00%) high mild
  7 (7.00%) high severe

static_eval/start       time:   [151.54 ns 151.88 ns 152.27 ns]
                        change: [-2.4597% -2.1590% -1.8629%] (p = 0.00 < 0.05)
                        Performance has improved.
Found 8 outliers among 100 measurements (8.00%)
  2 (2.00%) low mild
  4 (4.00%) high mild
  2 (2.00%) high severe
static_eval/mid_game    time:   [222.01 ns 222.48 ns 222.96 ns]
                        change: [-2.6126% -1.9533% -1.0233%] (p = 0.00 < 0.05)
                        Performance has improved.
Found 8 outliers among 100 measurements (8.00%)
  3 (3.00%) low mild
  4 (4.00%) high mild
  1 (1.00%) high severe
static_eval/endgame     time:   [147.13 ns 147.44 ns 147.82 ns]
                        change: [-3.2264% -2.7941% -2.3764%] (p = 0.00 < 0.05)
                        Performance has improved.
Found 5 outliers among 100 measurements (5.00%)
  1 (1.00%) low mild
  1 (1.00%) high mild
  3 (3.00%) high severe
static_eval/tactics     time:   [227.97 ns 228.63 ns 229.37 ns]
                        change: [-2.4594% -2.0842% -1.7222%] (p = 0.00 < 0.05)
                        Performance has improved.
Found 8 outliers among 100 measurements (8.00%)
  2 (2.00%) low mild
  3 (3.00%) high mild
  3 (3.00%) high severe

search_depth_4/start    time:   [16.237 ms 16.448 ms 16.613 ms]
                        change: [-1.5934% -0.3978% +0.9743%] (p = 0.56 > 0.05)
                        No change in performance detected.
search_depth_4/mid_game time:   [29.017 ms 29.190 ms 29.313 ms]
                        change: [+0.6780% +1.3853% +2.1690%] (p = 0.00 < 0.05)
                        Change within noise threshold.
search_depth_4/endgame  time:   [395.62 µs 398.72 µs 401.32 µs]
                        change: [-1.2213% -0.3670% +0.4591%] (p = 0.50 > 0.05)
                        No change in performance detected.
search_depth_4/tactics  time:   [211.12 ms 211.71 ms 212.76 ms]
                        change: [-0.9515% -0.4553% +0.0389%] (p = 0.13 > 0.05)
                        No change in performance detected.

```
