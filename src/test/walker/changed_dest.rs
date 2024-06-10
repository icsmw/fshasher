use crate::{error::E, hasher, reader, test::usecase::*, Options, Tolerance};

// This test is about stability to see react on situation when after path had been collected,
// some files has been removed
#[test]
fn changed_dest_after_collecting_ignore_error() -> Result<(), E> {
    let usecase = UseCase::unnamed(5, 10, 3, &[])?;
    let mut walker = Options::from(&usecase.root)?
        .tolerance(Tolerance::LogErrors)
        .walker(
            hasher::blake::Blake::new(),
            reader::buffering::Buffering::default(),
        )?;
    walker.collect()?;
    let removed_count = 100;
    usecase.remove(removed_count)?;
    assert!(walker.hash().is_ok());
    assert_eq!(walker.invalid().len(), removed_count);
    usecase.clean()?;
    Ok(())
}

#[test]
fn changed_dest_after_collecting_with_error() -> Result<(), E> {
    let usecase = UseCase::unnamed(5, 10, 3, &[])?;
    let mut walker = Options::from(&usecase.root)?
        .tolerance(Tolerance::StopOnErrors)
        .walker(
            hasher::blake::Blake::new(),
            reader::buffering::Buffering::default(),
        )?;
    walker.collect()?;
    let removed_count = 100;
    usecase.remove(removed_count)?;
    assert!(walker.hash().is_err());
    usecase.clean()?;
    Ok(())
}
