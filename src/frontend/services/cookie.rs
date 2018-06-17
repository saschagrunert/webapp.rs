//! A cookie handling service to read and write cookies

use failure::Error;
use stdweb::{unstable::TryInto, web::Date};

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

    /// Set a cookie for a given name, value and validity days
    pub fn set_cookie(&self, name: &str, value: &str, days: i32) {
        let date = Date::new();
        date.set_time(date.get_time() + (days * 24 * 60 * 60 * 1000) as f64);
        let expires = "; expires=".to_owned() + &date.to_utc_string();
        js! {
            document.cookie = @{name} + "=" + (@{value} || "")  + @{expires} + "; path=/";
        }
    }

    /// Retrieve a cookie for a given name
    pub fn get_cookie(&self, name: &str) -> Result<String, Error> {
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
    pub fn remove_cookie(&self, name: &str) {
        self.set_cookie(name, "", -1);
    }
}
