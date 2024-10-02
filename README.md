# Massively parallel Fibonacci

You are tasked with calculating the first 100000 fibonacci sequence numbers as fast as possible.
There is no *correct* way to solve to this problem. Abuse tokio, rayon, `std::thread`, channels, atomics, and whatever you think is the right tool for the job!   

The only requirement is that the sequence is mathematically accurate!   

There is an included criterion benchmark for your performance testing needs.



## Testing

To verify the correctness of generating the first 100,000 Fibonacci sequence numbers, you will need
to download and load a binary file containing this sequence, named `fibonacci_data.bin`. This file
can be downloaded from the root directory of the project using the following command:

```sh
wget -O fibonacci_data.bin "https://www.dropbox.com/scl/fi/6zbvut3evptw1mlm2v12e/fibonacci_data.bin?rlkey=rurifdumjozw3x6ar4sr8nwn7&st=a0z85mgk&dl=1"
```

Once downloaded, you can run `cargo test`, which will verify that the Fibonacci sequence generation
methods provided in this repository are correct.

