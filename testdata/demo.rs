use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct User {
    pub name: String,
    pub age: u32,
    email: String,
}

impl User {
    pub fn new(name: String, age: u32, email: String) -> Self {
        Self { name, age, email }
    }

    pub fn greeting(&self) -> String {
        format!("Hello, {}!", self.name)
    }

    fn validate_email(&self) -> bool {
        self.email.contains('@')
    }
}

fn helper() {
    println!("I'm a helper");
}
