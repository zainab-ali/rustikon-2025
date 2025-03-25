trait Animal {}

struct Sheep;
struct Dog;

impl Animal for Sheep {}
impl Animal for Dog {}

fn sheep() -> impl Animal {
    Sheep
}

fn dog() -> impl Animal {
    Dog
}

fn sheep_dog(t: bool) -> Box<dyn Animal> {
    if t {
        Box::new(sheep())
    } else {
        Box::new(dog())
    }
}

fn main() {}
