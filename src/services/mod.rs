extern crate diesel;
extern crate rocket;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use dotenvy::dotenv;
use rocket::form::Form;
use rocket::http::CookieJar;
use rocket::response::Debug;
// use rocket::serde::json::Json;
use rocket::{get, post};
use crate::models::{self, User, UserSession, WishDto, Friendship, FriendshipDto};
use crate::schema::friendships::{user_one, user_two, status};
use crate::schema::users::user_id;
use crate::schema::wishes::access_level;
use rocket_dyn_templates::{context, Template};
use crate::schema::{self};
use std::env;



// connects to database
pub fn establish_connection_pg() -> PgConnection {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

type Result<T, E = Debug<diesel::result::Error>> = std::result::Result<T, E>;

// make userportal page
#[get("/home")]
pub fn home_page() -> Template {
    Template::render("home", context! {})
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

#[post("/login", format="form", data="<user>")]
pub fn login(jar: &CookieJar<'_>, user: Form<User>) -> Template {
    use self::models::Wish;
    use self::schema::wishes::user_id;
    use self::models::User;
    use self::schema::users::username;

    let connection = &mut establish_connection_pg();

    // checks to see if user exists
    let is_user = self::schema::users::dsl::users
        .filter(username.eq(user.username.to_string()))
        .load::<User>(connection)
        .expect("Error loading users");

    if is_user.is_empty() {
        Template::render("home", context! {})
    } else {
        let session_user_id = user.user_id.to_string();
        jar.add(("user_id", session_user_id.clone()));

        println!("{}", session_user_id);
    
        let results = self::schema::wishes::dsl::wishes
            .filter(user_id.eq(session_user_id))
            .load::<Wish>(connection)
            .expect("Error loading posts");
    
        Template::render("wishes", context! {wishes: &results})
    }
}

#[post("/logout")]
pub fn logout(jar: &CookieJar<'_>) -> Template {
    jar.remove("user_id"); //removes cookies

    Template::render("home", context! {})
}

#[post("/post", format = "form", data = "<wish>")]
pub fn create_wish(wish: Form<WishDto>, user_session: UserSession) -> Template {
    use self::schema::wishes::dsl::*;    

    let connection: &mut PgConnection = &mut establish_connection_pg();

    let user_token = &user_session.user_token;

    let new_wish = WishDto {
        description: wish.description.to_string(),
        access_level: wish.access_level.to_string(),
        user_id: user_token.to_string()
    };

    diesel::insert_into(wishes)
        .values(new_wish)
        .execute(connection)
        .expect("Error saving new wish");

    get_wishes(user_session)
}

#[get("/")]
pub fn get_wishes(user_session: UserSession) -> Template {
    use self::models::Wish;
    use self::schema::wishes::user_id; 

    let connection = &mut establish_connection_pg();

    let user_token = &user_session.user_token;

    // retrieves vector of user's friends
    let friendships = self::schema::friendships::dsl::friendships
        .filter(status.eq("Accepted"))
        .filter((user_one.eq(user_token)).or(user_two.eq(user_token)))
        .load::<Friendship>(connection)
        .expect("Error loading friendships");

    //creates vector of user's friends' ids
    let mut friend_ids:Vec<String> = Vec::new();

    for i in &friendships {
        friend_ids.push(i.user_one.to_string());
        friend_ids.push(i.user_two.to_string());
    }

    println!("{:?}", friend_ids);

    let results = self::schema::wishes::dsl::wishes
        .filter(user_id.eq(user_token)) 
        .or_filter(access_level.eq("public")) 
        .or_filter((user_id.eq_any(friend_ids)).and(access_level.eq("friends")))
        .load::<Wish>(connection)
        .expect("Error loading wishes");

    Template::render("wishes", context! {wishes: &results})
}

#[get("/friendships")]
pub fn get_friendships(user_session: UserSession) -> Template {
    use self::models::Friendship;
    use self::schema::friendships::user_one;

    let connection = &mut establish_connection_pg();

    let user_token = &user_session.user_token;

    let results = self::schema::friendships::dsl::friendships
        .filter(user_one.eq(user_token))
        .or_filter(user_two.eq(user_token))
        .load::<Friendship>(connection)
        .expect("Error loading friendships");

    let requests = self::schema::friendships::dsl::friendships
        .filter((user_two.eq(user_token)).and(status.eq("pending")))
        .load::<Friendship>(connection)
        .expect("Error loading friendships");

    Template::render("friendships", context! {friendships: &results, requests: &requests})
}

#[post("/post_friendship", format="form", data="<friendship>")]
pub fn create_friendship_request(friendship: Form<FriendshipDto>, user_session: UserSession) -> Template {
    use self::schema::friendships::dsl::friendships;

    let connection = &mut establish_connection_pg();

    let user_token = user_session.user_token;

    // checks to see if requested user exists
    let requested_user = self::schema::users::dsl::users
        .filter(user_id.eq(friendship.user_two.to_string()))
        .load::<User>(connection)
        .expect("Error retrieving user");

    if requested_user.is_empty() {
        Template::render("friendships", context! {})
    } else  {
        let new_friendship = FriendshipDto {
            user_one: user_token.to_string(),
            user_two: friendship.user_two.to_string(),
            status: friendship.status.to_string()
        };

        diesel::insert_into(friendships)
            .values(new_friendship)
            .execute(connection)
            .expect("Friendship failed");

        let results = self::schema::friendships::dsl::friendships
            .filter(user_one.eq(user_token))
            .load::<Friendship>(connection)
            .expect("Error loading friendships");
    
        Template::render("friendships", context! {friendships: &results})
    }
}

#[post("/change_friendship", format="form", data="<friendship>")]
pub fn change_friendship_status(friendship: Form<FriendshipDto>, user_session: UserSession) -> Template {
    use self::schema::friendships::dsl::*;

    let connection = &mut establish_connection_pg();

    diesel::update(friendships)
        .filter((user_one.eq(friendship.user_one.to_string())).and(user_two.eq(friendship.user_two.to_string()))) //matches to friendship in table
        .set(status.eq(&friendship.status))
        .execute(connection)
        .expect("Error updating status");

    get_friendships(user_session)
}