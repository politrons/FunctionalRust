/**
This example shows the idiomatic Rust way to replace Mockito-style mocks.

`UserService` does not depend on a concrete database implementation.
It depends on the `UserRepo` trait, so production code can use `OracleRepo`
and tests can provide a small `MockUserRepo`.

The mock is just another struct implementing the same trait. No external mocking
framework is required for this simple case.
*/

trait UserRepo {
    fn find_user_by_id(&self, id: &str) -> String;
}

struct UserService<T:UserRepo> {
    user_repo: T,
}

impl<T: UserRepo> UserService<T> {
    fn find(&self, id:&str) -> String {
        self.user_repo.find_user_by_id(id)
    }
}

struct OracleRepo;

impl UserRepo for OracleRepo {
    fn find_user_by_id(&self, id:&str) -> String {
        //Find in DB
        "real_user".to_string()
    }
}

mod test {
    use super::*;

    #[test]
    fn dev_user_repo() {

        struct MockUserRepo;
        impl UserRepo for MockUserRepo {
            fn find_user_by_id(&self, id: &str) -> String {
                String::from("Politrons")
            }
        }
        let service  = UserService {user_repo:MockUserRepo};
        let user = service.find("foo");
        println!("User found {}",user);
    }

    #[test]
    fn prod_user_repo() {
        let service  = UserService {user_repo:OracleRepo};
        let user = service.find("foo");
        println!("User found {}",user);
    }
}