use std::fmt::Display;

use actix_session::Session;
use serde::{Deserialize, Serialize};
use sha2::{Sha512, Digest};

const SESSION_USERNAME_KEY: &'static str = env!("SESSION_USERNAME_KEY");

#[derive(Serialize, Deserialize, Debug)]
pub struct User{
    pub username: String,
    pub nickname: String,
    pub profile_picture_filename: Option<String>
}

pub enum UsernameConstraintError{
    LengthError, // not between 6 to 20
    CharacterError, // contains non-alphanumeric characters, except for '.' and '_'.
}

impl Display for UsernameConstraintError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match &self{
            Self::LengthError => "Username should between 6 to 20 characters long.",
            Self::CharacterError => "Username should be consisted with only alphanumeric, period or underscore character.",
        };
        f.write_str(message)
    }
}

pub enum PasswordConstraintError{
    LengthError, // at least 8 characters
    CombinationError, // should be a combination of upper/lowercases and numbers (symbol are not required).
}

impl Display for PasswordConstraintError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match &self{
            Self::LengthError => "Password should at least 8 characters.",
            Self::CombinationError => "Password should be a combination of upper/lowercases and numbers.",
        };
        f.write_str(message)
    }
}

impl User{
    pub fn encrypt_password(plain_password: &str) -> String{
        const PASSWORD_ENCRYPTION_SALT: &str = env!("PASSWORD_ENCRYPTION_SALT");

        let mut hasher = Sha512::new();
        hasher.update(plain_password);
        hasher.update(PASSWORD_ENCRYPTION_SALT);

        format!("{:x}", hasher.finalize())
    }

    pub fn check_username_constraint(username: &str) -> Result<(), UsernameConstraintError>{
        // Username must be between 6 to 20 lowercase alphanumeric, period or underscore characters long.
        const ALLOWED_CHARACTERS: &str = "abcdefghijklmnopqrstuvwxyz0123456789._";

        if !(6..=20).contains(&username.len()){
            Err(UsernameConstraintError::LengthError)
        }
        else if !username.chars().all(|c| ALLOWED_CHARACTERS.contains(c)){
            Err(UsernameConstraintError::CharacterError)
        }
        else{
            Ok(())
        }
    }

    pub fn check_password_constraint(password: &str) -> Result<(), PasswordConstraintError>{
        // Password must be at least 8 characters long and be consisted of at least one uppercase/lowercase alphabet and digit.
        if password.len() < 8{
            return Err(PasswordConstraintError::LengthError);
        }

        let mut contains_lowercase = false;
        let mut contains_uppercase = false;
        let mut contains_number = false;
        for char in password.chars(){
            if char.is_ascii_lowercase(){
                contains_lowercase = true;
            }
            else if char.is_ascii_uppercase(){
                contains_uppercase = true;
            }
            else if char.is_ascii_digit(){
                contains_number = true;
            }
        }

        if contains_lowercase && contains_uppercase && contains_number{
            Ok(())
        }
        else{
            Err(PasswordConstraintError::CombinationError)
        }
    }

    pub fn add_username_into_session(&self, session: Session){
        session.insert(SESSION_USERNAME_KEY, &self.username).unwrap()
    }

    pub fn get_username_from_session(session: Session) -> Option<String>{
        session.get::<String>(SESSION_USERNAME_KEY).unwrap()
    }

    pub fn expire_session(session: Session){
        session.remove(SESSION_USERNAME_KEY);
    }
}