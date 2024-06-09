mod cancellation;
mod progress;
mod stratagies;
use crate::{
    collector::Tolerance, entry::Entry, error::E, hasher, reader, test::usecase::*, Options,
};

const STRESS_TEST_ITERATIONS_COUNT: usize = 100;

#[test]
fn correction() -> Result<(), E> {
    let usecase = UseCase::unnamed(5, 10, 3, &["aaa", "bbb", "ccc"])?;
    let mut walker_a = Options::new()
        .entry(Entry::from(&usecase.root)?)?
        .tolerance(Tolerance::LogErrors)
        .walker(
            hasher::blake::Blake::new(),
            reader::buffering::Buffering::default(),
        )?;
    let hash_a = walker_a.collect()?.hash()?.to_vec();
    let mut walker_b = Options::new()
        .entry(Entry::from(&usecase.root)?)?
        .tolerance(Tolerance::LogErrors)
        .walker(
            hasher::blake::Blake::new(),
            reader::buffering::Buffering::default(),
        )?;
    let hash_b = walker_b.collect()?.hash()?.to_vec();
    assert_eq!(walker_a.count(), usecase.files.len());
    assert_eq!(walker_b.count(), usecase.files.len());
    assert_eq!(hash_a, hash_b);
    usecase.clean()?;
    Ok(())
}

#[test]
fn stress() -> Result<(), E> {
    let usecase = UseCase::unnamed(5, 10, 3, &["aaa", "bbb", "ccc"])?;
    for _ in 0..STRESS_TEST_ITERATIONS_COUNT {
        let mut walker_a = Options::new()
            .entry(Entry::from(&usecase.root)?)?
            .tolerance(Tolerance::LogErrors)
            .walker(
                hasher::blake::Blake::new(),
                reader::buffering::Buffering::default(),
            )?;
        let hash_a = walker_a.collect()?.hash()?.to_vec();
        let mut walker_b = Options::new()
            .entry(Entry::from(&usecase.root)?)?
            .tolerance(Tolerance::LogErrors)
            .walker(
                hasher::blake::Blake::new(),
                reader::buffering::Buffering::default(),
            )?;
        let hash_b = walker_b.collect()?.hash()?.to_vec();
        assert_eq!(walker_a.count(), usecase.files.len());
        assert_eq!(walker_b.count(), usecase.files.len());
        assert_eq!(hash_a, hash_b);
    }
    usecase.clean()?;
    Ok(())
}

#[test]
fn changes() -> Result<(), E> {
    let usecase = UseCase::unnamed(5, 10, 3, &["aaa", "bbb", "ccc"])?;
    let mut walker_a = Options::new()
        .entry(Entry::from(&usecase.root)?)?
        .tolerance(Tolerance::LogErrors)
        .walker(
            hasher::blake::Blake::new(),
            reader::buffering::Buffering::default(),
        )?;
    let hash_a = walker_a.collect()?.hash()?.to_vec();
    usecase.change()?;
    let mut walker_b = Options::new()
        .entry(Entry::from(&usecase.root)?)?
        .tolerance(Tolerance::LogErrors)
        .walker(
            hasher::blake::Blake::new(),
            reader::buffering::Buffering::default(),
        )?;
    let hash_b = walker_b.collect()?.hash()?.to_vec();
    assert_eq!(walker_a.count(), usecase.files.len());
    assert_eq!(walker_b.count(), usecase.files.len());
    assert_ne!(hash_a, hash_b);
    usecase.clean()?;
    Ok(())
}

#[test]
fn changes_stress() -> Result<(), E> {
    let usecase = UseCase::unnamed(5, 10, 3, &["aaa", "bbb", "ccc"])?;
    for _ in 0..STRESS_TEST_ITERATIONS_COUNT {
        let mut walker_a = Options::new()
            .entry(Entry::from(&usecase.root)?)?
            .tolerance(Tolerance::LogErrors)
            .walker(
                hasher::blake::Blake::default(),
                reader::buffering::Buffering::default(),
            )?;
        let hash_a = walker_a.collect()?.hash()?.to_vec();
        usecase.change()?;
        let mut walker_b = Options::new()
            .entry(Entry::from(&usecase.root)?)?
            .tolerance(Tolerance::LogErrors)
            .walker(
                hasher::blake::Blake::new(),
                reader::buffering::Buffering::default(),
            )?;
        let hash_b = walker_b.collect()?.hash()?.to_vec();
        assert_eq!(walker_a.count(), usecase.files.len());
        assert_eq!(walker_b.count(), usecase.files.len());
        assert_ne!(hash_a, hash_b);
    }
    usecase.clean()?;
    Ok(())
}
