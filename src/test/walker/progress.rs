use std::thread;

use crate::{
    entry::Entry, error::E, hasher, reader, test::usecase::*, Options, ReadingStrategy, Tolerance,
};

#[test]
fn progress() -> Result<(), E> {
    let usecase = UseCase::unnamed(5, 10, 3, &["aaa", "bbb", "ccc"])?;
    let mut walker = Options::from(&usecase.root)?
        .progress(10)
        .reading_strategy(ReadingStrategy::Buffer)?
        .tolerance(Tolerance::LogErrors)
        .walker(
            hasher::blake::Blake::new(),
            reader::buffering::Buffering::default(),
        )?;
    let rx_progress = walker.progress().unwrap();
    let handle = thread::spawn(move || {
        let mut ticks: usize = 0;
        while let Ok(msg) = rx_progress.recv() {
            ticks += 1;
        }
        ticks
    });
    walker.init()?.hash()?;
    let ticks = handle.join().expect("progress thread is finished");
    assert!(ticks > 0);
    usecase.clean()?;
    Ok(())
}
