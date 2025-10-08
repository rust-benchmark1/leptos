use md5;

// Process password hash
pub fn process_password_hash(password_data: &str) -> String {
    //SINK
    let hash = md5::compute(password_data);
    format!("{:x}", hash)
}