#![allow(dead_code)]

// Example 1: Foo WITHOUT Drop, This compiles
struct FooNoDrop<'a>(&'a mut bool);

fn test_no_drop() {
    let mut x = true;
    let _foo = FooNoDrop(&mut x);
    x = false; // This should be allowed because foo never uses the reference again
    println!("x is now: {}", x);
}

// Example 2: Foo WITH Drop. This does NOT compile.
struct FooWithDrop<'a>(&'a mut bool);

impl Drop for FooWithDrop<'_> {
    fn drop(&mut self) {}
}

fn test_with_drop() {
    let mut x = true;
    let mut _foo = FooWithDrop(&mut x);
    // x = false; // This should be rejected because Drop will access the reference
    // println!("x is now: {}", x);
}

fn main() {
    println!("Testing no drop case:");
    test_no_drop();

    // Uncomment the line below to see the compilation error
    println!("\nTesting with drop case:");
    test_with_drop();
}
