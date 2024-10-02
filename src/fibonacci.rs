use kanal::{bounded, SendError};
use num_bigint::BigUint;
use num_traits::{One, Zero};
use rayon::iter::{IndexedParallelIterator, IntoParallelIterator, ParallelIterator};
use std::thread;
use std::{cmp, sync::Arc};
use tokio::sync::{AcquireError, Semaphore};
use tokio::task::{self, JoinError, JoinHandle};

use crate::math::{matrix_pow, Matrix};

// Function to compute Fibonacci(n-1) and Fibonacci(n) using matrix exponentiation
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

// Iterative function to compute Fibonacci numbers within a chunk
pub fn fibonacci_chunk(start: usize, end: usize, f_n1: BigUint, f_n: BigUint) -> Vec<BigUint> {
    let mut fibs = Vec::with_capacity(end - start + 1);
    let mut a = f_n1;
    let mut b = f_n;

    for _ in start..=end {
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
/// - `n`: The number of Fibonacci numbers to compute.
///
/// # Returns
/// A vector containing the first `n` Fibonacci numbers.
///
/// # Example
/// ```
/// use fibonacci_assesment::fibonacci;
/// use num_bigint::BigUint;
/// let fib_sequence = fibonacci::seq_basic(10);
/// assert_eq!(fib_sequence[9], BigUint::from(34u32));
/// ```
pub fn seq_basic(n: usize) -> Vec<BigUint> {
    let mut fib_sequence = Vec::with_capacity(n + 1);
    fib_sequence.push(BigUint::zero());
    fib_sequence.push(BigUint::one());
    for i in 2..n {
        let next_value = &fib_sequence[i - 1] + &fib_sequence[i - 2];
        fib_sequence.push(next_value);
    }
    fib_sequence
}

// Hybrid approach: matrix exponentiation for boundaries, iterative chunking for Fibonacci numbers
pub fn seq_hybrid(limit: usize, chunk_size: usize) -> Vec<BigUint> {
    let mut result = vec![BigUint::zero(), BigUint::one()]; // Start with F(0), F(1)

    for start in (2..limit).step_by(chunk_size) {
        let end = std::cmp::min(start + chunk_size - 1, limit - 1);

        // Compute F(start-1) and F(start) using matrix exponentiation
        let (f_start_minus_1, f_start) = fibonacci_matrix(start - 1);

        // Compute the Fibonacci numbers iteratively within the chunk
        let chunk = fibonacci_chunk(start, end, f_start_minus_1, f_start);

        result.extend(chunk);
    }

    result
}

pub fn seq_hybrid_rayon(limit: usize, chunk_size: usize) -> Vec<BigUint> {
    let mut result = vec![BigUint::zero(), BigUint::one()]; // Start with F(0), F(1)

    // Create a parallel iterator over the start indices
    let chunks: Vec<Vec<BigUint>> = (2..limit)
        .into_par_iter()
        .step_by(chunk_size)
        .map(|start| {
            let end = std::cmp::min(start + chunk_size - 1, limit - 1);

            // Compute F(start-1) and F(start) using matrix exponentiation
            let (f_start_minus_1, f_start) = fibonacci_matrix(start - 1);

            // Compute the Fibonacci numbers iteratively within the chunk
            fibonacci_chunk(start, end, f_start_minus_1, f_start)
        })
        .collect();

    // Extend the result with all the chunks
    for chunk in chunks {
        result.extend(chunk);
    }

    result
}

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

pub async fn seq_hybrid_tokio(
    limit: usize,
    chunk_size: usize,
    max_concurrent_tasks: usize,
) -> Result<Vec<BigUint>, FibonacciSequenceError> {
    let semaphore = Arc::new(Semaphore::new(max_concurrent_tasks));

    // Pre-allocate the result vector with the correct size
    let mut result = vec![BigUint::zero(), BigUint::one()]; // Start with F(0), F(1)
    result.resize(limit, BigUint::zero());

    let mut tasks = vec![];

    for start in (2..limit).step_by(chunk_size) {
        let end = std::cmp::min(start + chunk_size - 1, limit - 1);
        let semaphore = Arc::clone(&semaphore);

        // Spawn a new task for each chunk
        let task: JoinHandle<Result<(usize, Vec<BigUint>), FibonacciSequenceError>> =
            task::spawn(async move {
                // Acquire a semaphore permit to control concurrency
                let _permit = semaphore.acquire().await?;

                // Compute F(start-1) and F(start) using matrix exponentiation
                let (f_start_minus_1, f_start) = fibonacci_matrix(start - 1);

                // Compute the Fibonacci numbers iteratively within the chunk
                let chunk = fibonacci_chunk(start, end, f_start_minus_1, f_start);

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

// Function to compute Fibonacci sequence in chunks, with matrix exponentiation for boundaries
pub fn seq_hybrid_kanal(
    limit: usize,
    chunk_size: usize,
) -> Result<Vec<BigUint>, FibonacciSequenceError> {
    // Create a channel to communicate between threads
    let (sender, receiver) = bounded::<(usize, Vec<BigUint>)>(limit / chunk_size + 1);

    // Vector to store join handles to propagate thread results/errors back to main thread
    let mut handles = Vec::new();

    for start in (2..limit).step_by(chunk_size) {
        let end = cmp::min(start + chunk_size - 1, limit - 1);

        let sender = sender.clone();
        let handle = thread::spawn(move || -> Result<(), FibonacciSequenceError> {
            // Compute boundary Fibonacci numbers for this chunk
            let (f_start_minus_1, f_start) = fibonacci_matrix(start - 1);

            // Compute the Fibonacci numbers for the chunk iteratively
            let chunk = fibonacci_chunk(start, end, f_start_minus_1, f_start);

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

    let mut final_result = vec![BigUint::zero(), BigUint::one()];

    // Collect the results in the correct order
    let mut results = vec![(0, vec![])]; // (start index, chunk)
    for (start, chunk) in receiver {
        results.push((start, chunk));
    }

    // Sort the results by the starting index and append each chunk to the final result
    results.sort_by_key(|(start, _)| *start);

    for (_, chunk) in results.into_iter().skip(1) {
        final_result.extend(chunk);
    }

    Ok(final_result)
}

pub async fn seq_hybrid_kanal_tokio(
    limit: usize,
    chunk_size: usize,
    max_concurrent_tasks: usize,
) -> Result<Vec<BigUint>, FibonacciSequenceError> {
    let semaphore = Arc::new(Semaphore::new(max_concurrent_tasks));
    let mut result = vec![BigUint::zero(), BigUint::one()]; // Start with F(0), F(1)

    // Create a bounded Kanal channel to send the chunk results
    let (sender, receiver) = bounded::<(usize, Vec<BigUint>)>(limit / chunk_size + 1);

    let mut tasks = vec![];

    for start in (2..limit).step_by(chunk_size) {
        let end = std::cmp::min(start + chunk_size - 1, limit - 1);
        let semaphore = Arc::clone(&semaphore);
        let sender = sender.clone();

        // Spawn a new task for each chunk
        let task: JoinHandle<Result<(), FibonacciSequenceError>> = task::spawn(async move {
            // Acquire a semaphore permit to control concurrency
            let _permit = semaphore.acquire().await?;

            // Compute F(start-1) and F(start) using matrix exponentiation
            let (f_start_minus_1, f_start) = fibonacci_matrix(start - 1);

            // Compute the Fibonacci numbers iteratively within the chunk
            let chunk = fibonacci_chunk(start, end, f_start_minus_1, f_start);

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

    // Collect the results in order
    let mut results = vec![(0, vec![])];
    while let Ok((start, chunk)) = receiver.recv() {
        results.push((start, chunk));
    }

    // Sort and combine chunks into the final result
    results.sort_by_key(|(start, _)| *start);
    for (_, chunk) in results.into_iter().skip(1) {
        result.extend(chunk);
    }

    Ok(result)
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
        // The expected Fibonacci sequence for n = 20
        let expected_u32_sequence = vec![
            0u32, 1, 1, 2, 3, 5, 8, 13, 21, 34, 55, 89, 144, 233, 377, 610, 987, 1597, 2584, 4181,
        ];

        // Convert the u32 sequence into BigUint
        let expected_sequence: Vec<BigUint> =
            expected_u32_sequence.into_iter().map(Into::into).collect();

        // Call the fibonacci function for n = 20
        let fib_sequence = fib_fn(20)
            .await
            .expect("expect a fibonacci sequence to be generated");

        assert_eq!(
            fib_sequence, expected_sequence,
            "Computed sequence must match expected sequence"
        );
    }

    pub fn test_fibonacci_20_sync<F, T>(fib_fn: F)
    where
        F: Fn(usize) -> T,
        T: FibonacciResult,
    {
        // The expected Fibonacci sequence for n = 20
        let expected_u32_sequence = vec![
            0u32, 1, 1, 2, 3, 5, 8, 13, 21, 34, 55, 89, 144, 233, 377, 610, 987, 1597, 2584, 4181,
        ];

        // Convert the u32 sequence into BigUint
        let expected_sequence: Vec<BigUint> =
            expected_u32_sequence.into_iter().map(Into::into).collect();

        // Call the Fibonacci function for n = 20
        let fib_sequence = fib_fn(20)
            .into_result()
            .expect("Expect to generate Fibonacci sequence");

        assert_eq!(
            fib_sequence, expected_sequence,
            "Computed sequence must match expected sequence"
        );
    }

    pub async fn test_fibonacci_100000_async<F, Fut>(fib_fn: F)
    where
        F: Fn(usize) -> Fut + Send,
        Fut: Future<Output = Result<Vec<BigUint>, FibonacciSequenceError>> + Send,
    {
        check_fibonacci_data_file_exists();
        let fib_map = load_fibonacci_from_binary("fibonacci_data.bin")
            .expect("Failed to load Fibonacci data");

        let n = 99999;
        println!("F({n}) = {}", fib_map[&n]);

        println!("Compute the Fibonacci sequence for n = {n}");
        let fib_sequence = fib_fn(n)
            .await
            .expect("expect to generate a fibonacci sequence");

        println!("Compare each computed Fibonacci number with the expected value");
        for (i, fib) in fib_sequence.iter().enumerate() {
            assert_eq!(fib, &fib_map[&i], "Mismatch at index {i}");
        }
    }

    pub fn test_fibonacci_100000_sync<F, T>(fib_fn: F)
    where
        F: Fn(usize) -> T,
        T: FibonacciResult,
    {
        // Load Fibonacci numbers from the binary file
        check_fibonacci_data_file_exists();
        let fib_map = load_fibonacci_from_binary("fibonacci_data.bin")
            .expect("Failed to load Fibonacci data");

        let n = 99999;
        println!("F({n}) = {}", fib_map[&n]);

        println!("Compute the Fibonacci sequence for n = {n}");
        let fib_sequence = fib_fn(n)
            .into_result()
            .expect("Expect to generate Fibonacci sequence");

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
}
