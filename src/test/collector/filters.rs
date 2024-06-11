use crate::*;
use entry::{Filter, FilterAccepted, PatternFilter, PatternFilterAccepted};
use test::usecase::*;

#[test]
fn filters() -> Result<(), error::E> {
    let usecase = UseCase::unnamed(1, 9, 1, &["aaa", "bbb", "ccc"])?;
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
fn patterns() -> Result<(), error::E> {
    let usecase = UseCase::unnamed(1, 9, 1, &["aaa", "bbb", "ccc"])?;
    let aaa: PatternFilterAccepted = PatternFilter::Accept("*.aaa").try_into().unwrap();
    usecase.files.iter().for_each(|p| {
        let ext = p.extension().unwrap().to_str().unwrap();
        if ext == "aaa" {
            assert!(aaa.filtered(p));
        } else {
            assert!(!aaa.filtered(p));
        }
    });
    let ddd: PatternFilterAccepted = PatternFilter::Accept("*.ddd").try_into().unwrap();
    usecase.files.iter().for_each(|p| {
        assert!(!ddd.filtered(p));
    });
    let ccc: PatternFilterAccepted = PatternFilter::Ignore("*.ccc").try_into().unwrap();
    usecase.files.iter().for_each(|p| {
        let ext = p.extension().unwrap().to_str().unwrap();
        if ext == "ccc" {
            assert!(!ccc.filtered(p));
        } else {
            assert!(ccc.filtered(p));
        }
    });
    usecase.clean()?;
    Ok(())
}

#[test]
fn files_exclude() -> Result<(), error::E> {
    let usecase = UseCase::unnamed(5, 9, 3, &["aaa", "bbb", "ccc"])?;
    let breaker = Breaker::new();
    let mut entry = Entry::from(&usecase.root)?;
    let included: &[&str] = &["ccc"];
    let excluded: &[&str] = &["aaa", "bbb"];
    for ext in excluded.iter() {
        entry = entry
            .exclude(Filter::Files(format!("*.{ext}")))
            .expect("filter is set");
    }
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
    let usecase = UseCase::unnamed(5, 9, 3, &["aaa", "bbb", "ccc"])?;
    let breaker = Breaker::new();
    let mut entry = Entry::from(&usecase.root)?;
    let included: &[&str] = &["ccc"];
    let excluded: &[&str] = &["aaa", "bbb"];
    for ext in included.iter() {
        entry = entry
            .include(Filter::Files(format!("*.{ext}")))
            .expect("filter is set");
    }
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
fn folders_exclude() -> Result<(), error::E> {
    let usecase = UseCase::folders(
        &[
            "aaa",
            "bbb",
            "exclude_ccc",
            "ddd_exclude",
            "exclude",
            "eee_exclude_eee",
        ],
        9,
        3,
        &["aaa", "bbb", "ccc"],
    )?;
    let breaker = Breaker::new();
    let entry = Entry::from(&usecase.root)?
        .exclude(Filter::Folders("*exclude*"))
        .expect("filter is set");
    let a = collector::collect(&None, &entry, &breaker, &Tolerance::LogErrors, &None)?;
    assert!(!a.0.is_empty());
    assert!(!a.0.iter().any(|p| p.to_string_lossy().contains("exclude")));
    usecase.clean()?;
    Ok(())
}

#[test]
fn folders_include() -> Result<(), error::E> {
    let usecase = UseCase::folders(
        &[
            "aaa",
            "bbb",
            "include_ccc",
            "ddd_include",
            "include",
            "eee_include_eee",
        ],
        9,
        3,
        &["aaa", "bbb", "ccc"],
    )?;
    let breaker = Breaker::new();
    let entry = Entry::from(&usecase.root)?
        .include(Filter::Folders("*include*"))
        .expect("filter is set");
    let a = collector::collect(&None, &entry, &breaker, &Tolerance::LogErrors, &None)?;
    assert!(!a.0.is_empty());
    assert!(!a.0.iter().any(|p| !p.to_string_lossy().contains("include")));
    usecase.clean()?;
    Ok(())
}

#[test]
fn folders_and_files() -> Result<(), error::E> {
    let usecase = UseCase::folders_and_files(
        &[
            "aaa",
            "bbb",
            "exclude_ccc",
            "ddd_exclude",
            "exclude",
            "eee_exclude_eee",
        ],
        &[
            "aaa",
            "bbb",
            "include_ccc",
            "ddd_include",
            "include",
            "eee_include_eee",
        ],
        3,
        &["aaa", "bbb", "ccc"],
    )?;
    let breaker = Breaker::new();
    let entry = Entry::from(&usecase.root)?
        .exclude(Filter::Folders("*exclude*"))
        .expect("filter is set")
        .include(Filter::Files("*include*"))
        .expect("filter is set");
    let a = collector::collect(&None, &entry, &breaker, &Tolerance::LogErrors, &None)?;
    assert!(!a.0.is_empty());
    assert!(a
        .0
        .iter()
        .any(|p| if p.to_string_lossy().contains("exclude") {
            false
        } else {
            p.file_name().unwrap().to_string_lossy().contains("include")
        }));
    usecase.clean()?;
    Ok(())
}

#[test]
fn folders_and_files_common_exclude() -> Result<(), error::E> {
    let usecase = UseCase::folders_and_files(
        &[
            "aaa",
            "bbb",
            "exclude_ccc",
            "ddd_exclude",
            "exclude",
            "eee_exclude_eee",
        ],
        &[
            "aaa",
            "bbb",
            "exclude_ccc",
            "ddd_exclude",
            "exclude",
            "eee_exclude_eee",
        ],
        3,
        &["aaa", "bbb", "ccc"],
    )?;
    let breaker = Breaker::new();
    let entry = Entry::from(&usecase.root)?
        .exclude(Filter::Common("*exclude*"))
        .expect("filter is set");
    let a = collector::collect(&None, &entry, &breaker, &Tolerance::LogErrors, &None)?;
    assert!(!a.0.is_empty());
    assert!(!a
        .0
        .iter()
        .any(|p| { p.to_string_lossy().contains("exclude") }));
    usecase.clean()?;
    Ok(())
}

#[test]
fn folders_and_files_common_include() -> Result<(), error::E> {
    let usecase = UseCase::folders_and_files(
        &[
            "aaa",
            "bbb",
            "include_ccc",
            "ddd_include",
            "include",
            "eee_include_eee",
        ],
        &[
            "aaa",
            "bbb",
            "include_ccc",
            "ddd_include",
            "include",
            "eee_include_eee",
        ],
        3,
        &["aaa", "bbb", "ccc"],
    )?;
    let breaker = Breaker::new();
    let entry = Entry::from(&usecase.root)?
        .include(Filter::Common("*include*"))
        .expect("filter is set");
    let a = collector::collect(&None, &entry, &breaker, &Tolerance::LogErrors, &None)?;
    assert!(!a.0.is_empty());
    assert!(!a.0.iter().any(|p| !p.to_string_lossy().contains("include")));
    usecase.clean()?;
    Ok(())
}

#[test]
fn patterns_exclude() -> Result<(), error::E> {
    let usecase = UseCase::folders_and_files(
        &[
            "aaa",
            "bbb",
            "exclude_ccc",
            "ddd_exclude",
            "exclude",
            "eee_exclude_eee",
        ],
        &[
            "aaa",
            "bbb",
            "exclude_ccc",
            "ddd_exclude",
            "exclude",
            "eee_exclude_eee",
        ],
        3,
        &["aaa", "bbb", "ccc"],
    )?;
    let breaker = Breaker::new();
    let entry = Entry::from(&usecase.root)?
        .pattern(PatternFilter::Ignore("*exclude*"))
        .expect("filter is set");
    let a = collector::collect(&None, &entry, &breaker, &Tolerance::LogErrors, &None)?;
    assert!(!a.0.is_empty());
    assert!(!a
        .0
        .iter()
        .any(|p| { p.to_string_lossy().contains("exclude") }));
    usecase.clean()?;
    Ok(())
}

#[test]
fn patterns_include() -> Result<(), error::E> {
    let usecase = UseCase::folders_and_files(
        &[
            "aaa",
            "bbb",
            "include_ccc",
            "ddd_include",
            "include",
            "eee_include_eee",
        ],
        &[
            "aaa",
            "bbb",
            "include_ccc",
            "ddd_include",
            "include",
            "eee_include_eee",
        ],
        3,
        &["aaa", "bbb", "ccc"],
    )?;
    let breaker = Breaker::new();
    let entry = Entry::from(&usecase.root)?
        .pattern(PatternFilter::Accept("*include*"))
        .expect("filter is set");
    let a = collector::collect(&None, &entry, &breaker, &Tolerance::LogErrors, &None)?;
    assert!(!a.0.is_empty());
    assert!(!a.0.iter().any(|p| !p.to_string_lossy().contains("include")));
    usecase.clean()?;
    Ok(())
}

#[test]
fn patterns_cmb() -> Result<(), error::E> {
    let usecase = UseCase::folders_and_files(
        &[
            "aaa",
            "bbb",
            "exclude_ccc",
            "ddd_exclude",
            "exclude",
            "eee_exclude_eee",
        ],
        &[
            "aaa",
            "bbb",
            "exclude_ccc",
            "ddd_exclude",
            "exclude",
            "eee_exclude_eee",
        ],
        3,
        &["aaa", "bbb", "ccc"],
    )?;
    let breaker = Breaker::new();
    let entry = Entry::from(&usecase.root)?
        .pattern(PatternFilter::Cmb(vec![
            PatternFilter::Ignore("*exclude*"),
            PatternFilter::Ignore("*.ccc"),
        ]))
        .expect("filter is set");
    let a = collector::collect(&None, &entry, &breaker, &Tolerance::LogErrors, &None)?;
    assert!(!a.0.is_empty());
    assert!(!a.0.iter().any(|p| {
        p.to_string_lossy().contains("exclude") && p.extension().unwrap().to_str().unwrap() != "ccc"
    }));
    usecase.clean()?;
    Ok(())
}
