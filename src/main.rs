
mod c_things {
    extern "C" {
        pub fn my_thing();
    }
}

fn my_thing() {
    unsafe {
        c_things::my_thing();
    }
}

fn main() {
    my_thing();
}
