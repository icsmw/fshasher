use crate::{
    test::{usecase::*, utils::create_text_file},
    Entry, Options, E,
};
use uuid::Uuid;

fn ingore_folders(count: u8, files: u16, deep: u8) -> Result<(), E> {
    let mut folders = Vec::new();
    for _ in 0..(count * 2) {
        folders.push(Uuid::new_v4().to_string());
    }
    let usecase = UseCase::folders(
        &folders.iter().map(|f| f.as_str()).collect::<Vec<&str>>(),
        files,
        deep,
        &[],
    )?;
    // Collect without any filters
    let mut walker = Options::from(&usecase.root)?.walker()?;
    walker.collect()?;
    assert_eq!(
        walker.count(),
        UseCase::expectation((count * 2) as usize, files as usize, (deep + 1) as usize)
    );
    // Collect with filters
    let mut list = String::new();
    let mut ignored = Vec::new();
    for i in 0..count {
        list.insert_str(0, &format!("**/{}\n", folders[i as usize]));
        ignored.push(folders[i as usize].to_owned());
    }
    create_text_file(usecase.root.join(".ignore"), &list)?;
    let entry = Entry::new()
        .entry(&usecase.root)?
        .context(crate::entry::ContextFile::Ignore(".ignore"));
    let mut walker = Options::new().entry(entry)?.walker()?;
    walker.collect()?;
    assert_eq!(
        walker.count(),
        // +1 <- .ignore file
        UseCase::expectation(count as usize, files as usize, (deep + 1) as usize) + 1
    );
    for (p, _) in walker.iter() {
        if p.parent().unwrap() == usecase.root {
            continue;
        }
        for ignored in ignored.iter() {
            assert!(!p.to_string_lossy().contains(ignored));
        }
    }
    usecase.clean()?;
    Ok(())
}

fn ingore_folders_with_files(count: u8, files: usize, deep: u8) -> Result<(), E> {
    let mut folders = Vec::new();
    for _ in 0..(count * 2) {
        folders.push(Uuid::new_v4().to_string());
    }
    let mut exts = Vec::new();
    for i in 0..(files * 2) {
        exts.push(format!("ext_{i}"));
    }
    let usecase = UseCase::folders(
        &folders.iter().map(|f| f.as_str()).collect::<Vec<&str>>(),
        (files * 2) as u16,
        deep,
        &exts.iter().map(|s| s.as_str()).collect::<Vec<&str>>(),
    )?;
    // Collect without any filters
    let mut walker = Options::from(&usecase.root)?.walker()?;
    walker.collect()?;
    assert_eq!(
        walker.count(),
        UseCase::expectation((count * 2) as usize, files * 2, (deep + 1) as usize)
    );
    // Collect with filters
    let mut list = String::new();
    let mut ignored = Vec::new();
    for i in 0..count {
        list.insert_str(0, &format!("**/{}\n", folders[i as usize]));
        ignored.push(folders[i as usize].to_owned());
    }
    (0..files).for_each(|i| {
        list.insert_str(0, &format!("*.{}\n", exts[i]));
        ignored.push(format!(".{}", exts[i]));
    });
    create_text_file(usecase.root.join(".ignore"), &list)?;
    let entry = Entry::new()
        .entry(&usecase.root)?
        .context(crate::entry::ContextFile::Ignore(".ignore"));
    let mut walker = Options::new().entry(entry)?.walker()?;
    walker.collect()?;
    assert_eq!(
        walker.count(),
        // +1 <- .ignore file
        UseCase::expectation(count as usize, files, (deep + 1) as usize) + 1
    );
    for (p, _) in walker.iter() {
        if p.parent().unwrap() == usecase.root {
            continue;
        }
        for ignored in ignored.iter() {
            assert!(!p.to_string_lossy().contains(ignored));
        }
    }
    usecase.clean()?;
    Ok(())
}

fn ingore_folders_negative(count: u8, files: u16, deep: u8) -> Result<(), E> {
    let mut folders = Vec::new();
    for _ in 0..(count * 2) {
        folders.push(Uuid::new_v4().to_string());
    }
    let usecase = UseCase::folders(
        &folders.iter().map(|f| f.as_str()).collect::<Vec<&str>>(),
        files,
        deep,
        &[],
    )?;
    // Collect without any filters
    let mut walker = Options::from(&usecase.root)?.walker()?;
    walker.collect()?;
    assert_eq!(
        walker.count(),
        UseCase::expectation((count * 2) as usize, files as usize, (deep + 1) as usize)
    );
    // Collect with filters
    let mut list = String::new();
    let mut ignored = Vec::new();
    for i in 0..count {
        list.insert_str(0, &format!("!**/{}\n", folders[i as usize]));
        ignored.push(folders[i as usize].to_owned());
    }
    create_text_file(usecase.root.join(".ignore"), &list)?;
    let entry = Entry::new()
        .entry(&usecase.root)?
        .context(crate::entry::ContextFile::Ignore(".ignore"));
    let mut walker = Options::new().entry(entry)?.walker()?;
    walker.collect()?;
    assert_eq!(
        walker.count(),
        // +1 <- .ignore file
        UseCase::expectation((count * 2) as usize, files as usize, (deep + 1) as usize) + 1
    );
    usecase.clean()?;
    Ok(())
}

