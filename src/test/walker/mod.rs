mod stratagies;
use crate::{error::E, hasher, reader, test::usecase::*, Entry, Options, Tolerance};

#[test]
fn correction() -> Result<(), E> {
    let usecase = UseCase::gen(5, 3, 10, &["aaa", "bbb", "ccc"])?;
    let mut walker_a = Options::new()
        .entry(Entry::from(&usecase.root)?)?
        .tolerance(Tolerance::LogErrors)
        .walker(
            hasher::blake::Blake::new(),
            reader::buffering::Buffering::default(),
        )?;
    let hash_a = walker_a.init()?.hash()?.to_vec();
    let mut walker_b = Options::new()
        .entry(Entry::from(&usecase.root)?)?
        .tolerance(Tolerance::LogErrors)
        .walker(
            hasher::blake::Blake::new(),
            reader::buffering::Buffering::default(),
        )?;
    let hash_b = walker_b.init()?.hash()?.to_vec();
    assert_eq!(walker_a.count(), usecase.files.len());
    assert_eq!(walker_b.count(), usecase.files.len());
    assert_eq!(hash_a, hash_b);
    usecase.clean()?;
    Ok(())
}

#[test]
fn stability() -> Result<(), E> {
    let usecase = UseCase::gen(5, 3, 5, &["aaa", "bbb", "ccc"])?;
    for _ in 0..10 {
        let mut walker_a = Options::new()
            .entry(Entry::from(&usecase.root)?)?
            .tolerance(Tolerance::LogErrors)
            .walker(
                hasher::blake::Blake::new(),
                reader::buffering::Buffering::default(),
            )?;
        let hash_a = walker_a.init()?.hash()?.to_vec();
        let mut walker_b = Options::new()
            .entry(Entry::from(&usecase.root)?)?
            .tolerance(Tolerance::LogErrors)
            .walker(
                hasher::blake::Blake::new(),
                reader::buffering::Buffering::default(),
            )?;
        let hash_b = walker_b.init()?.hash()?.to_vec();
        assert_eq!(walker_a.count(), usecase.files.len());
        assert_eq!(walker_b.count(), usecase.files.len());
        assert_eq!(hash_a, hash_b);
    }
    usecase.clean()?;
    Ok(())
}
