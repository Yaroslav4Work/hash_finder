# Hash Finder

## Description

Implementation of an application that calculates SHA-256 hashes for consecutive integers starting with 1. The program
prints the hash and a number if the hash ends with N zeros.

---

## Build and launch

### Installation (Downloading) on Linux

**Requires git**

1. Open the terminal and go to the directory we need: `cd /home/{username}`;
2. Clone the repository: `git clone https://github.com/Yaroslav4Work/hash_finder`;
3. Let's go to the repository; `cd ./hash_finder`.

### Build on Linux

**Requires rustc and cargo**

1. We are compiling the project into a release version: `cargo build --release`;
2. Making the file executable: `chmod +x ./target/release/hash_finder`;

### Launch

1. Run the compiled application: `./target/release/hash_finder -N {u8} -F {u32}`;
2. **Don't forget to pass compatible parameters!**.

---

## My thoughts

1. For the sake of interest, 3 application variants were prepared: single-threaded, asynchronous (concurrent) and
   multi-threaded;
2. The variants are implemented in src/lib.rs;
3. The executable file uses a multi-threaded version;
4. The multithreaded version was implemented using channels (synchronization primitives) rather than using mutexes and
   an atomic reference counter, since the total locking time when implemented with mutexes is significantly longer than
   the execution time of the single-threaded code (including attempts to implement this using non-blocking try_lock());
5. I'm not sure that the multithreaded implementation is completely correct;
6. The asynchronous variant uses a multi-threaded execution environment;
7. I think the async variant could be improved a bit by switching to a blocking wait variant (try_recv), setting up a
   sufficient channel buffer, and changing the priority of sending numbers to the channel over receiving results from
   the channel;
8. Yes, I understand that asynchrony (concurrency) is better for IO-bound, and multithreading is better for CPU-bound.
9. The output of results was implemented in the same thread (which is not very good, from the point of view of the
   implementation of the library crate), since it seemed redundant to me to make an observer (subscription mechanism)
   for the test task, I apologize for my laziness;
10. I also created a benchmark (`cargo bench`), but it doesn't capture the essence of the problem due to the standard
    execution time constraints, the use of the `criterion` runtime, and a lot of unnecessary console output. I might fix
    this in the future;
11. I also abandoned the tests due to uncontrolled println output. This could probably be fixed by rewriting println to
    write or creating an external channel to intercept messages, but I think I demonstrated my multithreaded programming
    skills for this task.

**Thank you for reading. I would greatly appreciate any objective criticism and suggestions for improvement.**