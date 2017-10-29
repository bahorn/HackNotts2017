#![feature(plugin)]
#![plugin(rocket_codegen)]
extern crate main;
extern crate rocket;
extern crate rocket_contrib;
#[macro_use] extern crate serde_derive;
extern crate serde;
extern crate serde_json;
extern crate diesel;
extern crate r2d2;
extern crate r2d2_diesel;
extern crate dotenv;
extern crate ring_pwhash;
extern crate uuid;
use diesel::pg::PgConnection;
use r2d2_diesel::ConnectionManager;
use std::env;
use std::path::{Path, PathBuf};
use rocket_contrib::Json;
use std::ops::Deref;
use rocket::http::{Status, Cookies, Cookie};
use rocket::request::{self, FromRequest};
use rocket::{Request, State, Outcome};
use rocket::response::NamedFile;
use self::main::*;
use self::main::models::*;
use self::diesel::prelude::*;
use ring_pwhash::scrypt;
use uuid::Uuid;
use std::process::Command;

// this bit was stolen from the docks. (And patched to use Postgres. )
type Pool = r2d2::Pool<ConnectionManager<PgConnection>>;
pub struct DbConn(pub r2d2::PooledConnection<ConnectionManager<PgConnection>>);
impl<'a, 'r> FromRequest<'a, 'r> for DbConn {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<DbConn, ()> {
        let pool = request.guard::<State<Pool>>()?;
        match pool.get() {
            Ok(conn) => Outcome::Success(DbConn(conn)),
            Err(_) => Outcome::Failure((Status::ServiceUnavailable, ()))
        }
    }
}
// For the convenience of using an &DbConn as an &SqliteConnection.
impl Deref for DbConn {
    type Target = PgConnection;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Deserialize)]
struct LoginForm {
    username: String,
    password: String
}

#[derive(Serialize)]
struct AuthReturn {
    status: bool,
    msg: String,
}

#[derive(Serialize, Deserialize)]
struct AuthCookie {
    username: String
}

#[derive(Deserialize)]
struct LogoutForm {
    valid: bool
}

#[derive(Deserialize)]
struct BlobForm {
    blob: String
}

#[derive(Deserialize)]
struct DelBlobForm {
    uuid: String
}

#[derive(Deserialize)]
struct PushBlobForm {
    uuid: String, // uuid of our data
    name: String // used to call when pushed
}

#[derive(Serialize, Deserialize)]
struct OutBlob {
    uuid: String,
    owner: String,
    value: String
}

#[post("/auth", data="<data>")]
fn auth(data: Json<LoginForm>, conn: DbConn, mut cookies: Cookies) -> Json<AuthReturn> {
    let mut auth = AuthReturn {status: false, msg: "Username/Password Invalid".to_string()};
    use main::schema::users::dsl::*;
    match users.filter(username.eq(&data.username)).get_result::<User>(&*conn) {
        Ok(user_data) => {
            match scrypt::scrypt_check(&data.password, &user_data.phash) {
                Ok(value) => {
                    if value == true {
                        auth.status = true;
                        auth.msg = "Successfully Authenticated".to_string();
                        cookies.add_private(
                            Cookie::new("user", data.username.to_string()));
                    }
                }
                Err(_) => {}
            }
        }
        Err(_) => {}
    }
    Json(auth)
}

#[post("/register", data="<data>")]
fn register(data: Json<LoginForm>, conn: DbConn, mut cookies: Cookies) -> Json<AuthReturn> {
    let mut auth = AuthReturn {status: false, msg: "Unable to register.".to_string()};
    let scrypt_settings = scrypt::ScryptParams::new(12, 8, 1); // seems ok, change if actually becomes real world.
    use schema::users;
    use self::models::NewUser;
    match cookies.get_private("user") {
        Some(_) => {
            auth.msg = "You are already logged in.".to_string();
            return Json(auth);
        }
        None => {}
    }
    let new_user = NewUser {
        username: &data.username,
        phash: &scrypt::scrypt_simple(&data.password, &scrypt_settings).expect("Unable to hash?")
    };
    match diesel::insert(&new_user).into(users::table).execute(&*conn) {
        Ok(_) => {
            auth.status = true;
            auth.msg = "Registered. You are now authenticated.".to_string();
        }
        Err(_) => {}
    }
    cookies.add_private(Cookie::new("user", data.username.to_string()));
    Json(auth)
}

#[post("/logout", data="<logout>")]
fn logout(logout: Json<LogoutForm>, mut cookies: Cookies) -> Json<AuthReturn> {
    let returned = AuthReturn {status: true, msg: "Logged out".to_string()};
    if logout.valid == true {
        cookies.remove_private(Cookie::named("user"));
    } // should probably edit the returned.
    Json(returned)
}

