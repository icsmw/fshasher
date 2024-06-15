use crate::{
    collector::Tolerance, entry::Entry, test::usecase::*, Hasher, Options, Reader, ReadingStrategy,
    E,
};
use std::path::PathBuf;

pub fn paths_to_cmp_string(paths: &[PathBuf]) -> String {
    paths
        .iter()
        .map(|p| p.display().to_string())
        .collect::<Vec<String>>()
        .join(",")
}

pub fn paths_to_cmp_string_vec(paths: Vec<&PathBuf>) -> String {
    paths
        .iter()
        .map(|p| p.display().to_string())
        .collect::<Vec<String>>()
        .join(",")
}

pub fn compare_same_dest<H: Hasher + 'static, R: Reader + 'static>(
    usecase: &UseCase,
    strategy: Option<ReadingStrategy>,
) -> Result<(), E>
where
    E: From<<H as Hasher>::Error> + From<<R as Reader>::Error>,
{
    let mut hashes: Vec<Vec<u8>> = Vec::new();
    for _ in 0..2 {
        let mut opt = Options::new()
            .entry(Entry::from(&usecase.root)?)?
            .tolerance(Tolerance::LogErrors);
        if let Some(ref strategy) = strategy {
            opt = opt.reading_strategy(strategy.clone())?;
        }
        let mut walker = opt.walker()?;
        walker.collect()?;
        assert_eq!(walker.paths.len(), usecase.files.len());
        hashes.push(walker.hash::<H, R>()?.to_vec());
        assert_eq!(
            walker
                .paths
                .iter()
                .filter(|(_, h)| if let Some(h) = h { h.is_ok() } else { false })
                .count(),
            usecase.files.len()
        );
    }
    assert_eq!(hashes.len(), 2);
    assert_eq!(hashes[0], hashes[1]);
    Ok(())
}

pub fn check_for_changes<H: Hasher + 'static, R: Reader + 'static>(
    usecase: &UseCase,
    strategy: Option<ReadingStrategy>,
) -> Result<(), E>
where
    E: From<<H as Hasher>::Error> + From<<R as Reader>::Error>,
{
    let mut opt = Options::new()
        .entry(Entry::from(&usecase.root)?)?
        .tolerance(Tolerance::LogErrors);
    if let Some(ref strategy) = strategy {
        opt = opt.reading_strategy(strategy.clone())?;
    }
    let mut walker_a = opt.clone().walker()?;
    walker_a.collect()?;
    assert_eq!(walker_a.paths.len(), usecase.files.len());
    let hash_a = walker_a.hash::<H, R>()?.to_vec();
    usecase.change(10)?;
    let mut walker_b = opt.walker()?;
    walker_b.collect()?;
    assert_eq!(walker_b.paths.len(), usecase.files.len());
    let hash_b = walker_b.hash::<H, R>()?.to_vec();
    assert_eq!(walker_a.count(), usecase.files.len());
    assert_eq!(walker_b.count(), usecase.files.len());
    assert_ne!(hash_a, hash_b);
    Ok(())
}
