# Simple example linking a C library to a binary

This is a simple example of linking a C library (contained in this package) with Autotools. While this is a toy example, there exist many packages that use Autotools for their build system.

## Package Layout

This repo has a normal Rust package layout, with the Rust source code under `src`, a `Cargo.toml`, etc. The C package is called `mypkg` and contains a single function `my_thing`, which just prints to the console. It's not a particularly well-formed package since it doesn't include a header file.

There's a `build.rs` build script that contains a simplistic procedure for determining if the autotools package has a `configure` script present and runs `autoreconf` if it doesn't. Once the configure script exists, it runs the usual `configure` and `make` commands through the `autotools` package.

## Linking

Just building the C library is not enough, we also have to link it into our Rust binary so that we can call the correct functions. In order to have a binary without other dependencies, we're going to use static linking (the default for most Rust packages).

We need to tell the Rust compiler that during the linking stage, it should also include the `libmypkg.a` library that got built by the build script. To do this, we included the following lines:

```
println!("cargo:rustc-link-search=native={}/lib", dst.display());
println!("cargo:rustc-link-lib=static=mypkg");
```

The first of these lines tells the Rust compiler where to go looking for this library. The `autotools` package provides a method `display()` to get this information. Since `display()` was missing the `lib` subdirectory, that's added on at the end.

The second of these lines tells the Rust compiler what library to link. The normal rules for linking on Linux are that a library in a file called `libmypkg.a` is referenced by the linker as `mypkg` (without the leading 'lib' prefix or '.a' suffix).

## Using our function

Now that linking is set up, the compiler can find the correct definitions. Since I wanted a simple example of linking, I didn't set up a header file. For a "real" library, you probably want to look at `bindgen` or other tools that can generate the declarations below automatically.

First we have to tell our Rust code about our C function.

```
extern "C" {
    pub fn my_thing();
}
```

This tells the compiler that the `my_thing` function exists and it's using C linkage so no name mangling happens. You can then call this funciton with:
```
fn main() {
    unsafe {
        my_thing();
    }
}
```

That's pretty neat. When I run this, I get the following output:
```
$ cargo run
hello, thing
```

## Making our function ergonomical

Calling a C function in Rust is always `unsafe`. The compiler can't guarantee what a function does and a C function can break every safety rule that Rust has. But in this case, we know that the function just calls `printf` and this is single-threaded code so we don't have to worry about ordering or any other details.

The easiest way of making this ergonomical is to make a function that wraps `my_thing`. We might call this `rust_my_thing()` and write it like this:

```
fn rust_my_thing() {
    unsafe {
        my_thing();
    }
}
```

Now we can call `rust_my_thing()` anywhere as a normal Rust function without any `unsafe` block. Better! However it's kind of annoying to have to change the name of the function (otherwise the compiler couldn't tell _which_ `my_thing` we want to call).

Thankfully, Rust has namespaces (modules) and one cool thing is that we can put our C function in a module so that we can re-use the `my_thing` name.

```
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
```

Nice! Now we have two `my_thing` functions: the original C function that we put in the `c_things` module and a `my_thing` Rust function in the root namespace. Of course if this were a real package, we would want to put our function in its own module too.

## Debugging our library

Rust does many compile time checks, but C code can be complied with pretty lax checking. Our single-function library is simple, but what if something goes wrong during execution? We won't get a Rust traceback, since it's not Rust code.

C and C++ developers are likely already familiar with `gdb`. This is a great tool for debugging compiled programs. `gdb` supports Rust nicely but there are a couple caveats to note.

First if we put a breakpoint on `my_thing`, `gdb` doesn't know which one we wanted and so it puts a breakpoint on _both_ the C `my_thing` _and_ the Rust `my_thing`. This is a bit annoying:

```
gdb target/debug/wrapped-package

(gdb) b my_thing
Breakpoint 1 at 0x7d01: my_thing. (2 locations)

(gdb) info break
Num     Type        Disp Enb Address             What
1       breakpoint  keep y   <MULTIPLE>
1.1                      y   0x0000000000007d01 in wrapped_package::my_thing at src/main.rs:10
1.2                      y   0x0000000000007d89 in my_thing at /path/to/mypkg/thing.c:4
```

Now if I step through my program, I'll hit both. Again, in this case that's not too bad (they're only called twice), but if this were a bigger application with many methods that had simililar names, it would be very annoying.

Thankfully, we can pass an additional argument `-q` when we make our breakpoint to ensure that it's _qualified_ properly. This way we can specify which function we want:

```
(gdb) b -q my_thing
Breakpoint 2 at 0x7d89: file /path/to/mypkg/thing.c, line 4.

(gdb) b -q wrapped_package::my_thing
Breakpoint 3 at 0x7d01: file src/main.rc, line 10.

(gdb) info break
1       breakpoint  keep y   <MULTIPLE>
1.1                      y   0x0000000000007d01 in wrapped_package::my_thing at src/main.rs:10
1.2                      y   0x0000000000007d89 in my_thing at /path/to/mypkg/thing.c:4
2                        y   0x0000000000007d89 in my_thing at /path/to/mypkg/thing.c:4
3                        y   0x0000000000007d01 in wrapped_package::my_thing at src/main.rs:10
```

And we're done! We can now set breakpoints where we want and debug our code.
