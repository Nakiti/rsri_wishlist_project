extern crate diesel;
extern crate rocket;
use diesel::pg::PgConnection;

use dotenvy::dotenv;
use rocket::form::Form;
use rocket::http::CookieJar;
use rocket::post;
use crate::models::User;
use rocket_dyn_templates::{context, Template};
use crate::schema::{self};
use std::env;
use rdiesel::{select_list, Expr};
use diesel::{RunQueryDsl, Connection};


// connects to database
pub fn establish_connection_pg() -> PgConnection {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

#[post("/register", format="form", data = "<user>")]
pub fn create_user(jar: &CookieJar<'_>, user: Form<User>) -> Template {
    use self::schema::users::dsl::*;

    let connection: &mut PgConnection = &mut establish_connection_pg();

    let new_user = User {
        user_id: user.user_id.to_string(),
        name: user.name.to_string(),
        username: user.username.to_string()
    };

    diesel::insert_into(users)
        .values(new_user)
        .execute(connection)
        .expect("Error saving new user");

    let session_id = user.user_id.to_string();
    jar.add(("user_id", session_id.clone())); //add user_id to cookies

    Template::render("wishes", context! {})
}

// impl rdiesel::Expr::<User, String> for schema::users::username {}

#[post("/login", format="form", data="<user>")]
pub fn login(jar: &CookieJar<'_>, user: Form<User>) -> Template {
    use self::schema::wishes::user_id;
    use self::schema::users::username;

    let connection: &mut PgConnection = &mut establish_connection_pg();

    // checks to see if user exists
    let user_q1 = username.eq(user.username.to_string());
    let is_user = select_list(connection, user_q1).expect("Error retrieving user");

    // let is_user = self::schema::users::dsl::users
    //     .filter(username.eq(user.username.to_string()))
    //     .load::<User>(connection)
    //     .expect("Error loading users");

    if is_user.is_empty() {
        Template::render("home", context! {})
    } else {
        let session_user_id = user.user_id.to_string();
        jar.add(("user_id", session_user_id.clone()));
        // jar.add(("connection", connection));

        println!("{}", session_user_id);
        
        let results_q1 = user_id.eq(session_user_id.to_string());
        let results = select_list(connection, results_q1).expect("Error retrieving results");

        // let results = self::schema::wishes::dsl::wishes
        //     .filter(user_id.eq(session_user_id))
        //     .load::<Wish>(connection)
        //     .expect("Error loading posts");
    
        Template::render("wishes", context! {wishes: &results})
    }
}

#[post("/logout")]
pub fn logout(jar: &CookieJar<'_>) -> Template {
    jar.remove("user_id"); //removes cookies

    Template::render("home", context! {})
}