use std::thread;

use crate::{error::E, hasher, reader, test::usecase::*, walker, Options};

#[test]
fn cancellation() -> Result<(), E> {
    let usecase = UseCase::unnamed(5, 10, 3, &[])?;
    let mut walker = Options::from(&usecase.root)?.progress(10).walker()?;
    let rx_progress = walker.progress().unwrap();
    walker.collect()?;
    let breaker = walker.breaker();
    let handle = thread::spawn(move || {
        while let Ok(_msg) = rx_progress.recv() {
            breaker.abort();
        }
    });
    let res = walker.hash::<hasher::blake::Blake, reader::buffering::Buffering>();
    if let Err(err) = res {
        assert!(matches!(err, walker::E::Aborted));
    } else {
        panic!("expecting hashing would be done with error");
    }
    assert!(handle.join().is_ok());
    usecase.clean()?;
    Ok(())
}
