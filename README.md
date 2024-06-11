`fshasher` allows quickly calculating a common hash for all files in a target folder (recursively).

# Introducing

## What it does?

`fshasher` actually does only two jobs:
- **collecting**: collecting paths to files from the target folder
- **hashing**: calculating hashes for each file and a common hash for all of them

## Features
- Because `fshasher` spawns multiple threads for collecting files and further hashing, it works quite fast; however, the performance depends on the file system and CPU performance (and the number of cores).
- `fshasher` has flexible configuration, which helps to find the best compromise between performance and loading the CPU and file system. For example, different ways to read files can be defined for different sizes (chunk by chunk, complete reading, or mapping the file into memory). `fshasher` also introduces traits `Reader` and `Hasher` to allow implementing custom readers and hashers.
- `fshasher` supports filtering files and folders. It allows including only necessary files into the hash or excluding others. Filtering works based on `glob` patterns.
- `fshasher` performs expensive and continuous operations like hashing and, of course, allows aborting/canceling collecting and hashing operations.
- `fshasher` has an embedded channel to share the progress of collecting files and further hashing.
- `fshasher` supports different levels of tolerance to errors; it allows safely skipping the processing of some files (for example, due to permission issues) and still getting the hash of the remaining files.

## Where it can be useful?

General use-cases for using `fshasher` can be:
- For build scripts/tasks. To reduce unnecessary build steps, you can check for changes in a folder and decide whether to perform certain build actions.
- For tracking changes. To quickly detect changes in target folders and trigger necessary actions.
- For any other actions that depend on the state of files in a target folder.

# Configuration

## General

To configure `fshasher` should be used `Options` struct. It has a couple of useful methods:

- `reading_strategy(ReadingStrategy)` - set reading strategy
- `threads(usize)` - set number of system thread, which collector and hasher can spawn (default value is equal to number of cores)
- `progress(usize)` - activate progress tracking; as argument you can define a capacity of a channel queue. 
- `tolerance(Tolerance)` - tolerance to errors; by defaul collector and hasher will not stop working on errors, but will report it.
- `path(AsRef<Path>)` - adds dest folder to be included into hashing; includes folder without filtering.
- `entry(Entry)` - adds dest folder to be included into hashing; includes folder with filtering.
- `include(Filter)` - adds global positive filter for all entries
- `exclude(Filter)` - adds global negative filter for all entries

## Filtering

To setup global filters, which will be applied to any entries, should be used `Options.include(Filter)` and `Options.exlude(Filter)` to set positive and/or negative filters. For filtering `fshasher` is using `glob` patterns.

Next example:
 - as entry paths takes: "/music/2023" and "/music/2024" 
 - include files, which has in name "star" and has extention "flac"
 - ingore files if they located in folder, which has in name "Bieber"

```ignore
    let walker = Options::new()
        .path("/music/2023")?
        .path("/music/2024")?
        .include(Filter::Files("*star*"))?
        .include(Filter::Files("*.flac"))?
        .exclude("*Bieber*")?.
        .walker(..)?;
```

With `Filter`, a glob pattern can be applied to a file's name or a folder's name only, whereas a regular glob pattern is applied to the full path. This allows for more accurate filtering.

- `Filter::Folders(AsRef<str>)` - A glob pattern that will be applied to a folder's name only.
- `Filter::Files(AsRef<str>)` - A glob pattern that will be applied to a file's name only.
- `Filter::Common(AsRef<str>)` - A glob pattern that will be applied to the full path (regular usage of glob patterns).

To create filter liked to entry, can be used `Entry`.

Next example:
 - as entry paths takes: "/music/2023" and "/music/2024" 
 - include files, which has in name "star" and has extention "flac" in both entries
 - ingore files from "/music/2023" if they located in folder, which has in name "Bieber"
 - ingore files from "/music/2024" if they located in folder, which has in name "Taylor Swift"

```ignore
    let music_2023 = Entry::from("music/2023")?.exclude(Filter::Folders("*Bieber*"))?;
    let music_2024 = Entry::from("music/2023")?.exclude(Filter::Folders("*Taylor Swift*"))?;
    let walker = Options::new()
        .entry(music_2023)?
        .entry(music_2024)?
        .include(Filter::Files("*star*"))?
        .include(Filter::Files("*.flac"))?
        .walker(..);
```


# Extending: hasher & reader

# Examples

## Basic example of usage

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

# Behaviour, errors, logs

## Error handeling

Hashing of big massive of files can be in some situation unpredictable. For example some permissions issue can cause errors; or in case if folder content has been changed right during calculation of hash. `fshasher` gives a controll of tolerancety to errors. It has next levels:

- `Tolerance::LogErrors`: Errors will be logged, but the collecting and hashing process will not be stopped.
- `Tolerance::DoNotLogErrors`: Errors will be ignored, and the collecting and hashing process will not be stopped.
- `Tolerance::StopOnErrors`: The collecting and hashing process will stop on any IO errors or errors related to hasher or reader.

## Why errors can be ignored?

If some files cause parmission errors, it isn't a "problem" of files collector, because a collector works in given context with given rights. If user calculating hash of some folder, which includes not permitted sub folders, it might be a choose of user.

Another situation - a list of collected files has been changed during hash calculating. In this case `hash()` function still can return a hash, which will reflect changes in anyway (for example if some file(s) has been removed).

The list of files, which caused errors will be available in `Walker`, folder `ignored`;

But at the end - ignore errors or not up to developer choose only.

## Logs
`fshasher` uses `log` crate, a lightweight logging facade for Rust. `log` is used with `env_logger` in pair. Well next shell command will make some logs visible to you.

```sh
export RUST_LOG=debug
```  
