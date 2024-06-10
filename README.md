`fshasher` allows quickly calculating a common hash for all files in a target folder (recursively).

# What it does?

`fshasher` actually does only two jobs:
- **collecting**: collecting paths to files from the target folder
- **hashing**: calculating hashes for each file and a common hash for all of them

# Features
- Because `fshasher` spawns multiple threads for collecting files and further hashing, it works quite fast; however, the performance depends on the file system and CPU performance (and the number of cores).
- `fshasher` has flexible configuration, which helps to find the best compromise between performance and loading the CPU and file system. For example, different ways to read files can be defined for different sizes (chunk by chunk, complete reading, or mapping the file into memory). `fshasher` also introduces traits `Reader` and `Hasher` to allow implementing custom readers and hashers.
- `fshasher` supports filtering files and folders. It allows including only necessary files into the hash or excluding others. Filtering works based on `glob` patterns.
- `fshasher` performs expensive and continuous operations like hashing and, of course, allows aborting/canceling collecting and hashing operations.
- `fshasher` has an embedded channel to share the progress of collecting files and further hashing.
- `fshasher` supports different levels of tolerance to errors; it allows safely skipping the processing of some files (for example, due to permission issues) and still getting the hash of the remaining files.

# Where it can be useful?

General use-cases for using `fshasher` can be:
- For build scripts/tasks. To reduce unnecessary build steps, you can check for changes in a folder and decide whether to perform certain build actions.
- For tracking changes. To quickly detect changes in target folders and trigger necessary actions.
- For any other actions that depend on the state of files in a target folder.

# Basic example of usage

```
use fshasher::{Options, Entry, Tolerance, hasher, reader};
use std::env::temp_dir;

let mut walker = Options::new()
    .entry(Entry::from(temp_dir()).unwrap()).unwrap()
    .tolerance(Tolerance::LogErrors)
    .walker(
        hasher::blake::Blake::default(),
        reader::buffering::Buffering::default(),
    ).unwrap();

println!("Hash of {}: {:?}", temp_dir().display(), walker.collect().unwrap().hash().unwrap())
```

# Behaviour & error handeling
