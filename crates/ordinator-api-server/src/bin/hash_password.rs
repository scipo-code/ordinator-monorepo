use argon2::Argon2;
use argon2::password_hash::PasswordHasher;
use argon2::password_hash::SaltString;
use argon2::password_hash::rand_core::OsRng;

fn main()
{
    let password = std::env::args()
        .nth(1)
        .expect("Usage: hash_password <password>");

    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hash = argon2.hash_password(password.as_bytes(), &salt).unwrap();

    println!("{}", hash.to_string());
}
