# Massively Parallel Fibonacci

This library provides multiple approaches to compute Fibonacci numbers efficiently, including
hybrid, parallel, and asynchronous methods. The primary goal is to compute Fibonacci sequences
for large inputs with optimal performance, using both matrix exponentiation and iterative methods,
along with concurrent execution mechanisms where applicable.

## Key Features

- **Matrix Exponentiation**: Used for efficient computation of Fibonacci numbers at specific
  positions.
- **Iterative Generation**: Generates chunks of Fibonacci numbers using iteration.
- **Hybrid Approaches**: Combines matrix exponentiation for boundary values with iteration
  for intermediate values.
- **Concurrency**: Several functions leverage parallelism, both synchronous and asynchronous, to
  compute large sequences of Fibonacci numbers faster.

## Fibonacci Computation Approaches

### `fibonacci_matrix`
Computes Fibonacci(n-1) and Fibonacci(n) using matrix exponentiation. This method is efficient
for large values of `n` and is the basis for chunk-based and hybrid computations.

### `fibonacci_chunk`
Iteratively computes a chunk of `n_steps` Fibonacci numbers, starting from two initial values,
`F(n-1)` and `F(n)`. Useful for generating chunks of Fibonacci sequences efficiently.

### `seq_basic`
Generates the first `n` Fibonacci numbers using a simple iterative approach. This is a direct
method that runs in O(n) time and uses O(n) space.

### `seq_hybrid`
Combines matrix exponentiation and iteration to generate a sequence of Fibonacci numbers.
It uses matrix exponentiation to compute boundary values and iterates within chunks for intermediate
values.

### `seq_hybrid_rayon`
Parallel version of `seq_hybrid`, leveraging the Rayon library for concurrent chunk processing.
This significantly speeds up the computation by parallelizing the chunk processing.

### `seq_hybrid_tokio`
Asynchronous version using Tokio, allowing the computation to be broken into tasks that run concurrently.
It controls the number of concurrent tasks using a semaphore for efficient resource utilization.

### `seq_hybrid_kanal`
Multi-threaded version using `std::thread` and the `kanal` library for communication between threads.
The chunks are computed in parallel threads, and the results are sent back to the main thread for
collection and combination.

### `seq_hybrid_kanal_tokio`
Combines the asynchronous concurrency of Tokio with the multi-threaded communication capabilities of
`kanal`. This method uses asynchronous tasks to compute chunks in parallel and communicates the results
back to the main thread.

### Usage Example
```rust
use fibonacci_assesment::fibonacci::seq_hybrid;
use num_bigint::BigUint;
let fib_sequence = seq_hybrid(100, 10);
assert_eq!(fib_sequence[99], BigUint::parse_bytes(b"218922995834555169026", 10).unwrap());
```

## Testing

To verify the correctness of generating the first 100,000 Fibonacci sequence numbers, you will need
to download and load a binary file containing this sequence, named `fibonacci_data.bin`. This file
can be downloaded from the root directory of the project using the following command:

```sh
wget -O fibonacci_data.bin "https://www.dropbox.com/scl/fi/6zbvut3evptw1mlm2v12e/fibonacci_data.bin?rlkey=rurifdumjozw3x6ar4sr8nwn7&st=a0z85mgk&dl=1"
```

Once downloaded, you can run `cargo test`, which will verify that the Fibonacci sequence generation
methods provided in this repository are correct.

## Benchmarking

To benchmark the Fibonacci sequence computation approaches, we use the `bench` crate to measure the time required to generate the first 100,000 Fibonacci sequence numbers for each approach. You can run the benchmarks by executing `cargo bench`.

| Approach                                       |   Average Time (ms) |
|:-----------------------------------------------|--------------------:|
| fib_hybrid_kanal_tokio_2000chunks              |             34.883  |
| fib_hybrid_kanal_tokio_1000chunks_30concurrent |             35.875  |
| fib_hybrid_tokio_1500chunks                    |             36.209  |
| fib_hybrid_kanal_tokio_1000chunks              |             36.35   |
| fib_hybrid_tokio_1000chunks                    |             36.9185 |
| fib_hybrid_kanal_tokio_1500chunks              |             37.4315 |
| fib_hybrid_tokio_1000chunks_30concurrent       |             37.5095 |
| fib_hybrid_kanal_tokio_700chunks               |             38.104  |
| fib_hybrid_tokio_500chunks                     |             41.2765 |
| fib_seq_hybrid_rayon_1000chunks                |             43.616  |
| fib_hybrid_kanal_tokio_5000chunks              |             43.839  |
| fib_seq_hybrid_rayon_1500chunks                |             46.0795 |
| fib_seq_hybrid_rayon_500chunks                 |             46.714  |
| fib_seq_hybrid_kanal_1000chunks                |             55.7885 |
| fib_seq_basic                                  |            168.545  |
| fib_seq_hybrid_10000chunks                     |            188.18   |




