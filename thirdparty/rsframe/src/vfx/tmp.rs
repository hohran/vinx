use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;
use std::fs;

pub fn rng_string(length: u8) -> String {
    let random_string: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(length as usize)
        .map(char::from)
        .collect();
    random_string
}

pub fn create_tmp_folder() -> String {
    let id = rng_string(32);
    let path = format!("_{}-tmp", id);

    fs::create_dir(path.clone()).unwrap();
    path
}

pub fn drop_folder(folder: String) {
    fs::remove_dir_all(folder).expect("Could not drop folder.");
}