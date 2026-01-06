use rocket::http::Status;
use rocket::serde::json::serde_json;
use rocket::serde::json::Value;

pub fn handle_user_payload(input: String) -> Result<Value, Status> {
    let trimmed = input.trim();

    let user_input = if trimmed.len() > 10 {
        trimmed.to_string()
    } else {
        "{}".to_string()
    };

    //SINK
    let user: Value = serde_json::from_str(&user_input)
        .map_err(|_| Status::BadRequest)?;

    Ok(user)
}
