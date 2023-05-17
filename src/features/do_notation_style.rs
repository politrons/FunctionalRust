use do_notation::m;

pub fn run() {
    option_program();
    result_program();
}

fn option_program() {
    let maybe_user_info = m! {
          user <- login("politrons");
          account <- get_user_account(user);
          get_user_info(account)
    };
    println!("{:?}", maybe_user_info)
}

fn result_program() {
    let result_user_info: Result<Account, String> = m! {
          user <- Ok(User("Politrons".to_string()));
          account <- Ok(Account { info: user.0 });
          Ok(account)
    };
    println!("{:?}", result_user_info.unwrap().info)
}

fn login(username: &str) -> Option<User> {
    Some(User(username.to_string()))
}

fn get_user_account(user: User) -> Option<Account> {
    Some(Account { info: user.0 })
}

fn get_user_info(account: Account) -> Option<String> {
    Some(account.info)
}

struct User(String);

struct Account {
    info: String,
}