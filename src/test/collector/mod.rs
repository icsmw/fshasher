mod filters;

use walker::Progress;

use crate::test::{usecase::*, utils::*};
use crate::*;
use std::thread;

const STABILITY_ITERATIONS_COUNT: usize = 100;

#[test]
fn correction() -> Result<(), error::E> {
    let usecase = UseCase::unnamed(5, 10, 3, &["aaa", "bbb", "ccc"])?;
    let breaker = Breaker::new();
    let mut a = collector::collect(
        &None,
        &Entry::from(&usecase.root)?,
        &breaker,
        &Tolerance::LogErrors,
        &None,
    )?;
    let mut b = collector::collect(
        &None,
        &Entry::from(&usecase.root)?,
        &breaker,
        &Tolerance::LogErrors,
        &None,
    )?;
    assert_eq!(a.0.len(), usecase.files.len());
    assert_eq!(b.0.len(), usecase.files.len());
    assert_eq!(a.0.len(), b.0.len());
    assert_eq!(a.1.len(), b.1.len());
    a.0.sort();
    a.1.sort();
    b.0.sort();
    b.1.sort();
    assert_eq!(paths_to_cmp_string(&a.0), paths_to_cmp_string(&b.0));
    assert_eq!(paths_to_cmp_string(&a.1), paths_to_cmp_string(&b.1));
    usecase.clean()?;
    Ok(())
}

#[test]
fn stability() -> Result<(), error::E> {
    let usecase = UseCase::unnamed(5, 10, 3, &["aaa", "bbb", "ccc"])?;
    let breaker = Breaker::new();
    for _ in 0..STABILITY_ITERATIONS_COUNT {
        let mut a = collector::collect(
            &None,
            &Entry::from(&usecase.root)?,
            &breaker,
            &Tolerance::LogErrors,
            &None,
        )?;
        let mut b = collector::collect(
            &None,
            &Entry::from(&usecase.root)?,
            &breaker,
            &Tolerance::LogErrors,
            &None,
        )?;
        assert_eq!(a.0.len(), usecase.files.len());
        assert_eq!(b.0.len(), usecase.files.len());
        assert_eq!(a.0.len(), b.0.len());
        assert_eq!(a.1.len(), b.1.len());
        a.0.sort();
        a.1.sort();
        b.0.sort();
        b.1.sort();
        assert_eq!(paths_to_cmp_string(&a.0), paths_to_cmp_string(&b.0));
        assert_eq!(paths_to_cmp_string(&a.1), paths_to_cmp_string(&b.1));
    }
    usecase.clean()?;
    Ok(())
}

#[test]
fn cancellation() -> Result<(), error::E> {
    let usecase = UseCase::unnamed(5, 10, 3, &["aaa", "bbb", "ccc"])?;
    let breaker = Breaker::new();
    breaker.abort();
    let result = collector::collect(
        &None,
        &Entry::from(&usecase.root)?,
        &breaker,
        &Tolerance::LogErrors,
        &None,
    );
    assert!(result.is_err());
    usecase.clean()?;
    Ok(())
}

#[test]
fn cancellation_stability() -> Result<(), error::E> {
    let usecase = UseCase::unnamed(5, 10, 3, &["aaa", "bbb", "ccc"])?;
    for _ in 0..STABILITY_ITERATIONS_COUNT {
        let breaker = Breaker::new();
        breaker.abort();
        let result = collector::collect(
            &None,
            &Entry::from(&usecase.root)?,
            &breaker,
            &Tolerance::LogErrors,
            &None,
        );
        assert!(result.is_err());
    }
    usecase.clean()?;
    Ok(())
}

#[test]
fn cancellation_during() -> Result<(), error::E> {
    let usecase = UseCase::unnamed(5, 10, 3, &[])?;
    let breaker = Breaker::new();
    let breaker_progress = breaker.clone();
    let (progress, Some(rx)) = Progress::channel(10) else {
        unreachable!("Progress channel has been created.")
    };
    let handle = thread::spawn(move || {
        while let Ok(_msg) = rx.recv() {
            breaker_progress.abort();
        }
    });
    let result = collector::collect(
        &Some(progress),
        &Entry::from(&usecase.root)?,
        &breaker,
        &Tolerance::LogErrors,
        &None,
    );
    let _ = handle.join();
    assert!(result.is_err());
    usecase.clean()?;
    Ok(())
}

#[test]
fn cancellation_during_stability() -> Result<(), error::E> {
    let usecase = UseCase::unnamed(5, 10, 3, &[])?;
    for _ in 0..STABILITY_ITERATIONS_COUNT {
        let breaker = Breaker::new();
        let breaker_progress = breaker.clone();
        let (progress, Some(rx)) = Progress::channel(10) else {
            unreachable!("Progress channel has been created.")
        };
        let handle = thread::spawn(move || {
            while let Ok(_msg) = rx.recv() {
                breaker_progress.abort();
            }
        });
        let result = collector::collect(
            &Some(progress),
            &Entry::from(&usecase.root)?,
            &breaker,
            &Tolerance::LogErrors,
            &None,
        );
        let _ = handle.join();
        assert!(result.is_err());
    }
    usecase.clean()?;
    Ok(())
}