fn ingore_folders_with_files_negative(count: u8, files: usize, deep: u8) -> Result<(), E> {
    let mut folders = Vec::new();
    for _ in 0..(count * 2) {
        folders.push(Uuid::new_v4().to_string());
    }
    let mut exts = Vec::new();
    for i in 0..(files * 2) {
        exts.push(format!("ext_{i}"));
    }
    let usecase = UseCase::folders(
        &folders.iter().map(|f| f.as_str()).collect::<Vec<&str>>(),
        (files * 2) as u16,
        deep,
        &exts.iter().map(|s| s.as_str()).collect::<Vec<&str>>(),
    )?;
    // Collect without any filters
    let mut walker = Options::from(&usecase.root)?.walker()?;
    walker.collect()?;
    assert_eq!(
        walker.count(),
        UseCase::expectation((count * 2) as usize, files * 2, (deep + 1) as usize)
    );
    // Collect with filters
    let mut list = String::new();
    let mut ignored = Vec::new();
    for i in 0..count {
        list.insert_str(0, &format!("!**/{}\n", folders[i as usize]));
    }
    (0..files).for_each(|i| {
        list.insert_str(0, &format!("*.{}\n", exts[i]));
        ignored.push(format!(".{}", exts[i]));
    });
    create_text_file(usecase.root.join(".ignore"), &list)?;
    let entry = Entry::new()
        .entry(&usecase.root)?
        .context(crate::entry::ContextFile::Ignore(".ignore"));
    let mut walker = Options::new().entry(entry)?.walker()?;
    walker.collect()?;
    assert_eq!(
        walker.count(),
        // +1 <- .ignore file
        UseCase::expectation((count * 2) as usize, files, (deep + 1) as usize) + 1
    );
    for (p, _) in walker.iter() {
        if p.parent().unwrap() == usecase.root {
            continue;
        }
        for ignored in ignored.iter() {
            assert!(!p.to_string_lossy().contains(ignored));
        }
    }
    // No collect all extentions also
    let mut list = String::new();
    for i in 0..count {
        list.insert_str(0, &format!("!**/{}\n", folders[i as usize]));
    }
    (0..files).for_each(|i| {
        list.insert_str(0, &format!("!*.{}\n", exts[i]));
    });
    create_text_file(usecase.root.join(".ignore"), &list)?;
    let entry = Entry::new()
        .entry(&usecase.root)?
        .context(crate::entry::ContextFile::Ignore(".ignore"));
    let mut walker = Options::new().entry(entry)?.walker()?;
    walker.collect()?;
    assert_eq!(
        walker.count(),
        // +1 <- .ignore file
        UseCase::expectation((count * 2) as usize, files * 2, (deep + 1) as usize) + 1
    );
    usecase.clean()?;
    Ok(())
}

fn accept_files(count: u8, files: usize, deep: u8) -> Result<(), E> {
    let mut folders = Vec::new();
    for _ in 0..count {
        folders.push(Uuid::new_v4().to_string());
    }
    let mut exts = Vec::new();
    for i in 0..(files * 2) {
        exts.push(format!("ext_{i}"));
    }
    let usecase = UseCase::folders(
        &folders.iter().map(|f| f.as_str()).collect::<Vec<&str>>(),
        (files * 2) as u16,
        deep,
        &exts.iter().map(|s| s.as_str()).collect::<Vec<&str>>(),
    )?;
    // Collect without any filters
    let mut walker = Options::from(&usecase.root)?.walker()?;
    walker.collect()?;
    assert_eq!(
        walker.count(),
        UseCase::expectation(count as usize, files * 2, (deep + 1) as usize)
    );
    // Collect with filters
    let mut list = String::new();
    let mut accepted = Vec::new();
    (0..files).for_each(|i| {
        list.insert_str(0, &format!("*.{}\n", exts[i]));
        accepted.push(format!(".{}", exts[i]));
    });
    create_text_file(usecase.root.join(".accept"), &list)?;
    let entry = Entry::new()
        .entry(&usecase.root)?
        .context(crate::entry::ContextFile::Accept(".accept"));
    let mut walker = Options::new().entry(entry)?.walker()?;
    walker.collect()?;
    assert_eq!(
        walker.count(),
        UseCase::expectation(count as usize, files, (deep + 1) as usize)
    );
    for (p, _) in walker.iter() {
        if p.parent().unwrap() == usecase.root {
            continue;
        }
        for accepted in accepted.iter() {
            assert!(p.to_string_lossy().contains(accepted));
        }
    }
    usecase.clean()?;
    Ok(())
}

