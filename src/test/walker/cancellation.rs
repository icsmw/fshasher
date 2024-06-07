use std::thread;

use crate::{error::E, hasher, reader, test::usecase::*, walker, JobType, Options};

// #[test]
// fn cancel_on_collecting() -> Result<(), E> {
//     let usecase = UseCase::unnamed(5, 10, 3, &[])?;
//     let mut walker = Options::from(&usecase.root)?.progress(10).walker(
//         hasher::blake::Blake::new(),
//         reader::buffering::Buffering::default(),
//     )?;
//     let rx_progress = walker.progress().unwrap();
//     let breaker = walker.breaker();
//     let handle = thread::spawn(move || {
//         while let Ok(msg) = rx_progress.recv() {
//             // println!(">>>>>>>>>>>>>>>>>> tick");
//             if matches!(msg.job, JobType::Collecting) {
//                 println!(">>>>>>>>>>>>>>>>>> WILL ABORT!");

//                 breaker.abort();
//             }
//         }
//     });
//     let res = walker.init()?.hash();
//     println!(">>>>>>>>>>>>>>>>>>>>>>>: {res:?}");
//     if let Err(err) = res {
//         assert!(matches!(err, walker::E::Aborted));
//     } else {
//         panic!("expecting hashing would be done with error");
//     }
//     assert!(handle.join().is_ok());
//     usecase.clean()?;
//     Ok(())
// }
