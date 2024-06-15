mod filters;

use walker::Progress;

use crate::test::{get_stress_iterations_count, usecase::*, utils::*};
use crate::*;
use std::path::PathBuf;
use std::thread;

#[test]
fn correction() -> Result<(), E> {
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
    a.1.sort_by(|(a, _), (b, _)| a.cmp(b));
    b.0.sort();
    b.1.sort_by(|(a, _), (b, _)| a.cmp(b));
    assert_eq!(paths_to_cmp_string(&a.0), paths_to_cmp_string(&b.0));
    assert_eq!(
        paths_to_cmp_string_vec(a.1.iter().map(|(p, _)| p).collect::<Vec<&PathBuf>>()),
        paths_to_cmp_string_vec(b.1.iter().map(|(p, _)| p).collect::<Vec<&PathBuf>>())
    );
    usecase.clean()?;
    Ok(())
}

#[test]
fn stress() -> Result<(), E> {
    let usecase = UseCase::unnamed(5, 10, 3, &["aaa", "bbb", "ccc"])?;
    let breaker = Breaker::new();
    for _ in 0..get_stress_iterations_count() {
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
        a.1.sort_by(|(a, _), (b, _)| a.cmp(b));
        b.0.sort();
        b.1.sort_by(|(a, _), (b, _)| a.cmp(b));
        assert_eq!(paths_to_cmp_string(&a.0), paths_to_cmp_string(&b.0));
        assert_eq!(
            paths_to_cmp_string_vec(a.1.iter().map(|(p, _)| p).collect::<Vec<&PathBuf>>()),
            paths_to_cmp_string_vec(b.1.iter().map(|(p, _)| p).collect::<Vec<&PathBuf>>())
        );
    }
    usecase.clean()?;
    Ok(())
}

#[test]
fn cancellation() -> Result<(), E> {
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
fn cancellation_stress() -> Result<(), E> {
    let usecase = UseCase::unnamed(5, 10, 3, &["aaa", "bbb", "ccc"])?;
    for _ in 0..get_stress_iterations_count() {
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
fn cancellation_during() -> Result<(), E> {
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
fn cancellation_during_stress() -> Result<(), E> {
    let usecase = UseCase::unnamed(5, 10, 3, &[])?;
    for _ in 0..get_stress_iterations_count() {
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
