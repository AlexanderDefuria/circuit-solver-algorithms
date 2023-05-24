use wasm_bindgen::prelude::*;

pub mod components;
pub mod container;
pub mod elements;
pub mod tools;
pub mod validation;
mod tests;

#[wasm_bindgen]
pub fn solve() -> String {
    String::from("Hello, world!")
}

#[wasm_bindgen]
pub fn add(x: i32, y: i32) -> i32 {
    x + y
}

pub fn simplify() {}

pub fn serialize() {}
pub fn deserialize() {}
