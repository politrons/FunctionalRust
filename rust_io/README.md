# Rust_IO

Macro implementation for [rust_io] defining several operators to be used emulating Haskel [do notation]


```rust
        let rio_program: RustIO<String, String> = rust_io! {
             _ <- RustIO::of(String::from("1981"));
             v <- RustIO::from_option(Some(String::from("hello")));
             t <- RustIO::from_option_func(|| Some(String::from(" pure")));
             z <- RustIO::from_func(|| String::from(" functional"));
             x <- RustIO::from_result(Ok(String::from(" world")));
             i <- RustIO::of(String::from("!!"));
             y <- RustIO::from_result_func(|| Ok(String::from("!!")));

             yield v + &t + &z + &x + &i + &y;
        };
        println!("${:?}", rio_program.is_empty());
        println!("${:?}", rio_program.is_ok());
        assert_eq!(rio_program.get(), "hello pure functional world!!!!");
```