#[test]
fn accept_files_test() -> Result<(), E> {
    accept_files(2, 1, 3)?;
    accept_files(3, 1, 3)?;
    accept_files(4, 1, 3)
}

#[test]
fn ingore_folders_test() -> Result<(), E> {
    ingore_folders(2, 1, 3)?;
    ingore_folders(3, 1, 3)?;
    ingore_folders(4, 1, 3)
}

#[test]
fn ingore_folders_with_files_test() -> Result<(), E> {
    ingore_folders_with_files(2, 1, 3)?;
    ingore_folders_with_files(3, 2, 3)?;
    ingore_folders_with_files(4, 3, 3)
}

#[test]
fn ingore_folders_negative_test() -> Result<(), E> {
    ingore_folders_negative(2, 1, 3)?;
    ingore_folders_negative(3, 1, 3)?;
    ingore_folders_negative(4, 1, 3)
}

#[test]
fn ingore_folders_with_files_negative_test() -> Result<(), E> {
    ingore_folders_with_files_negative(2, 1, 3)?;
    ingore_folders_with_files_negative(3, 2, 3)?;
    ingore_folders_with_files_negative(4, 3, 3)
}

#[test]
fn no_matches() -> Result<(), E> {
    let folders = 5;
    let files = 2;
    let deep = 3;
    let usecase = UseCase::unnamed(folders, files, deep, &[])?;
    let mut walker = Options::from(&usecase.root)?.walker()?;
    walker.collect()?;
    assert_eq!(
        walker.count(),
        UseCase::expectation(folders as usize, files as usize, (deep + 1) as usize)
    );
    create_text_file(usecase.root.join(".ignore"), "**/fake\n**/fake_2")?;
    let entry = Entry::new()
        .entry(&usecase.root)?
        .context(crate::entry::ContextFile::Ignore(".ignore"));
    let mut walker = Options::new().entry(entry)?.walker()?;
    walker.collect()?;
    assert_eq!(
        walker.count(),
        // +1 <- .ignore file
        UseCase::expectation(folders as usize, files as usize, (deep + 1) as usize) + 1
    );
    usecase.clean()?;
    Ok(())
}

#[test]
fn empty_file() -> Result<(), E> {
    let folders = 5;
    let files = 2;
    let deep = 3;
    let usecase = UseCase::unnamed(folders, files, deep, &[])?;
    let mut walker = Options::from(&usecase.root)?.walker()?;
    walker.collect()?;
    assert_eq!(
        walker.count(),
        UseCase::expectation(folders as usize, files as usize, (deep + 1) as usize)
    );
    create_text_file(usecase.root.join(".ignore"), "")?;
    let entry = Entry::new()
        .entry(&usecase.root)?
        .context(crate::entry::ContextFile::Ignore(".ignore"));
    let mut walker = Options::new().entry(entry)?.walker()?;
    walker.collect()?;
    assert_eq!(
        walker.count(),
        // +1 <- .ignore file
        UseCase::expectation(folders as usize, files as usize, (deep + 1) as usize) + 1
    );
    usecase.clean()?;
    Ok(())
}

#[test]
fn ignore_root() -> Result<(), E> {
    let folders = 5;
    let files = 2;
    let deep = 3;
    let usecase = UseCase::unnamed(folders, files, deep, &[])?;
    let mut walker = Options::from(&usecase.root)?.walker()?;
    walker.collect()?;
    assert_eq!(
        walker.count(),
        UseCase::expectation(folders as usize, files as usize, (deep + 1) as usize)
    );
    create_text_file(
        usecase.root.join(".ignore"),
        format!("**/{}/**/*", usecase.root.display()),
    )?;
    let entry = Entry::new()
        .entry(&usecase.root)?
        .context(crate::entry::ContextFile::Ignore(".ignore"));
    let mut walker = Options::new().entry(entry)?.walker()?;
    walker.collect()?;
    assert_eq!(walker.count(), 0);
    usecase.clean()?;
    Ok(())
}