// A get method for some reason.
#[get("/list_blobs")]
fn listblobs(conn: DbConn, mut cookies: Cookies) -> Json<Vec<OutBlob>> {
    use main::schema::blobs::dsl::*;
    let username: String;
    let mut ret: Vec<OutBlob> = Vec::new();
    match cookies.get_private("user") {
        Some(user) => {
            username = user.value().to_string();
        }
        None => {return Json(ret)} // nothing. just escape.
    }
    let results = blobs.filter(owner.eq(&username))
        .load::<Blob>(&*conn);
    match results {
        Ok(result) => {
            for i in result {
                ret.push(OutBlob {uuid: i.uuid, owner: i.owner , value: i.value});
            }
        },
        Err(_) => {}
    }
    Json(ret)
}

#[post("/add_blob", data="<blob>")]
fn addblob(blob: Json<BlobForm>, conn: DbConn, mut cookies: Cookies) -> Json<AuthReturn> {
    let mut returned = AuthReturn {status: false, msg: "Not logged in".to_string()};
    let username: String;;
    use schema::blobs;
    use self::models::NewBlob;
    match cookies.get_private("user") {
        Some(user) => {
            username = user.value().to_string();
        }
        None => {return Json(returned)} // nothing. just escape.
    }
    let new_uuid = Uuid::new_v4();
    let new_blob = NewBlob {
        uuid: &new_uuid.to_string(),
        owner: &username.to_string(),
        value: &blob.blob
    };
    match diesel::insert(&new_blob).into(blobs::table).execute(&*conn) {
        Ok(_) => {
            returned.status = true;
            returned.msg = "Successfully added blob.".to_string();
        }
        Err(_) => {
            returned.msg = "Unable to add blob.".to_string();
        }
    }
    Json(returned)
}

#[post("/del_blob", data="<blob>")]
fn delblob(blob: Json<DelBlobForm>, conn: DbConn, mut cookies: Cookies) -> Json<AuthReturn> {
    let mut returned = AuthReturn {status: false, msg: "Not logged in".to_string()};
    let username: String;
    use main::schema::blobs::dsl::*;
    match cookies.get_private("user") {
        Some(user) => {
            username = user.value().to_string();
        }
        None => {return Json(returned)}
    }
    match diesel::delete(blobs.filter(uuid.eq(&blob.uuid)).filter(owner.eq(username))).execute(&*conn) {
        Ok(_) => {
           returned.status = true;
           returned.msg = "Deleted blob".to_string();
        }
        Err(_) => {
            returned.msg = "Unable to run query".to_string();
        }
    }
    Json(returned)
}

// this pushes the blob to spaces
#[post("/push_blob", data="<query>")]
fn pushblob(query: Json<PushBlobForm>, conn: DbConn, mut cookies: Cookies) -> Json<AuthReturn> {
    let mut returned = AuthReturn {status: false, msg: "Not logged in".to_string()};
    let username: String;
    use main::schema::blobs::dsl::*;
    match cookies.get_private("user") {
        Some(user) => {
            username = user.value().to_string();
        }
        None => {return Json(returned)}
    }
    let result = blobs.filter(uuid.eq(&query.uuid))
        .filter(owner.eq(username)).get_result::<Blob>(&*conn);
    match result {
        Ok(blob) => {
            // push to production. if anything comes of this, this
            // is the first part to change....
            let cmd = Command::new("python")
                .arg("../pushcontrol/src/pushcontrol.py")
                .arg("bahorn")
                .arg(&query.name)
                .arg(blob.value)
                .output();
            match cmd {
                Ok(_) => {}
                Err(_) => {
                    returned.msg = "push failed to run".to_string();
                    return Json(returned);
                }
            }
            returned.status = true;
            returned.msg = "Ran job".to_string();
        }
        Err(_) => {
            returned.msg = "Job not found".to_string();
        }
    }
    Json(returned)
}

#[get("/info")]
fn info(mut cookies: Cookies) -> Json<AuthReturn> {
    let mut returned = AuthReturn {status: false, msg: "".to_string()};
    let username: String;
    match cookies.get_private("user") {
        Some(user) => {
            username = user.value().to_string();
            returned.status = true;
            returned.msg = username;
        }
        None => {}
    }
    Json(returned)
}

// stolen from example
#[get("/static/<file..>")]
fn files(file: PathBuf) -> Option<NamedFile> {
    NamedFile::open(Path::new("static/").join(file)).ok()
}

#[get("/")]
fn hello() -> Option<NamedFile> {
    NamedFile::open("static/index.html").ok()
}

fn init_pool() -> Pool {
    let config = r2d2::Config::default();
    let database = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = ConnectionManager::<PgConnection>::new(database);
    r2d2::Pool::new(config, manager).expect("db pool")
}

fn main() {
    rocket::ignite()
        .manage(init_pool())
        .mount("/", routes![hello, auth, register, listblobs, pushblob, 
               addblob, delblob, info, files,logout])
        .launch();
}
