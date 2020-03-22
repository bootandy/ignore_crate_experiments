use ignore::{WalkBuilder, WalkState};
use std::path::PathBuf;
use std::collections::HashMap;
use std::env;
use std::sync::{Arc, Mutex};
use std::thread;



pub (crate) fn walk_para_channel(dir: &str) {
    let (tx, rx) = channel::bounded::<PathBuf>(1000);

    // Receiver thread
    let stdout_thread = thread::spawn(move || {
        let mut hash: HashMap<String, u64> = HashMap::new();
        for dent in rx {
            //println!("{:?}", dent);
            hash.insert(dent.to_str().unwrap().to_string(), 1);
        }
        hash
    });

    // Walk the dir tree thread
    let builder = WalkBuilder::new(dir);
    builder.build_parallel().run(|| {
        let txc = tx.clone();
        Box::new(move |path| {
            match path {
                Ok(p) => {
                    txc.send(p.into_path()).unwrap();
                },
                Err(_) => {}
            };
            WalkState::Continue
        })
    });

    drop(tx);
    let hash = stdout_thread.join().unwrap();
    println!("Found {:?} files ", hash.len());
}

pub (crate) fn walk_para_hash_map(dir: &str) {
    let builder = WalkBuilder::new(dir);
    let hash: HashMap<PathBuf, u64> = HashMap::new();
    let mutex_hash = Arc::new(Mutex::new(hash));
    builder.build_parallel().run(|| {
        let the_hash = mutex_hash.clone();
        Box::new(move |path| {
            match path {
                Ok(p) => {
                    let mut h = the_hash.lock().unwrap();
                    h.insert(p.into_path(), 1);
                },
                Err(_) => {}
            };
            WalkState::Continue
        })
    });
    println!("Found {:?} files ", mutex_hash.lock().unwrap().len());
}
