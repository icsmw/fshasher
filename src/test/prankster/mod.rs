use std::thread;

use crate::{
    collector::Tolerance, entry::Entry, hasher, reader, test::usecase::*, Options, ReadingStrategy,
    E,
};

#[test]
fn threads_opt_min() -> Result<(), E> {
    assert!(Options::new().threads(0).is_err());
    Ok(())
}

#[test]
fn threads_opt_max() -> Result<(), E> {
    assert!(Options::new().threads(10000).is_err());
    Ok(())
}

#[test]
fn with_one_thread() -> Result<(), E> {
    let usecase = UseCase::unnamed(2, 2, 2, &[])?;
    let mut walker = Options::new()
        .entry(Entry::from(&usecase.root)?)?
        .threads(1)?
        .walker()?;
    assert!(!walker
        .collect()
        .unwrap()
        .hash::<hasher::blake::Blake, reader::buffering::Buffering>()
        .unwrap()
        .is_empty());
    usecase.clean()?;
    Ok(())
}

#[test]
fn with_custom_number_threads() -> Result<(), E> {
    let usecase = UseCase::unnamed(2, 2, 2, &[])?;
    let cores = thread::available_parallelism()
        .ok()
        .map(|n| n.get())
        .unwrap();
    let mut walker = Options::new()
        .entry(Entry::from(&usecase.root)?)?
        .threads(cores)?
        .walker()?;
    assert!(!walker
        .collect()
        .unwrap()
        .hash::<hasher::blake::Blake, reader::buffering::Buffering>()
        .unwrap()
        .is_empty());
    usecase.clean()?;
    Ok(())
}

#[test]
fn bad_options_no_threads() -> Result<(), E> {
    let usecase = UseCase::unnamed(2, 2, 2, &[])?;
    let opt = Options {
        entries: vec![Entry::from(&usecase.root)?],
        threads: Some(0),
        tolerance: Tolerance::LogErrors,
        progress: None,
        reading_strategy: ReadingStrategy::Buffer,
        global: Entry::new(),
    };
    let mut walker = opt.walker()?;
    assert!(walker.collect().is_err());
    assert!(walker
        .hash::<hasher::blake::Blake, reader::buffering::Buffering>()
        .unwrap()
        .is_empty());
    usecase.clean()?;
    Ok(())
}

#[test]
fn bad_options_too_many_threads() -> Result<(), E> {
    let usecase = UseCase::unnamed(2, 2, 2, &[])?;
    let opt = Options {
        entries: vec![Entry::from(&usecase.root)?],
        threads: Some(10000),
        tolerance: Tolerance::LogErrors,
        progress: None,
        reading_strategy: ReadingStrategy::Buffer,
        global: Entry::new(),
    };
    let mut walker = opt.walker()?;
    assert!(walker.collect().is_err());
    assert!(walker
        .hash::<hasher::blake::Blake, reader::buffering::Buffering>()
        .unwrap()
        .is_empty());
    usecase.clean()?;
    Ok(())
}
