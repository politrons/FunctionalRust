/**
Unsafe Rust should be small, explicit, and documented. The goal is to expose
a safe API around carefully constrained unsafe blocks.
*/


#[cfg(test)]
mod tests {

    #[test]
    pub fn run() {
       let first =  unsafe {[1,2,3].get_unchecked(0)} ;

        println!("Unsafe first value:{}", first);
    }

}
