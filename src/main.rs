use std::env;

#[tokio::main]
async fn main() {
    let raw_args = &env::args().collect::<Vec<String>>()[1..];
    let args = raw_args.chunks_exact(2).map(|chunk| chunk.to_vec());

    if args.len() < 2 {
        panic!(
            "Not enough required arguments passed. Passed: {}, expected: -N u8 -F u32",
            raw_args.join(" ")
        );
    }

    let mut n: Option<u8> = None;
    let mut f: Option<u32> = None;

    for arg in args {
        if arg.len() < 2 {
            panic!("Empty argument: {}", arg[0]);
        }

        match &*arg[0] {
            "-N" => {
                n = Some(
                    arg[1]
                        .parse::<u8>()
                        .map_err(
                            |_| "The value of the argument -N must be compatible with the u8 type",
                        )
                        .unwrap(),
                )
            }
            "-F" => {
                f = Some(
                    arg[1]
                        .parse::<u32>()
                        .map_err(
                            |_| "The value of the argument -F must be compatible with the u8 type",
                        )
                        .unwrap(),
                )
            }
            other => panic!("Unknown argument: {}", other),
        }
    }

    if let (Some(n), Some(f)) = (n, f) {
        let hash_finder = hash_finder::multithread_impl::HashFinder::new(10, n, f);
        hash_finder.run().unwrap();
    } else {
        panic!("The -N or -F arguments were not set.");
    }
}
