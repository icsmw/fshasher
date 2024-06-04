use crate::test::{usecase::*, utils::*};
use crate::*;

#[test]
fn correction() -> Result<(), error::E> {
    let usecase = UseCase::gen(5, 3, 10, &["aaa", "bbb", "ccc"])?;
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
    let usecase = UseCase::gen(5, 3, 5, &["aaa", "bbb", "ccc"])?;
    let breaker = Breaker::new();
    for _ in 0..1000 {
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
