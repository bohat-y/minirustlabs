// 1. Enum for ticket category
#[derive(Debug)]
enum Category {
    Infant,
    Child,
    Adult,
    OAP,
}

// 2. Age -> Category
fn age_to_category(age: u32) -> Category {
    match age {
        0..=2   => Category::Infant,
        3..=16  => Category::Child,
        17..=65 => Category::Adult,
        _       => Category::OAP,
    }
}

// 3. Category -> Price
fn ticket_price(category: &Category) -> f64 {
    match category {
        Category::Infant => 0.00,
        Category::Child  => 13.00,
        Category::Adult  => 17.50,
        Category::OAP    => 13.50,
    }
}

// 4. Person struct
#[derive(Debug)]
struct Person {
    name: String,
    age:  u32,
}

impl Person {
    fn new(name: &str, age: u32) -> Self {
        Person { name: name.to_string(), age }
    }
}

fn main() {
    // 5. Vector of persons - family with 2 children
    let family: Vec<Person> = vec![
        Person::new("Alice",  42),
        Person::new("Bob",    45),
        Person::new("Charlie", 10),
        Person::new("Diana",   7),
    ];

    let family2: Vec<Person> = vec![
        Person {name: String::from("Alice"), age: 42},
        Person {name: String::from("Bob"),  age: 10},
    ];

    println!("Family:");
    for p in &family {
        println!("  {:?}", p);
    }

    // 6. Total ticket cost via fold()
    let total = family.iter().fold(0.0_f64, |acc, p| {
        acc + ticket_price(&age_to_category(p.age))
    });


    println!("\nTotal ticket price: €{:.2}", total);


    // 7. Children only via filter()
    let children: Vec<&Person> = family
        .iter()
        .filter(|p| matches!(age_to_category(p.age), Category::Child))
        .collect();

    println!("\nChildren:");
    for c in &children {
        println!("  {:?}", c);
    }
}
