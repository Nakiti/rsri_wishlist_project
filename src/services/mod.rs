extern crate diesel;
extern crate rocket;
use diesel::pg::PgConnection;

use dotenvy::dotenv;
use rocket::form::Form;
use rocket::response::Debug;
use rocket::{get, post};
use crate::models::{User, UserSession, WishDto, Wish, Friendship, FriendshipDto};
use crate::schema::friendships::{status, user_one, user_two};
use crate::schema::users::user_id;
use crate::schema::wishes::access_level;
use rocket_dyn_templates::{context, Template};
use crate::schema::{self};
use std::env;
use rdiesel::{Expr, Field};
use diesel::Connection;
use rdiesel::update_where;

// connects to database
pub fn establish_connection_pg() -> PgConnection {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

type Result<T, E = Debug<diesel::result::Error>> = std::result::Result<T, E>;



impl rdiesel::Expr::<User, String> for schema::users::username {}

#[post("/post", format = "form", data = "<wish>")]
pub fn create_wish(wish: Form<WishDto>, user_session: UserSession) {
    //use self::schema::wishes::dsl::*;    

    let mut cx = user_session.into_context();
    let auth_user = cx.auth_user();

    //let connection: &mut PgConnection = &mut establish_connection_pg();

    let user_token = auth_user.user_id;

    let new_wish = WishDto {
        description: wish.description.to_string(),
        access_level: wish.access_level.to_string(),
        user_id: user_token
    };

    let _ = cx.insert(new_wish);

    // diesel::insert_into(wishes)
    //     .values(new_wish)
    //     .execute(connection)
    //     .expect("Error saving new wish");

    // get_wishes(user_session);
}

impl rdiesel::Expr<Friendship, i32> for schema::friendships::status {}
impl rdiesel::Expr<Friendship, i32> for schema::friendships::user_one {}
impl rdiesel::Expr<Friendship, i32> for schema::friendships::user_two {}

impl rdiesel::Expr<Wish, i32> for schema::wishes::user_id {}
impl rdiesel::Expr<Wish, String> for schema::wishes::access_level {}

#[get("/")]
pub fn get_wishes(user_session: UserSession) -> Template {
    //use self::models::Wish;
    use self::schema::wishes::user_id; 

    let mut cx = user_session.into_context();

    let auth_user = cx.auth_user();

    //let connection = &mut establish_connection_pg();

    let user_token = auth_user.user_id;
    let q1 = status.eq(1);
    let q2 = user_one.eq(user_token);
    let q3 = user_two.eq(user_token);
    let q4 = q2.or(q3);
    let q5 = q1.and(q4);

    let friendships = cx.select_list(q5).unwrap();
    // let friendships = select_list(connection, q5);

    // retrieves vector of user's friends
    // let friendships = self::schema::friendships::dsl::friendships
    //     .filter(status.eq("Accepted"))
    //     .filter((user_one.eq(user_token)).or(user_two.eq(user_token)))
    //     .load::<Friendship>(connection)
    //     .expect("Error loading friendships");

    //creates vector of user's friends' ids
    let mut friend_ids:Vec<i32> = Vec::new();

    for i in friendships {
        friend_ids.push(i.user_one);
        friend_ids.push(i.user_two);
    }

    let wish_q1 = user_id.eq(user_token);
    let wish_q2 = access_level.eq("public".to_string());
    let wish_q3 = user_id.eq_any(friend_ids);
    let wish_q4 = access_level.eq("friends".to_string());
    let wish_q5 = wish_q3.and(wish_q4);
    let wish_q6 = wish_q2.or(wish_q5);
    let wish_q7 = wish_q1.or(wish_q6);

    let results = cx.select_list(wish_q7).unwrap();

    // let results = select_list(connection, wish_q7).expect("Error retrieving wishes");

    // let results = self::schema::wishes::dsl::wishes
    //     .filter(user_id.eq(user_token)) 
    //     .or_filter(access_level.eq("public")) 
    //     .or_filter((user_id.eq_any(friend_ids)).and(access_level.eq("friends")))
    //     .load::<Wish>(connection)
    //     .expect("Error loading wishes");

    Template::render("wishes", context! {wishes: &results})
}

#[get("/friendships")]
pub fn get_friendships(user_session: UserSession) -> Template {
    //use self::models::Friendship;
    use self::schema::friendships::user_one;

    //let connection = &mut establish_connection_pg();

    let mut cx = user_session.into_context();

    let auth_user = cx.auth_user();
    let user_token = auth_user.user_id;

    let results_q1 = user_one.eq(user_token);
    let results_q2 = user_two.eq(user_token);
    let results_q3 = results_q1.or(results_q2);

    let results = cx.select_list(results_q3).unwrap();

    // let results = select_list(connection, results_q3).expect("Error retreiving results");


    let requests_q1 = user_two.eq(user_token);
    let requests_q2 = status.eq(0);
    let requests_q3 = requests_q1.and(requests_q2);

    let requests = cx.select_list(requests_q3).unwrap();

    // let requests = select_list(connection, requests_q3).expect("Error retrieving requets");
    

    // let results = self::schema::friendships::dsl::friendships
    //     .filter(user_one.eq(user_token))
    //     .or_filter(user_two.eq(user_token))
    //     .load::<Friendship>(connection)
    //     .expect("Error loading friendships");

    // let requests = self::schema::friendships::dsl::friendships
    //     .filter((user_two.eq(user_token)).and(status.eq("pending")))
    //     .load::<Friendship>(connection)
    //     .expect("Error loading friendships");

    Template::render("friendships", context! {friendships: results, requests: requests})
}


impl rdiesel::Expr<User, i32> for schema::users::user_id {}
#[post("/post_friendship", format="form", data="<friendship>")]
pub fn create_friendship_request(friendship: Form<FriendshipDto>, user_session: UserSession) -> Template {
    //use self::schema::friendships::dsl::friendships;

    //let connection = &mut establish_connection_pg();

    let mut cx = user_session.into_context();
    let auth_user = cx.auth_user();

    let user_token = auth_user.user_id;

    // checks to see if requested user exists
    let user_q1 = user_id.eq(friendship.user_two);

    let requested_user = cx.select_first(user_q1).unwrap().is_some();
    //let requested_user = select_list(connection, user_q1).expect("Error retrieving users");

    // let requested_user = self::schema::users::dsl::users
    //     .filter(user_id.eq(friendship.user_two.to_string()))
    //     .load::<User>(connection)
    //     .expect("Error retrieving user");

    if !requested_user {
        Template::render("friendships", context! {})
    } else  {
        let new_friendship = FriendshipDto {
            user_one: user_token,
            user_two: friendship.user_two,
            status: friendship.status
        };

        let _ = cx.insert(new_friendship);
        // diesel::insert_into(friendships)
        //     .values(new_friendship)
        //     .execute(connection)
        //     .expect("Friendship failed");

        //let results_q1 = rdiesel::Expr::eq(user_one, user_token.to_string());
        let results_q1 = user_one.eq(user_token);

        let results = cx.select_list(results_q1).unwrap();
        // let results = select_list(connection, results_q1).expect("Error retrieving results");

        // let results = self::schema::friendships::dsl::friendships
        //     .filter(user_one.eq(user_token))
        //     .load::<Friendship>(connection)
        //     .expect("Error loading friendships");
    
        Template::render("friendships", context! {friendships: &results})
    }
}

impl rdiesel::Field<Friendship, i32, User> for schema::friendships::user_one {}
impl rdiesel::Field<Friendship, i32, User> for schema::friendships::user_two {}
impl rdiesel::Field<Friendship, i32, User> for schema::friendships::status {}

#[post("/change_friendship", format="form", data="<friendship>")]
pub fn change_friendship_status(friendship: Form<FriendshipDto>, user_session: UserSession) {
    use self::schema::friendships::dsl::*;

    let mut cx = user_session.into_context();

    let auth_user = cx.auth_user();
    let connection = &mut establish_connection_pg();

    let q1 = rdiesel::Expr::eq(user_one, friendship.user_one);
    let q2 = rdiesel::Expr::eq(user_two, friendship.user_two);
    let q3 = rdiesel::Expr::and(q1, q2);


    let _ = cx.update_where(q3, status.assign(friendship.status));

    //let _ = update_where(connection, q3, status.assign(friendship.status));

    // // diesel::update(friendships)
    // //     .filter((user_one.eq(friendship.user_one.to_string())).and(user_two.eq(friendship.user_two.to_string()))) //matches to friendship in table
    // //     .set(status.eq(&friendship.status))
    // //     .execute(connection)
    // //     .expect("Error updating status");

    // // get_friendships(user_session)
}