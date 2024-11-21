#[derive(Clone)]
pub struct User 
{
    user_id: String,
    username: String,
    password: String,
}

impl User 
{
    pub fn new(user_id: String, username: String, password: String) -> Self {
        Self {
            user_id,
            username,
            password,
        }
    }

    pub fn get_user_id(&self) -> String {
        self.user_id.clone()
    }

    pub fn get_username(&self) -> String {
        self.username.clone()
    }

    pub fn get_password(&self) -> String {
        self.password.clone()
    }
}
