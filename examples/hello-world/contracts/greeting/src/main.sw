contract;

use std::logging::log;

struct Person {
    name: str[32],
}

struct Greeting {
    id: u64,
    greeting: str[32],
    person: Person,
}

abi Greet {
    fn new_greeting(id: u64, greeting: str[32], person_name: str[32]);
}

impl Greet for Contract {
    fn new_greeting(id: u64, greeting: str[32], person_name: str[32]) {
        log(Greeting{ id, greeting, person: Person{ name: person_name }});
    }
}
