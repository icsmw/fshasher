mod cancellation;
mod changed_dest;
mod progress;
mod stratagies;

use std::env::temp_dir;

use crate::{
    collector::Tolerance,
    entry::Entry,
    error::E,
    hasher, reader,
    test::{get_stress_iterations_count, usecase::*},
    Options,
};

fn test_dest_for_correction(usecase: &UseCase) -> Result<(), E> {
    let mut hashes: Vec<Vec<u8>> = Vec::new();
    for _ in 0..2 {
        let mut walker = Options::new()
            .entry(Entry::from(&usecase.root)?)?
            .tolerance(Tolerance::LogErrors)
            .walker(
                hasher::blake::Blake::default(),
                reader::buffering::Buffering::default(),
            )?;
        walker.collect()?;
        assert_eq!(walker.paths.len(), usecase.files.len());
        hashes.push(walker.hash()?.to_vec());
        assert_eq!(walker.count(), usecase.files.len());
    }
    assert_eq!(hashes.len(), 2);
    assert_eq!(hashes[0], hashes[1]);
    Ok(())
}
#[test]
fn correction() -> Result<(), E> {
    let usecase = UseCase::unnamed(5, 10, 3, &["aaa", "bbb", "ccc"])?;
    test_dest_for_correction(&usecase)?;
    usecase.clean()?;
    Ok(())
}

#[test]
fn stress() -> Result<(), E> {
    let usecase = UseCase::unnamed(5, 10, 3, &["aaa", "bbb", "ccc"])?;
    for _ in 0..get_stress_iterations_count() {
        test_dest_for_correction(&usecase)?;
    }
    usecase.clean()?;
    Ok(())
}

fn test_dest_for_changes(usecase: &UseCase) -> Result<(), E> {
    let mut walker_a = Options::new()
        .entry(Entry::from(&usecase.root)?)?
        .tolerance(Tolerance::LogErrors)
        .walker(
            hasher::blake::Blake::default(),
            reader::buffering::Buffering::default(),
        )?;
    walker_a.collect()?;
    assert_eq!(walker_a.paths.len(), usecase.files.len());
    let hash_a = walker_a.hash()?.to_vec();
    usecase.change(10)?;
    let mut walker_b = Options::new()
        .entry(Entry::from(&usecase.root)?)?
        .tolerance(Tolerance::LogErrors)
        .walker(
            hasher::blake::Blake::default(),
            reader::buffering::Buffering::default(),
        )?;
    walker_b.collect()?;
    assert_eq!(walker_b.paths.len(), usecase.files.len());
    let hash_b = walker_b.hash()?.to_vec();
    assert_eq!(walker_a.count(), usecase.files.len());
    assert_eq!(walker_b.count(), usecase.files.len());
    assert_ne!(hash_a, hash_b);
    Ok(())
}

#[test]
fn changes() -> Result<(), E> {
    let usecase = UseCase::unnamed(5, 10, 3, &["aaa", "bbb", "ccc"])?;
    test_dest_for_changes(&usecase)?;
    usecase.clean()?;
    Ok(())
}

#[test]
fn changes_stress() -> Result<(), E> {
    let usecase = UseCase::unnamed(5, 10, 3, &["aaa", "bbb", "ccc"])?;
    for _ in 0..get_stress_iterations_count() {
        test_dest_for_changes(&usecase)?;
    }
    usecase.clean()?;
    Ok(())
}

#[test]
fn stress_permissions_issue() -> Result<(), E> {
    for _ in 0..get_stress_iterations_count() {
        let mut walker = Options::new()
            .entry(Entry::from(temp_dir())?)?
            .tolerance(Tolerance::LogErrors)
            .walker(
                hasher::blake::Blake::default(),
                reader::buffering::Buffering::default(),
            )?;
        let hash = walker.collect()?.hash()?.to_vec();
        if walker.iter().count() > 0 {
            assert!(!hash.is_empty());
        } else {
            assert!(hash.is_empty());
        }
    }
    Ok(())
}

#[test]
fn empty_dest_folder() -> Result<(), E> {
    let usecase = UseCaseEmpty::gen()?;
    let mut walker = Options::new()
        .entry(Entry::from(&usecase.root)?)?
        .tolerance(Tolerance::LogErrors)
        .walker(
            hasher::blake::Blake::default(),
            reader::buffering::Buffering::default(),
        )?;
    let hash = walker.collect()?.hash()?.to_vec();
    assert!(hash.is_empty());
    usecase.clean()?;
    Ok(())
}

#[test]
fn empty_folders() -> Result<(), E> {
    let usecase = UseCase::unnamed(5, 0, 3, &[])?;
    let mut walker = Options::new()
        .entry(Entry::from(&usecase.root)?)?
        .tolerance(Tolerance::LogErrors)
        .walker(
            hasher::blake::Blake::default(),
            reader::buffering::Buffering::default(),
        )?;
    let hash = walker.collect()?.hash()?.to_vec();
    assert!(hash.is_empty());
    usecase.clean()?;
    Ok(())
}

#[test]
fn removed_dest() -> Result<(), E> {
    let usecase = UseCase::unnamed(5, 2, 3, &[])?;
    let mut walker = Options::new()
        .entry(Entry::from(&usecase.root)?)?
        .tolerance(Tolerance::LogErrors)
        .walker(
            hasher::blake::Blake::default(),
            reader::buffering::Buffering::default(),
        )?;
    walker.collect()?;
    assert_eq!(walker.paths.len(), usecase.files.len());
    usecase.clean()?;
    assert!(walker.hash()?.is_empty());
    assert_eq!(walker.invalid().len(), usecase.files.len());
    Ok(())
}

#[test]
fn removed_dest_no_tolerance() -> Result<(), E> {
    let usecase = UseCase::unnamed(5, 2, 3, &[])?;
    let mut walker = Options::new()
        .entry(Entry::from(&usecase.root)?)?
        .tolerance(Tolerance::StopOnErrors)
        .walker(
            hasher::blake::Blake::default(),
            reader::buffering::Buffering::default(),
        )?;
    walker.collect()?;
    assert_eq!(walker.paths.len(), usecase.files.len());
    usecase.clean()?;
    assert!(walker.hash().is_err());
    Ok(())
}

#[test]
fn iterator() -> Result<(), E> {
    let usecase = UseCase::unnamed(2, 4, 2, &[])?;
    let mut walker = Options::new().entry(Entry::from(&usecase.root)?)?.walker(
        hasher::blake::Blake::default(),
        reader::buffering::Buffering::default(),
    )?;
    let _ = walker.collect()?.hash()?;
    assert_eq!(walker.count(), usecase.files.len());
    assert_eq!(walker.iter().count(), usecase.files.len());
    for (filename, _) in walker.iter() {
        assert!(usecase.files.contains(filename));
    }
    usecase.clean()?;
    Ok(())
}
