use crate::{
    collector::Tolerance, error::E, hasher, reader, test::usecase::*, Options, ReadingStrategy,
};

#[test]
fn buffer() -> Result<(), E> {
    let usecase = UseCase::unnamed(5, 10, 3, &["aaa", "bbb", "ccc"])?;
    let mut walker_a = Options::from(&usecase.root)?
        .reading_strategy(ReadingStrategy::Buffer)?
        .tolerance(Tolerance::LogErrors)
        .walker()?;
    let hash_a = walker_a
        .collect()?
        .hash::<hasher::blake::Blake, reader::buffering::Buffering>()?
        .to_vec();
    let mut walker_b = Options::from(&usecase.root)?
        .tolerance(Tolerance::LogErrors)
        .walker()?;
    let hash_b = walker_b
        .collect()?
        .hash::<hasher::blake::Blake, reader::buffering::Buffering>()?
        .to_vec();
    assert_eq!(walker_a.count(), usecase.files.len());
    assert_eq!(walker_b.count(), usecase.files.len());
    assert_eq!(hash_a, hash_b);
    usecase.clean()?;
    Ok(())
}

#[test]
fn complete() -> Result<(), E> {
    let usecase = UseCase::unnamed(5, 10, 3, &["aaa", "bbb", "ccc"])?;
    let mut walker_a = Options::from(&usecase.root)?
        .reading_strategy(ReadingStrategy::Complete)?
        .tolerance(Tolerance::LogErrors)
        .walker()?;
    let hash_a = walker_a
        .collect()?
        .hash::<hasher::blake::Blake, reader::buffering::Buffering>()?
        .to_vec();
    let mut walker_b = Options::from(&usecase.root)?
        .tolerance(Tolerance::LogErrors)
        .walker()?;
    let hash_b = walker_b
        .collect()?
        .hash::<hasher::blake::Blake, reader::buffering::Buffering>()?
        .to_vec();
    assert_eq!(walker_a.count(), usecase.files.len());
    assert_eq!(walker_b.count(), usecase.files.len());
    assert_eq!(hash_a, hash_b);
    usecase.clean()?;
    Ok(())
}

#[test]
fn memory_mapped() -> Result<(), E> {
    let usecase = UseCase::unnamed(5, 10, 3, &["aaa", "bbb", "ccc"])?;
    let mut walker_a = Options::from(&usecase.root)?
        .reading_strategy(ReadingStrategy::MemoryMapped)?
        .tolerance(Tolerance::LogErrors)
        .walker()?;
    let hash_a = walker_a
        .collect()?
        .hash::<hasher::blake::Blake, reader::mapping::Mapping>()?
        .to_vec();
    let mut walker_b = Options::from(&usecase.root)?
        .tolerance(Tolerance::LogErrors)
        .walker()?;
    let hash_b = walker_b
        .collect()?
        .hash::<hasher::blake::Blake, reader::buffering::Buffering>()?
        .to_vec();
    assert_eq!(walker_a.count(), usecase.files.len());
    assert_eq!(walker_b.count(), usecase.files.len());
    assert_eq!(hash_a, hash_b);
    usecase.clean()?;
    Ok(())
}

#[test]
fn scenario() -> Result<(), E> {
    let usecase = UseCase::unnamed(5, 10, 3, &["aaa", "bbb", "ccc"])?;
    let mut walker_a = Options::from(&usecase.root)?
        .reading_strategy(ReadingStrategy::Scenario(vec![
            (0..1024 * 1024, Box::new(ReadingStrategy::Complete)),
            (1024 * 1024..u64::MAX, Box::new(ReadingStrategy::Buffer)),
        ]))?
        .tolerance(Tolerance::LogErrors)
        .walker()?;
    let hash_a = walker_a
        .collect()?
        .hash::<hasher::blake::Blake, reader::buffering::Buffering>()?
        .to_vec();
    let mut walker_b = Options::from(&usecase.root)?
        .tolerance(Tolerance::LogErrors)
        .walker()?;
    let hash_b = walker_b
        .collect()?
        .hash::<hasher::blake::Blake, reader::buffering::Buffering>()?
        .to_vec();
    assert_eq!(walker_a.count(), usecase.files.len());
    assert_eq!(walker_b.count(), usecase.files.len());
    assert_eq!(hash_a, hash_b);
    usecase.clean()?;
    Ok(())
}
