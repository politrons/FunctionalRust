// To use the `derive(Lenses)` macro
use pl_lens::Lenses;

// To use the `lens!` macro
use pl_lens::lens;

use pl_lens::{Lens};

#[derive(Lenses, Debug)]
struct Human {
    name: String,
    age: u32,
    skills: Skills,
}

#[derive(Lenses, Debug)]
struct Skills {
    name: String,
    level: Level,
}

#[derive(Lenses, Debug)]
struct Level {
    value: u32,
}

/// [pl-lens]  Library allow to introduce in rust lens feature to being able to create copies of Data structure,
/// only changing the fields that we want to change, and keep copies of the rest of fields, allowing immutability
/// in a very clean and elegant way.
/// In normal conditions in case you want to embrace immutability, you would need to create new Data type, and pass
/// all old arguments to the new one, and modify the fields that you want to change.
/// Using [lens!] macro, we can pass as argument the Data Type followed by the structure of field that you want to change.
/// [DataType.field_to_change] then using [set] operator, we pass the instance of the type to create a new copy, and finally
/// the new value.
/// Inner data types changes is also allowed [DataType.another_data_type.field_to_change]
/// After we use the lens [set] human is borrowed so we can not longer use the previous variable of human.
fn change_human() {
    let level = Level {
        value: 100
    };
    let skills = Skills {
        name: "Hunter".to_string(),
        level,
    };
    let human = Human {
        name: "politrons".to_string(),
        age: 42,
        skills,
    };

    let new_human = lens!(Human.name)
        .set(human, "POLITRON".to_string());
    println!("${:?}", new_human);

    let skill_human = lens!(Human.skills.name)
        .set(new_human, "Collector".to_string());
    println!("${:?}", skill_human);

    let level_human = lens!(Human.skills.level.value)
        .set(skill_human, 1981);
    println!("${:?}", level_human);

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lens() {
        change_human()
    }
}
