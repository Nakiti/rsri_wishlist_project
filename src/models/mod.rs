use crate::schema::{friendships, users, wishes};

use diesel::associations::HasTable;
use diesel::prelude::*;
// use rocket::data::Outcome;
// use rocket::data::Outcome;
use serde::{Serialize, Deserialize};
use rocket::request::{FromRequest, Outcome};
use rocket::Request;
use rocket::http::Status;
use rocket::FromForm;
use crate::services::establish_connection_pg; 
// use diesel::{Insertable, Queryable};
use rdiesel::ContextImpl;

impl HasTable for User {
    type Table = crate::schema::users::table;

    fn table() -> Self::Table {
        crate::schema::users::table
    }
}

// impl AuthProvider for &User {
//     type User = User;

//     fn authenticate(&self) -> Option<Self::User> {
//         Some((*self).clone())
//     }
// }

#[derive(Queryable, Insertable, Serialize, Deserialize, FromForm, Selectable, Clone)]
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

impl HasTable for Wish {
    type Table = crate::schema::wishes::table;

    fn table() -> Self::Table {
        crate::schema::wishes::table
    }
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

impl HasTable for WishDto {
    type Table = crate::schema::wishes::table;

    fn table() -> Self::Table {
        crate::schema::wishes::table
    }
}
#[derive(Queryable, Insertable, Serialize, Deserialize, Associations, FromForm)]
#[diesel(belongs_to(User))]
#[diesel(table_name = wishes)]
pub struct WishDto {
    pub description: String,
    pub access_level: String,
    pub user_id: String
}

impl HasTable for Friendship {
    type Table = crate::schema::friendships::table;

    fn table() -> Self::Table {
        crate::schema::friendships::table
    }
}
#[derive(Queryable, Insertable, Serialize, Deserialize, FromForm)]
#[diesel(table_name = friendships)]
pub struct Friendship {
    pub friendship_id: i32,
    pub user_one: String,
    pub user_two: String,
    pub status: String
}

impl HasTable for FriendshipDto {
    type Table = crate::schema::friendships::table;

    fn table() -> Self::Table {
        crate::schema::friendships::table
    }
}
#[derive(Queryable, Insertable, Serialize, Deserialize, FromForm)]
#[diesel(table_name = friendships)]
pub struct FriendshipDto {
    pub user_one: String,
    pub user_two: String,
    pub status: String
}


pub struct UserSession {
    pub user: User,
    pub connection: diesel::pg::PgConnection,
}

impl UserSession {
    pub fn into_context(self) -> Context {
        Context::new(self)
    }
}

type Context = rdiesel::Context<UserSession, User>;

impl ContextImpl for UserSession {
    type User = User;
    type Conn = diesel::pg::PgConnection;

    fn auth_user(&self) -> User {
        self.user.clone()
    }

    fn conn(&mut self) -> &mut Self::Conn {
        &mut self.connection
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for UserSession {
    type Error = ();

    async fn from_request(req: &'r Request<'_>) -> Outcome<UserSession, Self::Error> {
        let token = req.cookies().get("user_id").unwrap().value();

        let usr_token1 = token.to_string();

        println!("Your id: {}", usr_token1);

        let mut connection = establish_connection_pg();

        if usr_token1.is_empty() {
            Outcome::Error((Status::Unauthorized, ()))
        } else {
            // let session_user = UserSession {
            //     user_token: usr_token1,
            //     connection,
            // };
            let user = users::table
                .filter(users::user_id.eq(usr_token1))
                .first(&mut connection)
                .ok()
                .expect("Error");

            Outcome::Success(UserSession {user, connection})
        }
    }
}