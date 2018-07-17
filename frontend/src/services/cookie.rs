//! A cookie handling service to read and write cookies

use failure::Error;
use stdweb::unstable::TryInto;

#[derive(Debug, Fail)]
pub enum CookieError {
    #[fail(display = "no cookie found for the given name")]
    NotFound,
}

pub struct CookieService;

impl CookieService {
    /// Create a new cookie service instance
    pub fn new() -> Self {
        CookieService
    }

    /// Set a cookie for a given name and value for a default validity of one year
    pub fn set(&self, name: &str, value: &str) {
        self.set_expiring(name, value, 365)
    }

    /// Retrieve a cookie for a given name
    pub fn get(&self, name: &str) -> Result<String, Error> {
        let cookie_strings = js! { return document.cookie.split(';') };
        let cookies: Vec<String> = cookie_strings.try_into()?;
        cookies
            .iter()
            .filter_map(|x| {
                let name_value: Vec<_> = x.split("=").collect();
                match name_value.get(0) {
                    None => None,
                    Some(c) => {
                        if *c == name {
                            name_value.get(1).map(|x| (*x).to_owned())
                        } else {
                            None
                        }
                    }
                }
            })
            .collect::<Vec<String>>()
            .pop()
            .ok_or(CookieError::NotFound.into())
    }

    /// Remove a cookie for a given name
    pub fn remove(&self, name: &str) {
        self.set_expiring(name, "", -1);
    }

    /// Set a cookie for a given name, value and validity days
    fn set_expiring(&self, name: &str, value: &str, days: i32) {
        js! {
            document.cookie = @{name} + "=" + (@{value} || "")  +
                ";max-age=" + (@{days} * 24 * 60 * 60) + ";path=/";
        }
    }
}
