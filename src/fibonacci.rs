use kanal::{bounded, SendError};
use num_bigint::BigUint;
use num_traits::{One, Zero};
use rayon::iter::{IndexedParallelIterator, IntoParallelIterator, ParallelIterator};
use std::thread;
use std::{cmp, sync::Arc};
use tokio::sync::{AcquireError, Semaphore};
use tokio::task::{self, JoinError, JoinHandle};

use crate::math::{matrix_pow, Matrix};

#[derive(Debug, thiserror::Error)]
pub enum FibonacciSequenceError {
    #[error("Tokio Join error: {0:?}")]
    TokioJoin(#[from] JoinError),
    #[error("Std Join error: {0:?}")]
    StdJoin(String),
    #[error("Acquire error: {0:?}")]
    Acquire(#[from] AcquireError),
    #[error("Send error: {0:?}")]
    Send(#[from] SendError),
}

/// Function to compute Fibonacci(n-1) and Fibonacci(n) using matrix exponentiation
pub fn fibonacci_matrix(n: usize) -> (BigUint, BigUint) {
    if n == 0 {
        return (BigUint::zero(), BigUint::one());
    }

    let base_matrix = Matrix {
        a: BigUint::one(),
        b: BigUint::one(),
        c: BigUint::one(),
        d: BigUint::zero(),
    };

    // Compute the nth power of the base matrix
    let result_matrix = matrix_pow(base_matrix, n);

    (result_matrix.c, result_matrix.a) // F(n-1) and F(n)
}

/// Computes `n_steps` Fibonacci numbers iteratively, starting from two initial values `f_n1` (F(n-1))
/// and `f_n` (F(n)). Useful for generating Fibonacci numbers in chunks.
///
/// # Parameters
/// - `n_steps`: Number of Fibonacci numbers to compute.
/// - `f_n1`: The (n-1)-th Fibonacci number.
/// - `f_n`: The n-th Fibonacci number.
///
/// # Returns
/// - A vector containing `n_steps` Fibonacci numbers starting from `f_n`.
///
/// # Example
/// ```rust
/// use fibonacci_assesment::fibonacci::fibonacci_chunk;
/// use num_bigint::BigUint;
/// let f_n1 = BigUint::from(5_u32); // F(5) = 5
/// let f_n = BigUint::from(8_u32);  // F(6) = 8
/// let chunk = fibonacci_chunk(4, f_n1, f_n);
/// assert_eq!(chunk, vec![
///     BigUint::from(8_u32),  // F(6)
///     BigUint::from(13_u32), // F(7)
///     BigUint::from(21_u32), // F(8)
///     BigUint::from(34_u32), // F(9)
/// ]);
/// ```
pub fn fibonacci_chunk(n_steps: usize, f_n1: BigUint, f_n: BigUint) -> Vec<BigUint> {
    let mut fibs = Vec::with_capacity(n_steps);
    let mut a = f_n1;
    let mut b = f_n;

    for _ in 0..n_steps {
        let next = &a + &b;
        fibs.push(b.clone());
        a = b;
        b = next;
    }

    fibs
}

/// Generates the first `n` Fibonacci numbers using an iterative approach.
///
/// This algorithm computes each Fibonacci number by summing the two previous values,
/// storing the results in a vector. It runs in O(n) time and uses O(n) space.
///
/// # Parameters
/// - `count`: The number of Fibonacci numbers to compute. Must be 2 or greater.
///
/// # Returns
/// - A vector containing the first `count` Fibonacci numbers.
///
/// # Panics
/// - This function will panic if `count` is less than 2.
///
/// # Example
/// ```
/// use fibonacci_assesment::fibonacci;
/// use num_bigint::BigUint;
/// let fib_sequence = fibonacci::seq_basic(10);
/// assert_eq!(fib_sequence[0], BigUint::from(0_u32));  // F(0)
/// assert_eq!(fib_sequence[1], BigUint::from(1_u32));  // F(1)
/// assert_eq!(fib_sequence[8], BigUint::from(21_u32)); // F(8)
/// assert_eq!(fib_sequence[9], BigUint::from(34_u32)); // F(9)
/// ```
pub fn seq_basic(count: usize) -> Vec<BigUint> {
    assert!(count >= 2, "Count must be 2 or greater");
    let mut fib_sequence = Vec::with_capacity(count);
    fib_sequence.push(BigUint::zero());
    fib_sequence.push(BigUint::one());
    for i in 2..count {
        let next_value = &fib_sequence[i - 1] + &fib_sequence[i - 2];
        fib_sequence.push(next_value);
    }
    fib_sequence
}

/// Hybrid approach: Uses matrix exponentiation for computing Fibonacci boundary values
/// and iteration for calculating Fibonacci numbers within those boundaries.
///
/// Parameters:
/// - `count`: The number of Fibonacci numbers to compute. Must be 2 or greater.
/// - `chunk_size`: Size of each chunk to be processed iteratively.
///
/// Returns:
/// - A vector containing the first `count` Fibonacci numbers.
///
/// # Panics
/// - This function will panic if `count` is less than 2.
pub fn seq_hybrid(count: usize, chunk_size: usize) -> Vec<BigUint> {
    assert!(count >= 2, "Count must be 2 or greater");
    let mut result = vec![BigUint::zero(), BigUint::one()]; // Start with F(0), F(1)

    for start in (2..count).step_by(chunk_size) {
        let end = std::cmp::min(start + chunk_size - 1, count - 1);

        // Compute F(start-1) and F(start) using matrix exponentiation
        let (f_start_minus_1, f_start) = fibonacci_matrix(start - 1);

        // Compute the Fibonacci numbers iteratively within the chunk
        let chunk = fibonacci_chunk(end - start + 1, f_start_minus_1, f_start);

        result.extend(chunk);
    }

    result
}

/// Hybrid approach with parallel processing: Combines matrix exponentiation for boundary
/// Fibonacci values and iteration to calculate the sequence, leveraging Rayon for parallelism.
///
/// Parameters:
/// - `count`: The number of Fibonacci numbers to compute. Must be 2 or greater.
/// - `chunk_size`: Size of each chunk to be processed iteratively in parallel.
///
/// Returns:
/// - A vector containing the first `count` Fibonacci numbers.
///
/// # Panics
/// - This function will panic if `count` is less than 2.
pub fn seq_hybrid_rayon(count: usize, chunk_size: usize) -> Vec<BigUint> {
    assert!(count >= 2, "Count must be 2 or greater");
    let mut result = vec![BigUint::zero(), BigUint::one()]; // Start with F(0), F(1)

    // Create a parallel iterator over the start indices
    let chunks: Vec<Vec<BigUint>> = (2..count)
        .into_par_iter()
        .step_by(chunk_size)
        .map(|start| {
            let end = std::cmp::min(start + chunk_size - 1, count - 1);

            // Compute F(start-1) and F(start) using matrix exponentiation
            let (f_start_minus_1, f_start) = fibonacci_matrix(start - 1);

            // Compute the Fibonacci numbers iteratively within the chunk
            fibonacci_chunk(end - start + 1, f_start_minus_1, f_start)
        })
        .collect();

    // Extend the result with all the chunks
    for chunk in chunks {
        result.extend(chunk);
    }

    result
}

/// Hybrid approach with asynchronous parallelism using Tokio: Combines matrix exponentiation for boundary
/// Fibonacci values and iteration to calculate the sequence in parallel using asynchronous tasks.
///
/// Parameters:
/// - `count`: The number of Fibonacci numbers to compute. Must be 2 or greater.
/// - `chunk_size`: Size of each chunk to be processed iteratively in parallel.
/// - `max_concurrent_tasks`: Maximum number of concurrent asynchronous tasks allowed.
///
/// Returns:
/// - A vector containing the first `count` Fibonacci numbers, or an error if encountered.
///
/// # Panics
/// - This function will panic if `count` is less than 2.
pub async fn seq_hybrid_tokio(
    count: usize,
    chunk_size: usize,
    max_concurrent_tasks: usize,
) -> Result<Vec<BigUint>, FibonacciSequenceError> {
    assert!(count >= 2, "Count must be 2 or greater");
    let semaphore = Arc::new(Semaphore::new(max_concurrent_tasks));

    // Pre-allocate the result vector with the correct size
    let mut result = vec![BigUint::zero(), BigUint::one()]; // Start with F(0), F(1)
    result.resize(count, BigUint::zero());

    let mut tasks = vec![];

    for start in (2..count).step_by(chunk_size) {
        let end = std::cmp::min(start + chunk_size - 1, count - 1);
        let semaphore = Arc::clone(&semaphore);

        // Spawn a new task for each chunk
        let task: JoinHandle<Result<(usize, Vec<BigUint>), FibonacciSequenceError>> =
            task::spawn(async move {
                // Acquire a semaphore permit to control concurrency
                let _permit = semaphore.acquire().await?;

                // Compute F(start-1) and F(start) using matrix exponentiation
                let (f_start_minus_1, f_start) = fibonacci_matrix(start - 1);

                // Compute the Fibonacci numbers iteratively within the chunk
                let chunk = fibonacci_chunk(end - start + 1, f_start_minus_1, f_start);

                Ok((start, chunk)) // Return the start index and the computed chunk
            });

        tasks.push(task);
    }

    // Wait for all tasks to complete and insert chunks directly into the result
    for task in tasks {
        let (start, chunk) = task.await??;
        // Insert the chunk directly into the pre-allocated result vector
        for (i, value) in chunk.into_iter().enumerate() {
            result[start + i] = value;
        }
    }

    Ok(result)
}

/// Hybrid approach with multi-threading using `std::thread` and `kanal` for communication:
/// Combines matrix exponentiation for boundary Fibonacci values an iteration, with
/// communication between threads handled through the `kanal` library.
///
/// Parameters:
/// - `count`: The number of Fibonacci numbers to compute. Must be 2 or greater.
/// - `chunk_size`: Size of each chunk to be processed iteratively across multiple threads.
///
/// Returns:
/// - A vector containing the first `count` Fibonacci numbers, or an error if encountered.
///
/// # Panics
/// - This function will panic if `count` is less than 2.
pub fn seq_hybrid_kanal(
    count: usize,
    chunk_size: usize,
) -> Result<Vec<BigUint>, FibonacciSequenceError> {
    assert!(count >= 2, "Count must be 2 or greater");
    // Create a channel to communicate between threads
    let (sender, receiver) = bounded::<(usize, Vec<BigUint>)>(count / chunk_size + 1);

    // Vector to store join handles to propagate thread results/errors back to main thread
    let mut handles = Vec::new();

    for start in (2..count).step_by(chunk_size) {
        let end = cmp::min(start + chunk_size - 1, count - 1);

        let sender = sender.clone();
        let handle = thread::spawn(move || -> Result<(), FibonacciSequenceError> {
            // Compute boundary Fibonacci numbers for this chunk
            let (f_start_minus_1, f_start) = fibonacci_matrix(start - 1);

            // Compute the Fibonacci numbers for the chunk iteratively
            let chunk = fibonacci_chunk(end - start + 1, f_start_minus_1, f_start);

            // Send the result to the main thread
            sender.send((start, chunk))?;

            Ok(())
        });

        // Store the handle to join later and propagate errors
        handles.push(handle);
    }

    // Drop the sender to allow the receiver to exit after all threads finish
    drop(sender);

    // Wait for all threads to finish and propagate any errors
    for handle in handles {
        handle
            .join()
            .map_err(|e| FibonacciSequenceError::StdJoin(format!("Thread panicked: {:?}", e)))??;
    }

    // Collect the results
    let mut results = Vec::new();
    for (start, chunk) in receiver {
        results.push((start, chunk));
    }

    // Sort the results by the starting index
    results.sort_by_key(|(start, _)| *start);

    // Append each chunk to the final result
    let mut final_result = vec![BigUint::zero(), BigUint::one()]; // Start with F(0), F(1)
    for (_, chunk) in results.into_iter() {
        final_result.extend(chunk);
    }

    Ok(final_result)
}

/// Hybrid approach with asynchronous parallelism using Tokio threads and `kanal` for communication:
/// Combines matrix exponentiation for boundary Fibonacci values and iteration, with
/// asynchronous tasks managed by Tokio and thread communication handled through the `kanal` library.
///
/// Parameters:
/// - `count`: The number of Fibonacci numbers to compute. Must be 2 or greater.
/// - `chunk_size`: Size of each chunk to be processed iteratively by separate asynchronous tasks.
/// - `max_concurrent_tasks`: Maximum number of concurrent asynchronous tasks to be executed at a time.
///
/// Returns:
/// - A vector containing the first `count` Fibonacci numbers, or an error if encountered.
///
/// # Panics
/// - This function will panic if `count` is less than 2.
pub async fn seq_hybrid_kanal_tokio(
    count: usize,
    chunk_size: usize,
    max_concurrent_tasks: usize,
) -> Result<Vec<BigUint>, FibonacciSequenceError> {
    assert!(count >= 2, "Count must be 2 or greater");
    let semaphore = Arc::new(Semaphore::new(max_concurrent_tasks));

    // Create a bounded Kanal channel to send the chunk results
    let (sender, receiver) = bounded::<(usize, Vec<BigUint>)>(count / chunk_size + 1);

    let mut tasks = vec![];

    for start in (2..count).step_by(chunk_size) {
        let end = std::cmp::min(start + chunk_size - 1, count - 1);
        let semaphore = Arc::clone(&semaphore);
        let sender = sender.clone();

        // Spawn a new task for each chunk
        let task: JoinHandle<Result<(), FibonacciSequenceError>> = task::spawn(async move {
            // Acquire a semaphore permit to control concurrency
            let _permit = semaphore.acquire().await?;

            // Compute F(start-1) and F(start) using matrix exponentiation
            let (f_start_minus_1, f_start) = fibonacci_matrix(start - 1);

            // Compute the Fibonacci numbers iteratively within the chunk
            let chunk = fibonacci_chunk(end - start + 1, f_start_minus_1, f_start);

            // Send the result along with the start index to the receiver
            sender.send((start, chunk))?;
            Ok(())
        });

        tasks.push(task);
    }

    // Drop the sender after spawning all tasks to signal that no more data will be sent
    drop(sender);

    // Wait for all tasks to complete
    for task in tasks {
        task.await??;
    }

    // Collect the results
    let mut results = Vec::new();
    for (start, chunk) in receiver {
        results.push((start, chunk));
    }

    // Sort the results by the starting index
    results.sort_by_key(|(start, _)| *start);

    // Append each chunk to the final result
    let mut final_result = vec![BigUint::zero(), BigUint::one()]; // Start with F(0), F(1)
    for (_, chunk) in results.into_iter() {
        final_result.extend(chunk);
    }

    Ok(final_result)
}

#[cfg(test)]
mod fibonacci_100000_tests {
    use super::*;

    #[test]
    fn test_fibonacci_basic_100000() {
        test_util::test_fibonacci_100000_sync(seq_basic)
    }

    #[test]
    fn test_fibonacci_hybrid_rayon_100000() {
        test_util::test_fibonacci_100000_sync(|n| seq_hybrid_rayon(n, 10000))
    }

    #[test]
    fn test_fibonacci_hybrid_100000() {
        test_util::test_fibonacci_100000_sync(|n| seq_hybrid(n, 10000))
    }

    #[tokio::test]
    async fn test_fibonacci_hybrid_tokio_100000() {
        test_util::test_fibonacci_100000_async(|n| seq_hybrid_tokio(n, 10000, 20)).await
    }

    #[test]
    fn test_fibonacci_hybrid_kanal_100000() {
        test_util::test_fibonacci_100000_sync(|n| seq_hybrid_kanal(n, 10000))
    }

    #[tokio::test]
    async fn test_fibonacci_hybrid_kanal_tokio_100000() {
        test_util::test_fibonacci_100000_async(|n| seq_hybrid_kanal_tokio(n, 10000, 20)).await
    }
}

#[cfg(test)]
mod fibonacci_20_tests {
    use super::*;

    #[test]
    fn test_fibonacci_basic_20() {
        test_util::test_fibonacci_20_sync(seq_basic)
    }

    #[test]
    fn test_fibonacci_hybrid_rayon_20() {
        test_util::test_fibonacci_20_sync(|n| seq_hybrid_rayon(n, 10000))
    }

    #[test]
    fn test_fibonacci_hybrid_20() {
        test_util::test_fibonacci_20_sync(|n| seq_hybrid(n, 10000))
    }

    #[tokio::test]
    async fn test_fibonacci_hybrid_tokio_20() {
        test_util::test_fibonacci_20_async(|n| seq_hybrid_tokio(n, 10000, 20)).await
    }

    #[test]
    fn test_fibonacci_hybrid_kanal_20() {
        test_util::test_fibonacci_20_sync(|n| seq_hybrid_kanal(n, 10000))
    }

    #[tokio::test]
    async fn test_fibonacci_hybrid_kanal_tokio_20() {
        test_util::test_fibonacci_20_async(|n| seq_hybrid_kanal_tokio(n, 10000, 20)).await
    }
}

#[cfg(test)]
mod fibonacci_panic_tests {
    use super::*;

    #[test]
    fn test_fibonacci_basic_panic() {
        test_util::test_fibonacci_panic_sync(seq_basic)
    }

    #[test]
    fn test_fibonacci_hybrid_rayon_panic() {
        test_util::test_fibonacci_panic_sync(|n| seq_hybrid_rayon(n, 10000))
    }

    #[test]
    fn test_fibonacci_hybrid_panic() {
        test_util::test_fibonacci_panic_sync(|n| seq_hybrid(n, 10000))
    }

    #[tokio::test]
    async fn test_fibonacci_hybrid_tokio_panic() {
        test_util::test_fibonacci_panic_async(|n| seq_hybrid_tokio(n, 10000, 20)).await
    }

    #[test]
    fn test_fibonacci_hybrid_kanal_panic() {
        test_util::test_fibonacci_panic_sync(|n| seq_hybrid_kanal(n, 10000))
    }

    #[tokio::test]
    async fn test_fibonacci_hybrid_kanal_tokio_panic() {
        test_util::test_fibonacci_panic_async(|n| seq_hybrid_kanal_tokio(n, 10000, 20)).await
    }
}

#[cfg(test)]
mod test_util {
    use byteorder::{BigEndian, ReadBytesExt};
    use num_bigint::BigUint;
    use std::collections::HashMap;
    use std::fs::File;
    use std::future::Future;
    use std::io::{self, BufReader, Read};
    use std::path::Path;

    use super::FibonacciSequenceError;

    pub async fn test_fibonacci_20_async<F, Fut>(fib_fn: F)
    where
        F: Fn(usize) -> Fut,
        Fut: Future<Output = Result<Vec<BigUint>, FibonacciSequenceError>>,
    {
        // Call the fibonacci function for n = 20
        let fib_sequence = fib_fn(20)
            .await
            .expect("Expect a fibonacci sequence to be generated");

        // Verify the result
        verify_fibonacci_20_sequence(fib_sequence);
    }

    pub fn test_fibonacci_20_sync<F, T>(fib_fn: F)
    where
        F: Fn(usize) -> T,
        T: FibonacciResult,
    {
        // Call the fibonacci function for n = 20
        let fib_sequence = fib_fn(20)
            .into_result()
            .expect("Expect to generate Fibonacci sequence");

        // Verify the result
        verify_fibonacci_20_sequence(fib_sequence);
    }

    fn verify_fibonacci_20_sequence(fib_sequence: Vec<BigUint>) {
        // The expected Fibonacci sequence for n = 20
        let expected_u32_sequence = vec![
            0u32, 1, 1, 2, 3, 5, 8, 13, 21, 34, 55, 89, 144, 233, 377, 610, 987, 1597, 2584, 4181,
        ];

        // Convert the u32 sequence into BigUint
        let expected_sequence: Vec<BigUint> =
            expected_u32_sequence.into_iter().map(Into::into).collect();

        // Verify that the computed sequence matches the expected sequence
        assert_eq!(
            fib_sequence, expected_sequence,
            "Computed sequence must match expected sequence"
        );
    }

    const N: usize = 99999;

    pub async fn test_fibonacci_100000_async<F, Fut>(fib_fn: F)
    where
        F: Fn(usize) -> Fut + Send,
        Fut: Future<Output = Result<Vec<BigUint>, FibonacciSequenceError>> + Send,
    {
        println!("Compute the Fibonacci sequence for n = {N}");
        let fib_sequence = fib_fn(N)
            .await
            .expect("expect to generate a fibonacci sequence");

        verify_fibonacci_10000_sequence(fib_sequence);
    }

    pub fn test_fibonacci_100000_sync<F, T>(fib_fn: F)
    where
        F: Fn(usize) -> T,
        T: FibonacciResult,
    {
        println!("Compute the Fibonacci sequence for n = {N}");
        let fib_sequence = fib_fn(N)
            .into_result()
            .expect("Expect to generate Fibonacci sequence");

        verify_fibonacci_10000_sequence(fib_sequence);
    }

    fn verify_fibonacci_10000_sequence(fib_sequence: Vec<BigUint>) {
        // Load Fibonacci numbers from the binary file
        check_fibonacci_data_file_exists();
        let fib_map = load_fibonacci_from_binary("fibonacci_data.bin")
            .expect("Failed to load Fibonacci data");

        println!("F({N}) = {}", fib_map[&N]);

        println!("Compare each computed Fibonacci number with the expected value");
        for (i, fib) in fib_sequence.iter().enumerate() {
            assert_eq!(fib, &fib_map[&i], "Mismatch at index {i}");
        }
    }

    pub trait FibonacciResult {
        fn into_result(self) -> Result<Vec<BigUint>, FibonacciSequenceError>;
    }

    impl FibonacciResult for Vec<BigUint> {
        fn into_result(self) -> Result<Vec<BigUint>, FibonacciSequenceError> {
            Ok(self)
        }
    }

    impl FibonacciResult for Result<Vec<BigUint>, FibonacciSequenceError> {
        fn into_result(self) -> Result<Vec<BigUint>, FibonacciSequenceError> {
            self
        }
    }

    // Function to load Fibonacci numbers from the binary file
    fn load_fibonacci_from_binary(file_path: &str) -> io::Result<HashMap<usize, BigUint>> {
        println!("Loading fibonacci sequence from file");
        let file = File::open(file_path)?;
        let mut reader = BufReader::new(file);
        let mut fib_map = HashMap::new();
        let mut index = 0;

        // Loop through the binary file
        while let Ok(length) = reader.read_u32::<BigEndian>() {
            // Read the length of the next number
            // Read the number itself based on the length
            let mut num_bytes = vec![0u8; length as usize];
            reader.read_exact(&mut num_bytes)?; // Ensure we read the exact number of bytes

            let fib_number = BigUint::from_bytes_be(&num_bytes);
            fib_map.insert(index, fib_number);
            index += 1;
        }

        println!("Finished loading fibonacci sequence from file");
        Ok(fib_map)
    }

    fn check_fibonacci_data_file_exists() {
        let file_path = Path::new("fibonacci_data.bin");
        if !file_path.exists() {
            println!("Error: The fibonacci binary file is missing.");
            println!("Please download it manually using the following command:");
            println!("wget -O fibonacci_data.bin \"https://www.dropbox.com/scl/fi/6zbvut3evptw1mlm2v12e/fibonacci_data.bin?rlkey=rurifdumjozw3x6ar4sr8nwn7&st=a0z85mgk&dl=1\"");

            // Fail the test if the file is missing
            panic!("Test cannot proceed without the binary file.");
        }
    }

    pub fn test_fibonacci_panic_sync<F, T>(fib_fn: F)
    where
        F: Fn(usize) -> T + std::panic::UnwindSafe + std::panic::RefUnwindSafe,
        T: FibonacciResult,
    {
        // Test for n = 0
        let result = std::panic::catch_unwind(|| {
            fib_fn(0);
        });
        assert!(result.is_err(), "Expected a panic for n = 0");

        // Test for n = 1
        let result = std::panic::catch_unwind(|| {
            fib_fn(1);
        });
        assert!(result.is_err(), "Expected a panic for n = 1");
    }

    pub async fn test_fibonacci_panic_async<F, Fut>(fib_fn: F)
    where
        F: Fn(usize) -> Fut
            + std::panic::UnwindSafe
            + std::panic::RefUnwindSafe
            + Clone
            + Send
            + Sync
            + 'static,
        Fut: Future<Output = Result<Vec<BigUint>, FibonacciSequenceError>> + Send + 'static,
    {
        // Test for n = 0
        let fib_fn_clone = fib_fn.clone(); // Clone fib_fn for reuse
        let result = tokio::spawn(async move { fib_fn_clone(0).await }).await;

        // Ensure the task panicked
        assert!(result.is_err(), "Expected a panic for n = 0");

        // Test for n = 1
        let result = tokio::spawn(async move { fib_fn(1).await }).await;

        // Ensure the task panicked
        assert!(result.is_err(), "Expected a panic for n = 1");
    }
}
