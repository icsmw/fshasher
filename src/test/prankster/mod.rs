use crate::{
    collector::Tolerance, entry::Entry, error::E, hasher, reader, test::usecase::*, Options,
    ReadingStrategy,
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
        .walker(
            hasher::blake::Blake::new(),
            reader::buffering::Buffering::default(),
        )?;
    assert!(!walker.collect().unwrap().hash().unwrap().is_empty());
    usecase.clean()?;
    Ok(())
}

#[test]
fn with_custom_number_threads() -> Result<(), E> {
    let usecase = UseCase::unnamed(2, 2, 2, &[])?;
    let mut walker = Options::new()
        .entry(Entry::from(&usecase.root)?)?
        .threads(5)?
        .walker(
            hasher::blake::Blake::new(),
            reader::buffering::Buffering::default(),
        )?;
    assert!(!walker.collect().unwrap().hash().unwrap().is_empty());
    usecase.clean()?;
    Ok(())
}

#[test]
fn bad_options_no_threads() -> Result<(), E> {
    let usecase = UseCase::unnamed(2, 2, 2, &[])?;
    let mut opt = Options {
        entries: vec![Entry::from(&usecase.root)?],
        threads: Some(0),
        tolerance: Tolerance::LogErrors,
        progress: None,
        reading_strategy: ReadingStrategy::Buffer,
        global: Entry::new(),
    };
    let mut walker = opt.walker(
        hasher::blake::Blake::new(),
        reader::buffering::Buffering::default(),
    )?;
    assert!(walker.collect().is_err());
    assert!(walker.hash().unwrap().is_empty());
    usecase.clean()?;
    Ok(())
}

#[test]
fn bad_options_too_many_threads() -> Result<(), E> {
    let usecase = UseCase::unnamed(2, 2, 2, &[])?;
    let mut opt = Options {
        entries: vec![Entry::from(&usecase.root)?],
        threads: Some(10000),
        tolerance: Tolerance::LogErrors,
        progress: None,
        reading_strategy: ReadingStrategy::Buffer,
        global: Entry::new(),
    };
    let mut walker = opt.walker(
        hasher::blake::Blake::new(),
        reader::buffering::Buffering::default(),
    )?;
    assert!(walker.collect().is_err());
    assert!(walker.hash().unwrap().is_empty());
    usecase.clean()?;
    Ok(())
}