#![cfg(test)]

use crate::todo_list::{Todolist, TodolistClient};

use super::*;
use soroban_sdk::{vec, Env, String};

fn setup() -> (Env, TodolistClient<'static>) {
    let env = Env::default();
    let contract_id = env.register(Todolist, ());
    let client = TodolistClient::new(&env, &contract_id);

    (env, client)
}
#[test]
fn test() {
    let (env, client) = setup();

    let title = String::from_str(&env, "Go home!!!");

    let description = String::from_str(&env, "From Garage to the hostel");

    let words = client.create_todo(&title, &description);

    let all_todo = client.get_todos();

    assert_eq!(all_todo.len(), 1);
    assert_eq!(words.description, description);
    assert_eq!(words.title, title);
    assert_eq!(words.id, 1);
    assert!(!words.status);
}

#[test]
fn test_delete() {
    let (env, client) = setup();

    let title = String::from_str(&env, "Go home!!!");

    let id = 1_u32;

    let description = String::from_str(&env, "From Garage to the hostel");

    client.create_todo(&title, &description);

    client.delete_todo(&id);

    let all_todo = client.get_todos();

    assert_eq!(all_todo.len(), 0);
}

#[test]
fn text_update() {
    let (env, client) = setup();

    let title = String::from_str(&env, "Go home!!!");
    let title2 = String::from_str(&env, "Home alone");

    let id = 1_u32;

    let description = String::from_str(&env, "From Garage to the hostel");
    let description = String::from_str(&env, "From Garage to the hostel");
    let description2 = String::from_str(&env, "Go hard or Go home");

    client.create_todo(&title, &description);
    let update = client.update_todo2(&id, &title2, &description2);

    let all_todos = client.get_todos();
    let all_todo = client.get_todo(&id);

    assert_eq!(update, true);
    // assert_eq!(all_todos.get(0).unwrap().description, description2);
    assert_eq!(all_todo.description, description2);
}
