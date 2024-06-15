[![LICENSE](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](LICENSE.txt)
[![](https://github.com/icsmw/fshasher/actions/workflows/push_and_pr_master.yml/badge.svg)](https://github.com/icsmw/fshasher/actions/workflows/push_and_pr_master.yml)
![Crates.io](https://img.shields.io/crates/v/fshasher)

`fshasher` allows for quickly calculating a common hash for all files in a target folder (recursively).

# Table of Contents

1. [Introduction](#introduction)

-   [What does it do](#what-does-it-do)
-   [Features](#features)
-   [Where can it be useful?](#where-can-it-be-useful)
-   [Basic example of usage](#basic-example-of-usage)

2. [Configuration](#configuration)

-   [General](#general)
-   [Filtering](#filtering)
-   [Patterns](#patterns)
-   [Reading Strategy](#reading-strategy)

3. [Extending: Hasher & Reader](#extending-hasher-reader)

4. [Behaviour, Errors, Logs](#behaviour-errors-logs)

-   [Error Handling](#error-handling)
-   [Why Errors Can Be Ignored?](#why-errors-can-be-ignored)
-   [Logs](#logs)

# Introduction

## What does it do?

`fshasher` performs two primary tasks:
- **Collecting**: Gathering paths to files from the target folder.
- **Hashing**: Calculating hashes for each file and a common hash for all of them.

## Features

- `fshasher` spawns multiple threads for collecting files and further hashing, resulting in high speed; however, the performance depends on the file system and CPU performance (including the number of cores).
- `fshasher` offers flexible configuration, allowing users to find the best compromise between performance and CPU/file system load. Different methods for reading files can be defined based on their sizes (chunk by chunk, complete reading, or memory-mapped files). `fshasher` also introduces the `Reader` and `Hasher` traits for implementing custom readers and hashers.
- `fshasher` supports filtering files and folders, allowing the inclusion of only necessary files in the hash or the exclusion of others. Filtering is based on `glob` patterns.
- `fshasher` performs expensive and continuous operations like hashing and allows for aborting/canceling collecting and hashing operations.
- `fshasher` includes an embedded channel to share the progress of collecting files and hashing.
- `fshasher` supports different levels of error tolerance, enabling the safe skipping of some files (e.g., due to permission issues) while still obtaining the hash of the remaining files.

## Where can it be useful?

General use cases for `fshasher` include:
- **Build scripts/tasks**: To reduce unnecessary build steps by checking for changes in a folder and deciding whether to perform certain build actions.
- **Tracking changes**: To quickly detect changes in target folders and trigger necessary actions.
- **Other use cases**: Any actions that depend on the state of files in a target folder.

## Basic example of usage

```
use fshasher::{Options, Entry, Tolerance, hasher, reader};
use std::env::temp_dir;
///
let mut walker = Options::new()
    .entry(Entry::from(temp_dir()).unwrap()).unwrap()
    .tolerance(Tolerance::LogErrors)
    .walker().unwrap();
let hash = walker.collect().unwrap()
    .hash::<hasher::blake::Blake, reader::buffering::Buffering>().unwrap();
println!("Hash of {}: {:?}", temp_dir().display(), hash);
```

# Configuration

## General

To configure `fshasher`, use the `Options` struct. It provides several useful methods:

- `reading_strategy(ReadingStrategy)` - Sets the reading strategy.
- `threads(usize)` - Sets the number of system threads that the collector and hasher can spawn (default value is equal to the number of cores).
- `progress(usize)` - Activates progress tracking; as an argument, you can define the capacity of the channel queue.
- `tolerance(Tolerance)` - Sets tolerance to errors; by default, the collector and hasher will not stop working on errors but will report them.
- `path(AsRef<Path>)` - Adds a destination folder to be included in hashing; includes the folder without filtering.
- `entry(Entry)` - Adds a destination folder to be included in hashing; includes the folder with filtering.
- `include(Filter)` - Adds a global positive filter for all entries.
- `exclude(Filter)` - Adds a global negative filter for all entries.

## Filtering

To set up global filters, which will be applied to all entries, use `Options.include(Filter)` and `Options.exclude(Filter)` to set positive and/or negative filters. For filtering, `fshasher` uses `glob` patterns.

The following example:
 - Includes entry paths: "/music/2023" and "/music/2024".
 - Includes files with "star" in the name and with the "flac" extension.
 - Ignores files located in folders that have "Bieber" in the name.


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

To create a filter linked to an entry, use `Entry`.

The following example:
 - Takes entry paths: "/music/2023" and "/music/2024".
 - Includes files that have "star" in the name and have the extension "flac" in both entries.
 - Ignores files from "/music/2023" if they are located in folders that have "Bieber" in the name.
 - Ignores files from "/music/2024" if they are located in folders that have "Taylor Swift" in the name.

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

> **Note**: Exclude `Filter` has priority over include `Filter`. If an exclude `Filter` matches, the include `Filter` will not be checked.

## Patterns

While `Filter` applies a `glob` pattern specifically to the filename or filepath, `PatternFilter` applies a `glob` pattern to the full path (filename including path), i.e., in the regular way of using `glob` patterns.

- `PatternFilter::Ignore(AsRef<str>)` - If the given glob pattern matches, the path will be ignored.
- `PatternFilter::Accept(AsRef<str>)` - If the given glob pattern matches, the path will be included.
- `PatternFilter::Cmb(Vec<PatternFilter<AsRef<str>>>)` - Allows defining a combination of `PatternFilter`. `PatternFilter::Cmb(..)` doesn't support nested combinations; attempting to nest another `PatternFilter::Cmb(..)` inside will cause an error.

The following example:
 - Takes entry paths: "/music/2023" and "/music/2024".
 - Includes files with the extension "flac" **OR** "mp3" in both entries.
 - Ignores files from "/music/2023" if the full filename contains "Bieber".
 - Ignores files from "/music/2024" if the full filename contains "Taylor Swift".

```ignore
    let music_2023 = Entry::from("music/2023")?
        .pattern(PatternFilter::Accept("*.flac"))?
        .pattern(PatternFilter::Accept("*.mp3"))?;
    let music_2024 = Entry::from("music/2023")?
        .pattern(PatternFilter::Accept("*.flac"))?
        .pattern(PatternFilter::Accept("*.mp3"))?;
    let walker = Options::new()
        .entry(music_2023)?
        .entry(music_2024)?
        .walker(..);
```

> **Note** `PatternFilter` has higher priority to `Filter`. If `PatternFilter` has been defined, any `Filter` will be ignored.

One more variant of `PatternFilter` is `PatternFilter::Cmb(Vec<PatternFilter<AsRef<str>>>)`. You can use it to combine `PatternFilter` with condition `AND`.

Next example:
 - as entry paths takes: "/music/2023" and "/music/2024";
 - collect files from "/music/2023" files with extention "flac" **AND** if full filename has not "Bieber";
 - collect files from "/music/2024" files with extention "flac" **AND** if full filename has not "Taylor Swift";

```ignore
    let music_2023 = Entry::from("music/2023")?.pattern(PatternFilter::Cmb(vec![
        PatternFilter::Accept("*.flac")?,
        PatternFilter::Ignore("*Bieber*")?,
    ]));
    let music_2024 = Entry::from("music/2023")?.pattern(PatternFilter::Cmb(vec![
        PatternFilter::Accept("*.flac")?,
        PatternFilter::Ignore("*Taylor Swift*")?,
    ]));
    let walker = Options::new()
        .entry(music_2023)?
        .entry(music_2024)?
        .walker(..);
```

## Reading Strategy

Configuring a reading strategy helps optimize the hashing process to match a specific system's capabilities. On the one hand, the faster a file is read, the sooner its hashing can begin. On the other hand, hashing too much data at once can reduce performance or overload the CPU. To find a balance, the `ReadingStrategy` can be used.

- `ReadingStrategy::Buffer` - Each file will be read in the "classic" way using a limited size buffer, chunk by chunk until the end. The hasher will receive small chunks of data to calculate the hash of the file. This strategy doesn't load the CPU much, but it entails many IO operations.
- `ReadingStrategy::Complete` - With this strategy, the file will be read first, and the complete file's content will be passed to the hasher to calculate the hash. This strategy involves fewer IO operations but loads the CPU more.
- `ReadingStrategy::MemoryMapped` - Instead of reading the file traditionally, this strategy maps the file into memory and provides the full content to the hasher.
- `ReadingStrategy::Scenario(Vec<(Range<u64>, Box<ReadingStrategy>)>)` - The scenario strategy allows combining different strategies based on the file's size.

In the following example:
- Use the `ReadingStrategy::MemoryMapped` strategy for files smaller than 1024KB.
- Use the `ReadingStrategy::Buffer` strategy for files larger than 1024KB.

```
    use fshasher::{collector::Tolerance, hasher, reader, Options, ReadingStrategy};
    use std::env::temp_dir;

    let mut walker = Options::from(temp_dir())
        .unwrap()
        .reading_strategy(ReadingStrategy::Scenario(vec![
            (0..1024 * 1024, Box::new(ReadingStrategy::MemoryMapped)),
            (1024 * 1024..u64::MAX, Box::new(ReadingStrategy::Buffer)),
        ]))
        .unwrap()
        .tolerance(Tolerance::LogErrors)
        .walker()
        .unwrap();
    let hash = walker.collect()
        .unwrap()
        .hash::<hasher::blake::Blake, reader::mapping::Mapping>()
        .unwrap()
        .to_vec();
    assert!(!hash.is_empty());
```

> **Note**: There is a very small chance to find a way to increase performance using `ReadingStrategy`, but in terms of CPU load, the difference can be quite significant.

# Extending: Hasher & Reader

Implementing a custom `hasher` can be achieved by implementing the `Hasher: Send + Sync` trait. Similarly, implementing a custom `reader` requires the implementation of the `Reader: Send + Sync` trait.

Out of the box, `fshasher` includes the following readers:

- `reader::buffering::Buffering` - A "classic" reader that reads the file chunk by chunk until the end. It doesn't support mapping the file into memory (cannot be used with `ReadingStrategy::MemoryMapped`).
- `reader::mapping::Mapping` - Supports mapping the file into memory (can be used with `ReadingStrategy::MemoryMapped`) and "classic" reading chunk by chunk until the end of the file.
- `reader::md::Md` - Instead reading of file, this reader creates a byte slice with date of last modification of file and size. Obviously, this reader will give very fast results, but it should be used only if you are sure checking the metadata would be enough to make the right conclusion.

`fshasher` includes only one hasher out of the box:

- `hasher::blake::Blake` - A hasher based on the `blake3` crate.

# Behaviour, Errors, Logs

## Error Handling

Hashing a large number of files can be unpredictable in some situations. For example, permission issues can cause errors, or a folder's content might change during the hash calculation. `fshasher` provides control over the tolerance to errors. It has the following levels:

- `Tolerance::LogErrors`: Errors will be logged, but the collecting and hashing process will not be stopped.
- `Tolerance::DoNotLogErrors`: Errors will be ignored, and the collecting and hashing process will not be stopped.
- `Tolerance::StopOnErrors`: The collecting and hashing process will stop on any IO errors or errors related to the hasher or reader.

## Why Errors Can Be Ignored?

If some files cause permission errors, it isn't a "problem" of the file collector, as the collector works in the given context with the given rights. If a user calculates the hash of a folder that includes subfolders without proper permissions, it might be the user's choice.

Another situation is when the list of collected files changes during hash calculation. In this case, the `hash()` function can still return a hash that reflects the changes in any way (for example, if some file(s) have been removed).

Meanwhile, the list of files that caused errors will be available in the `Walker` field `ignored`.

Ultimately, whether to ignore errors or not is up to the developer's choice.

## Logs

`fshasher` uses the `log` crate, a lightweight logging facade for Rust. `log` is used in conjunction with `env_logger`. The following shell command will make some logs visible to you:

```sh
export RUST_LOG=debug
```  

# Contributing

Contributions are welcome! Please read the short [Contributing Guide](CONTRIBUTING.md).
