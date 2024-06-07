use std::thread;

use crate::{error::E, hasher, reader, test::usecase::*, JobType, Options};

#[test]
fn progress() -> Result<(), E> {
    let usecase = UseCase::unnamed(5, 10, 3, &[])?;
    let mut walker = Options::from(&usecase.root)?.progress(10).walker(
        hasher::blake::Blake::new(),
        reader::buffering::Buffering::default(),
    )?;
    let rx_progress = walker.progress().unwrap();
    let handle = thread::spawn(move || {
        let mut ticks: usize = 0;
        let mut collecting = false;
        let mut hashing = false;
        while let Ok(msg) = rx_progress.recv() {
            ticks += 1;
            if matches!(msg.job, JobType::Collecting) {
                collecting = true;
            }
            if matches!(msg.job, JobType::Hashing) {
                hashing = true;
            }
        }
        (ticks, collecting, hashing)
    });
    walker.init()?.hash()?;
    let (ticks, collecting, hashing) = handle.join().expect("progress thread is finished");
    assert!(ticks > 0);
    assert!(collecting);
    assert!(hashing);
    usecase.clean()?;
    Ok(())
}
