use rumbok::AllArgsConstructor;
use serde::Serialize;

#[derive(Serialize, AllArgsConstructor)]
pub struct Email {
    id: usize,
    content: String,
}
