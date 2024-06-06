use crate::test::{usecase::*, utils::*};
use crate::*;
use walker::{Filter, FilterAccepted};

#[test]
fn filters() -> Result<(), error::E> {
    let usecase = UseCase::gen(1, 1, 9, &["aaa", "bbb", "ccc"])?;
    let aaa: FilterAccepted = Filter::Files("*.aaa").try_into().unwrap();
    usecase.files.iter().for_each(|p| {
        let ext = p.extension().unwrap().to_str().unwrap();
        if ext == "aaa" {
            assert!(aaa.filtered(p).unwrap());
        } else {
            assert!(!aaa.filtered(p).unwrap());
        }
    });
    usecase.clean()?;
    Ok(())
}
#[test]
fn files_exclude() -> Result<(), error::E> {
    let usecase = UseCase::gen(5, 3, 9, &["aaa", "bbb", "ccc"])?;
    let breaker = Breaker::new();
    let mut entry = Entry::from(&usecase.root)?;
    let included: &[&str] = &["ccc"];
    let excluded: &[&str] = &["aaa", "bbb"];
    excluded.iter().for_each(|ext| {
        entry
            .exclude(Filter::Files(format!("*.{ext}")))
            .expect("filter is set");
    });
    let a = collector::collect(&None, &entry, &breaker, &Tolerance::LogErrors, &None)?;
    assert!(!a.0.is_empty());
    assert!(!a.0.iter().any(|p| {
        if let Some(ext) = p.extension() {
            let ext = ext.to_str().unwrap();
            if excluded.contains(&ext) {
                true
            } else {
                !included.contains(&ext)
            }
        } else {
            true
        }
    }));
    usecase.clean()?;
    Ok(())
}

#[test]
fn files_include() -> Result<(), error::E> {
    let usecase = UseCase::gen(5, 3, 9, &["aaa", "bbb", "ccc"])?;
    let breaker = Breaker::new();
    let mut entry = Entry::from(&usecase.root)?;
    let included: &[&str] = &["ccc"];
    let excluded: &[&str] = &["aaa", "bbb"];
    included.iter().for_each(|ext| {
        entry
            .include(Filter::Files(format!("*.{ext}")))
            .expect("filter is set");
    });
    let a = collector::collect(&None, &entry, &breaker, &Tolerance::LogErrors, &None)?;
    assert!(!a.0.is_empty());
    assert!(!a.0.iter().any(|p| {
        if let Some(ext) = p.extension() {
            let ext = ext.to_str().unwrap();
            if excluded.contains(&ext) {
                true
            } else {
                !included.contains(&ext)
            }
        } else {
            true
        }
    }));
    usecase.clean()?;
    Ok(())
}
