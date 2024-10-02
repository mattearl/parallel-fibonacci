//! # Fibonacci Sequence Library
//!
//! This library provides multiple approaches to compute Fibonacci numbers efficiently, including
//! hybrid, parallel, and asynchronous methods. The primary goal is to compute Fibonacci sequences
//! for large inputs with optimal performance, using both matrix exponentiation and iterative methods,
//! along with concurrent execution mechanisms where applicable.
//!
//! ## Key Features
//! - **Matrix Exponentiation**: Used for efficient computation of Fibonacci numbers at specific
//!   positions.
//! - **Iterative Generation**: Generates chunks of Fibonacci numbers based using iteration.
//! - **Hybrid Approaches**: Combines matrix exponentiation for boundary values with iteration
//!   for intermediate values.
//! - **Concurrency**: Several functions leverage parallelism, both synchronous and asynchronous, to
//!   compute large sequences of Fibonacci numbers faster.
//!
//! ## Overview of Functions
//!
//! ### Errors
//! - `FibonacciSequenceError`: Enum representing various errors that might occur during execution,
//!   such as Tokio or standard join errors, semaphore acquisition issues, or send/receive errors in
//!   threaded communication.
//!
//! ### Fibonacci Computation Approaches
//!
//! #### `fibonacci_matrix``
//! Computes Fibonacci(n-1) and Fibonacci(n) using matrix exponentiation. This method is efficient
//! for large values of `n` and is the basis for chunk-based and hybrid computations.
//!
//! #### `fibonacci_chunk`
//! Iteratively computes a chunk of `n_steps` Fibonacci numbers, starting from two initial values,
//! `F(n-1)` and `F(n)`. Useful for generating chunks of Fibonacci sequences efficiently.
//!
//! #### `seq_basic`
//! Generates the first `n` Fibonacci numbers using a simple iterative approach. This is a direct
//! method that runs in O(n) time and uses O(n) space.
//!
//! #### `seq_hybrid`
//! Combines matrix exponentiation and iteration to generate a sequence of Fibonacci numbers.
//! It uses matrix exponentiation to compute boundary values and iterates within chunks for intermediate
//! values.
//!
//! #### `seq_hybrid_rayon``
//! Parallel version of `seq_hybrid`, leveraging the Rayon library for concurrent chunk processing.
//! This significantly speeds up the computation by parallelizing the chunk processing.
//!
//! #### `seq_hybrid_tokio`
//! Asynchronous version using Tokio, allowing the computation to be broken into tasks that run concurrently.
//! It controls the number of concurrent tasks using a semaphore for efficient resource utilization.
//!
//! #### `seq_hybrid_kanal`
//! Multi-threaded version using `std::thread` and the `kanal` library for communication between threads.
//! The chunks are computed in parallel threads, and the results are sent back to the main thread for
//! collection and combination.
//!
//! #### `seq_hybrid_kanal_tokio`
//! Combines the asynchronous concurrency of Tokio with the multi-threaded communication capabilities of
//! `kanal`. This method uses asynchronous tasks to compute chunks in parallel and communicates the results
//! back to the main thread.
//!
//! ## Usage Example
//! ```rust
//! use fibonacci_assesment::fibonacci::seq_hybrid;
//! use num_bigint::BigUint;
//! let fib_sequence = seq_hybrid(100, 10);
//! assert_eq!(fib_sequence[99], BigUint::parse_bytes(b"218922995834555169026", 10).unwrap());
//! ```

pub mod fibonacci;
pub mod math;
