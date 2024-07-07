use serenity::all::UserId;

#[derive(Debug, Clone)]
pub struct UserRaw {
    pub id: i64,
    pub email: String,
}

impl From<User> for UserRaw {
    fn from(value: User) -> Self {
        UserRaw {
            id: value.id.get() as i64,
            email: value.email,
        }
    }
}

#[derive(Debug, Clone)]
pub struct User {
    pub id: UserId,
    pub email: String,
}

impl From<UserRaw> for User {
    fn from(value: UserRaw) -> Self {
        User {
            id: UserId::new(value.id as u64),
            email: value.email,
        }
    }
}