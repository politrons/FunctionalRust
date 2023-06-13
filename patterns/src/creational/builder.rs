///Data type that we want to build
#[derive(Debug)]
pub struct Human {
    age: u32,
    name: String,
    sex: String,
}


///Builder data type, that we will implement the [Builder pattern]
pub struct HumanBuilder {
    with_age: Option<u32>,
    with_name: Option<String>,
    with_sex: Option<String>,

}

///Implementation of [Builder pattern]. We use the [HumanBuilder] type to keep all the temporal data with [Option] types,
/// so then we can control filled and unfilled fields. Then once we use [build] function, we transform the Builder object
/// into the final Data type.
 impl HumanBuilder {
    pub fn new() -> Self {
        return HumanBuilder { with_age: None, with_name: None, with_sex: None };
    }

    pub fn with_age(self, age: u32) -> Self {
        return HumanBuilder { with_age: Some(age), with_name: self.with_name, with_sex: self.with_sex };
    }

    pub fn with_name(self, name: String) -> Self {
        return HumanBuilder { with_age: self.with_age, with_name: Some(name), with_sex: self.with_sex };
    }

    pub fn with_sex(self, sex: String) -> Self {
        return HumanBuilder { with_age: self.with_age, with_name: self.with_name, with_sex: Some(sex) };
    }

    pub fn build(self) -> Human {
        let s = self.with_age;
        return Human { age: self.with_age.unwrap_or_default(), name: self.with_name.unwrap_or_default(), sex: self.with_sex.unwrap_or_default() };
    }
}

#[cfg(test)]
mod tests {
    use crate::creational::builder::HumanBuilder;

    #[test]
    fn builder_pattern() {
        let human = HumanBuilder::new()
            .with_name("Politrons".to_string())
            .with_age(42)
            .with_sex("Male".to_string())
            .build();

        println!("${:?}", human);
        assert_eq!(human.name, "Politrons");
    }
}