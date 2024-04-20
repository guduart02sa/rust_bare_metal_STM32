//The code fails because it tries to use a string slice from a String object that is immediately deallocated,
//causing a dangling reference. Rust’s safety rules prevent this, leading to a compilation error.
//To solve this, we just separete the operations.
fn ret_string() -> String {
    String::from("  A String object  ")
}

//==================  Problem 2 ==================//
//It's necessary to unify lifetimes for this
fn choose_str<'a>(s1: &'a str, s2: &'a str, select_s1: bool) -> &'a str {
    if select_s1 {
        s1
    } else {
        s2
    }
}

//==================  Problem 3 ==================//
use std::ops::{Deref, DerefMut};

//It will require using a generic parameter. What does it represent?

//The generic parameter represents the lifetime of the borrowed string slice.

enum Oor<'a> {
    Owned(String),  
    Borrowed(&'a str),
}

/* Implement the Deref trait for the OOR structure so that it dereferences into an a &str. 
What is the lifetime of the resulting &str (note that you have no choice there)? 
Why is that always appropriate? */

//The lifetime of the resulting &str is the same as the lifetime of the borrowed string slice. You can also tell that is in line 45.
//This is "always" appropriate because the lifetime of the borrowed string slice is guaranteed to be valid for the duration of the Oor instance.

impl<'a> Deref for Oor<'a> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        match self {
            Oor::Owned(s) => s,
            Oor::Borrowed(s) => s,
        }
    }
}


impl<'a> DerefMut for Oor<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        if let Oor::Borrowed(s) = self {
            *self = Oor::Owned(s.to_string());
        }

        match self {
            Oor::Owned(s) => s,
            Oor::Borrowed(_) => unreachable!(), // This should never happen due to the above mutation
        }
    }
}

#[test]
fn test() {
    // Check Deref for both variants of OOR
    let s1 = Oor::Owned(String::from("  Hello, world.  "));
    assert_eq!(s1.trim(), "Hello, world.");
    let mut s2 = Oor::Borrowed("  Hello, world!  ");
    assert_eq!(s2.trim(), "Hello, world!");

    // Check choose
    let s = choose_str(&s1, &s2, true);
    assert_eq!(s.trim(), "Hello, world.");
    let s = choose_str(&s1, &s2, false);
    assert_eq!(s.trim(), "Hello, world!");

    // Check DerefMut, a borrowed string should become owned
    assert!(matches!(s1, Oor::Owned(_)));
    assert!(matches!(s2, Oor::Borrowed(_)));
    unsafe {
        for c in s2.as_bytes_mut() {
            if *c == b'!' {
                *c = b'?';
            }
        }
    }
    assert!(matches!(s2, Oor::Owned(_)));
    assert_eq!(s2.trim(), "Hello, world?");
}
// Passed the test :)

fn main() {
    let temp = ret_string();
    let s = temp.trim();
    assert_eq!(s, "A String object");
}
