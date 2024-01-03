use crate::schema::{users, wishes, friendships};

use diesel::prelude::*;
use serde::{Serialize, Deserialize};
use rocket::request::{FromRequest, Outcome};
use rocket::Request;
use rocket::http::Status;
use rocket::FromForm;



#[derive(Queryable, Insertable, Serialize, Deserialize, FromForm)]
#[diesel(table_name = users)]
pub struct User {
    pub user_id: String,
    pub name: String,
    pub username: String
}

#[derive(Queryable, Insertable, Serialize, Deserialize, FromForm)]
#[diesel(table_name = users)]
pub struct UserDto {
    pub user_id: String,
    pub name: String,
    pub username: String
}

#[derive(Queryable, Insertable, Serialize, Deserialize, Associations)]
#[diesel(belongs_to(User))]
#[diesel(table_name = wishes)]
pub struct Wish {
    pub wish_id: i32,
    pub description: String,
    pub access_level: String,
    pub user_id: String
}

#[derive(Queryable, Insertable, Serialize, Deserialize, Associations, FromForm)]
#[diesel(belongs_to(User))]
#[diesel(table_name = wishes)]
pub struct WishDto {
    pub description: String,
    pub access_level: String,
    pub user_id: String
}

#[derive(Queryable, Insertable, Serialize, Deserialize, FromForm)]
#[diesel(table_name = friendships)]
pub struct Friendship {
    pub friendship_id: i32,
    pub user_one: String,
    pub user_two: String,
    pub status: String
}

#[derive(Queryable, Insertable, Serialize, Deserialize, FromForm)]
#[diesel(table_name = friendships)]
pub struct FriendshipDto {
    pub user_one: String,
    pub user_two: String,
    pub status: String
}

pub struct UserSession {
    pub user_token: String
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for UserSession {
    type Error = ();

    async fn from_request(req: &'r Request<'_>) -> Outcome<UserSession, Self::Error> {
        let token = req.cookies().get("user_id").unwrap().value();

        let usr_token1 = token.to_string();
        println!("Your id: {}", usr_token1);

        if usr_token1.is_empty() {
            Outcome::Error((Status::Unauthorized, ()))
        } else {
            let session_user = UserSession {
                user_token: usr_token1,
            };
            Outcome::Success(session_user)
        }
    }
}