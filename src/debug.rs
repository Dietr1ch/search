use std::mem::size_of;

pub fn print_type_size<T>() {
    println!("~{}~ ({}B)", type_name::<T>(), size_of::<T>(),);
}

pub fn print_size<T: std::fmt::Debug>(t: &T) {
    println!(
        "~{}~ ({}B):\n#+begin_src ron\n{:?}\n#+end_src\n",
        type_name::<T>(),
        size_of::<T>(),
        t
    );
}

/// Returns a shorter version of [`std::any::type_name`]
// MIT from https://github.com/jakobhellermann/pretty-type-name/tree/main
pub fn type_name<T: ?Sized>() -> String {
    let name = std::any::type_name::<T>();
    type_name_str(name)
}

fn type_name_str(name: &str) -> String {
    if let Some(before) = name.strip_suffix("::{{closure}}") {
        return format!("{}::{{{{closure}}}}", type_name_str(before));
    }

    // code taken from [bevy](https://github.com/bevyengine/bevy/blob/89a41bc62843be5f92b4b978f6d801af4de14a2d/crates/bevy_reflect/src/type_registry.rs#L156)
    let mut short_name = String::new();

    // A typename may be a composition of several other type names (e.g. generic parameters)
    // separated by the characters that we try to find below.
    // Then, each individual typename is shortened to its last path component.
    //
    // Note: Instead of `find`, `split_inclusive` would be nice but it's still unstable...
    let mut remainder = name;
    while let Some(index) = remainder.find(&['&', '<', '>', '(', ')', '[', ']', ',', ';'][..]) {
        let (path, new_remainder) = remainder.split_at(index);
        // Push the shortened path in front of the found character
        short_name.push_str(path.rsplit(':').next().unwrap());
        // Push the character that was found
        let character = new_remainder.chars().next().unwrap();
        short_name.push(character);
        // Advance the remainder
        if character == ',' || character == ';' {
            // A comma or semicolon is always followed by a space
            short_name.push(' ');
            remainder = &new_remainder[2..];
        } else {
            remainder = &new_remainder[1..];
        }
    }

    // The remainder will only be non-empty if there were no matches at all
    if !remainder.is_empty() {
        // Then, the full typename is a path that has to be shortened
        short_name.push_str(remainder.rsplit(':').next().unwrap());
    }

    short_name
}
