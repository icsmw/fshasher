mod cancellation;
mod changed_dest;
mod progress;
mod stratagies;

use std::env::temp_dir;

use crate::{
    collector::Tolerance,
    entry::Entry,
    hasher, reader,
    test::{get_stress_iterations_count, usecase::*, utils},
    Options, E,
};

#[test]
fn correction() -> Result<(), E> {
    let usecase = UseCase::unnamed(5, 10, 3, &["aaa", "bbb", "ccc"])?;
    utils::compare_same_dest::<hasher::blake::Blake, reader::buffering::Buffering>(&usecase, None)?;
    usecase.clean()?;
    Ok(())
}

#[test]
fn stress() -> Result<(), E> {
    let usecase = UseCase::unnamed(5, 10, 3, &["aaa", "bbb", "ccc"])?;
    for _ in 0..get_stress_iterations_count() {
        utils::compare_same_dest::<hasher::blake::Blake, reader::buffering::Buffering>(
            &usecase, None,
        )?;
    }
    usecase.clean()?;
    Ok(())
}

#[test]
fn changes() -> Result<(), E> {
    let usecase = UseCase::unnamed(5, 10, 3, &["aaa", "bbb", "ccc"])?;
    utils::check_for_changes::<hasher::blake::Blake, reader::buffering::Buffering>(&usecase, None)?;
    usecase.clean()?;
    Ok(())
}

#[test]
fn changes_stress() -> Result<(), E> {
    let usecase = UseCase::unnamed(5, 10, 3, &["aaa", "bbb", "ccc"])?;
    for _ in 0..get_stress_iterations_count() {
        utils::check_for_changes::<hasher::blake::Blake, reader::buffering::Buffering>(
            &usecase, None,
        )?;
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
            .walker()?;
        let hash = walker
            .collect()?
            .hash::<hasher::blake::Blake, reader::buffering::Buffering>()?
            .to_vec();
        if walker
            .iter()
            .filter(|(_, h)| {
                if let Some(hash) = h {
                    hash.is_ok()
                } else {
                    false
                }
            })
            .count()
            > 0
        {
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
        .walker()?;
    let hash = walker
        .collect()?
        .hash::<hasher::blake::Blake, reader::buffering::Buffering>()?
        .to_vec();
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
        .walker()?;
    let hash = walker
        .collect()?
        .hash::<hasher::blake::Blake, reader::buffering::Buffering>()?
        .to_vec();
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
        .walker()?;
    walker.collect()?;
    assert_eq!(walker.paths.len(), usecase.files.len());
    usecase.clean()?;
    assert!(walker
        .hash::<hasher::blake::Blake, reader::buffering::Buffering>()
        .is_ok());
    assert_eq!(
        walker
            .iter()
            .filter(|(_, h)| if let Some(h) = h { h.is_err() } else { false })
            .count(),
        usecase.files.len()
    );
    Ok(())
}

#[test]
fn removed_dest_no_tolerance() -> Result<(), E> {
    let usecase = UseCase::unnamed(5, 2, 3, &[])?;
    let mut walker = Options::new()
        .entry(Entry::from(&usecase.root)?)?
        .tolerance(Tolerance::StopOnErrors)
        .walker()?;
    walker.collect()?;
    assert_eq!(walker.paths.len(), usecase.files.len());
    usecase.clean()?;
    assert!(walker
        .hash::<hasher::blake::Blake, reader::buffering::Buffering>()
        .is_err());
    Ok(())
}

#[test]
fn iterator() -> Result<(), E> {
    let usecase = UseCase::unnamed(2, 4, 2, &[])?;
    let mut walker = Options::new()
        .entry(Entry::from(&usecase.root)?)?
        .walker()?;
    let _ = walker
        .collect()?
        .hash::<hasher::blake::Blake, reader::buffering::Buffering>()?;
    assert_eq!(walker.count(), usecase.files.len());
    assert_eq!(walker.iter().count(), usecase.files.len());
    for (filename, _) in walker.iter() {
        assert!(usecase.files.contains(filename));
    }
    usecase.clean()?;
    Ok(())
}
