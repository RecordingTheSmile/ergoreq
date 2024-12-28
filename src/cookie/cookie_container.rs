use std::vec;

use chrono::Utc;
use cookie::{time::OffsetDateTime, Cookie};
use dashmap::DashMap;

/// Automatically store and set cookie headers for request.
pub trait CookieContainer: Send + Sync {
    /// Store cookies from response
    fn store_from_response<'a>(&self, cookies: Vec<Cookie<'a>>, url: &reqwest::Url);

    /// Serialize all matched cookies to `Cookie` header value
    fn to_header_value(&self, url: &reqwest::Url) -> Vec<String>;
}

/// key: cookie name
///
/// value: [`Cookie`]
pub type CookieMap = DashMap<String, Cookie<'static>>;

/// key: path
///
/// value: [`CookieMap`]
pub type PathMap = DashMap<String, CookieMap>;

/// key: domain
///
/// value: [`PathMap`]
pub type DomainMap = DashMap<String, PathMap>;

/// Default `CookieContainer` implementation
pub struct ErgoCookieContainer {
    store: DomainMap,
    match_domain_only: bool,
    no_expire_check: bool,
    ignore_secure: bool,
}

impl ErgoCookieContainer {
    pub fn new(match_domain_only: bool, no_expire_check: bool, ignore_secure: bool) -> Self {
        ErgoCookieContainer {
            store: DomainMap::new(),
            match_domain_only,
            no_expire_check,
            ignore_secure,
        }
    }

    /// Create a new `CookieContainer` with default secure settings.
    pub fn new_secure() -> Self {
        Self::new(false, false, false)
    }

    /// judge if two domain match cookie domain policy
    ///
    /// ## Match condition
    /// * request_domain(www.google.com) == cookie_domain(www.google.com)
    /// * request_domain(www.google.com) == cookie_domain(.google.com)
    /// * request_domain(img.static.google.com) == cookie_domain(static.google.com)
    ///
    /// ## Unmatch condition
    /// * request_domain(google.com) != cookie_domain(www.google.com)
    /// * request_domain(abc.google.com) !=  cookie_domain(c.google.com)
    fn is_domain_match(cookie_domain: &str, request_domain: &str) -> bool {
        // Convert domain strings to lowercase for case-insensitive comparison
        let cookie_domain = cookie_domain.to_ascii_lowercase();
        let request_domain = request_domain.to_ascii_lowercase();

        // Check for an exact match between the cookie domain and request domain
        if cookie_domain == request_domain {
            return true;
        }

        if cookie_domain.starts_with('.') {
            // Remove the leading '.' from the cookie domain and compare
            // the end of the request domain with the modified cookie domain
            if request_domain.ends_with(&cookie_domain[1..]) {
                return true;
            }
        }
        // Check if the request domain ends with the modified cookie domain and is separated by a dot
        else if request_domain.ends_with(&format!(".{}", cookie_domain)) {
            return true;
        }

        false
    }

    /// Get a mutable reference to inner storage.
    ///
    /// You can edit cookies in inner storage directly.
    pub fn get_storage_mut(&mut self) -> &mut DomainMap {
        &mut self.store
    }

    fn remove_target_cookie(&self, cookie: Cookie, url: &reqwest::Url) {
        let domain = match cookie.domain().and_then(|v| Some(v.to_string())) {
            Some(domain) => domain,
            None => match url.host_str() {
                Some(domain) => domain.to_owned(),
                None => return,
            },
        };
        // get_all_matched_path_map
        let all_matched_path_map = self
            .store
            .iter()
            .filter(|v| Self::is_domain_match(v.key(), &domain));
        if self.match_domain_only {
            for path_map in all_matched_path_map {
                if let Some(cookie_map) = path_map.get("") {
                    cookie_map.remove(cookie.name());
                }
            }
        } else {
            let cookie_path = cookie.path().unwrap_or(url.path());
            for path_map in all_matched_path_map {
                if let Some(cookie_map) = path_map.get(cookie_path) {
                    cookie_map.remove(cookie.name());
                }
            }
        }
    }

    pub fn remove_all_expired_cookies(&self) {
        for path_map in self.store.iter() {
            for cookie_map in path_map.iter() {
                cookie_map.retain(|_, v| {
                    if let Some(exp) = v.expires_datetime() {
                        if Utc::now().timestamp() >= exp.unix_timestamp() {
                            false
                        } else {
                            true
                        }
                    } else {
                        true
                    }
                })
            }
        }
    }

    /// set cookies manually, inner call `store_from_response`
    ///
    /// ## Notice
    /// `origin_url` in parameter cannot be empty, for `store_from_response` will check it.
    pub fn set_cookie(&self, cookie: Vec<Cookie>, origin_url: &str) -> Result<(), url::ParseError> {
        self.store_from_response(cookie, &reqwest::Url::parse(origin_url)?);
        Ok(())
    }

    /// serialize all cookies stored, returns `cookies` and its `origin_url`
    ///
    /// Please notice that the `scheme` in `origin_url` will be inferred from `Secure` configuration in `cookie`,
    ///
    /// instead of request url.
    /// ## Returns
    /// `Vec<(Cookie,OriginUrl)>`
    pub fn serialize_cookies(&self) -> Vec<(Cookie<'static>, String)> {
        let mut result = vec![];

        for domain_map in self.store.iter() {
            for path_map in domain_map.iter() {
                for cookie_map in path_map.iter() {
                    let scheme = if cookie_map.value().secure().unwrap_or(false) {
                        "https"
                    } else {
                        "http"
                    };

                    result.push((
                        cookie_map.value().to_owned(),
                        format!("{scheme}://{}{}", domain_map.key(), path_map.key()),
                    ))
                }
            }
        }
        result
    }
}

impl CookieContainer for ErgoCookieContainer {
    fn store_from_response<'a>(&self, cookies: Vec<Cookie<'a>>, url: &reqwest::Url) {
        for mut cookie in cookies {
            // if cookie is http_only and request is not a http request
            if cookie.http_only().unwrap_or(false)
                && url.scheme() != "http"
                && url.scheme() != "https"
            {
                // ignore cookie
                continue;
            }

            if !self.no_expire_check {
                // check if expired
                if let Some(exp_at) = cookie.expires_datetime() {
                    let time_now = chrono::Utc::now().timestamp();
                    if time_now >= exp_at.unix_timestamp() {
                        self.remove_target_cookie(cookie, url);
                        continue;
                    }
                } else if let Some(max_age) = cookie.max_age() {
                    if max_age.is_zero() || max_age.is_negative() {
                        self.remove_target_cookie(cookie, url);
                        continue;
                    } else {
                        // convert cookie `max-age` to `expires`
                        cookie.set_expires(OffsetDateTime::now_utc() + max_age);

                        //unset `max-age`
                        cookie.set_max_age(None);
                    }
                }
            }

            let domain = match cookie.domain().and_then(|v| Some(v.to_string())) {
                Some(domain) => domain,
                None => match url.host_str() {
                    Some(domain) => domain.to_owned(),
                    None => continue,
                },
            };
            let domain = domain.trim();

            let store = |path_map: &PathMap| {
                if self.match_domain_only {
                    if let Some(any_map) = path_map.get("") {
                        any_map.insert(cookie.name().to_owned(), cookie.into_owned());
                    } else {
                        let any_map = CookieMap::new();
                        any_map.insert(cookie.name().to_owned(), cookie.into_owned());
                        path_map.insert("".to_owned(), any_map);
                    }
                } else {
                    let cookie_path = cookie.path().unwrap_or(url.path()).to_owned();
                    if let Some(cookie_map) = path_map.get(&cookie_path) {
                        cookie_map.insert(cookie.name().to_owned(), cookie.into_owned());
                    } else {
                        let cookie_map = CookieMap::new();
                        cookie_map.insert(cookie.name().to_owned(), cookie.into_owned());
                        path_map.insert(cookie_path.to_owned(), cookie_map);
                    }
                }
            };

            if let Some(path_map) = self.store.get(domain) {
                store(path_map.value());
            } else {
                let path_map = PathMap::new();
                store(&path_map);
                self.store.insert(domain.to_owned(), path_map);
            }
        }
    }

    fn to_header_value(&self, url: &reqwest::Url) -> Vec<String> {
        if url.host_str().is_none() {
            return vec![];
        }
        if !self.no_expire_check {
            // remove all expired cookies
            self.remove_all_expired_cookies();
        }
        let all_matched_path_map = self
            .store
            .iter()
            .filter(|v| Self::is_domain_match(v.key(), url.host_str().unwrap()));

        let mut result = vec![];

        for path_map in all_matched_path_map {
            if self.match_domain_only {
                if let Some(cookie_map) = path_map.get("") {
                    for cookie in cookie_map.value() {
                        if !self.ignore_secure {
                            if cookie.secure().unwrap_or(false) && url.scheme() != "https" {
                                continue;
                            }
                        }
                        result.push(cookie.value().encoded().stripped().to_string());
                    }
                }
            } else {
                if let Some(cookie_map) = path_map.get(url.path()) {
                    for cookie in cookie_map.value() {
                        if !self.ignore_secure {
                            if cookie.secure().unwrap_or(false) && url.scheme() != "https" {
                                continue;
                            }
                        }
                        result.push(cookie.value().encoded().stripped().to_string());
                    }
                }
            }
        }

        result
    }
}

impl Default for ErgoCookieContainer {
    fn default() -> Self {
        ErgoCookieContainer::new(false, false, false)
    }
}

#[cfg(test)]
mod test_default_cookie_container {
    use crate::cookie::{cookie_container::ErgoCookieContainer, cookie_parser::ErgoCookieParser};

    use super::CookieContainer;

    const SET_COOKIE_HEADERS: [&str; 13] = [
        "mycookie=example; path=/; domain=",
        "subdomain_cookie=subdomain; path=/; domain=.example.com;",
        "domain_cookie=domain; path=/; domain=example.com",
        "cross_domain_cookie=cross; path=/; domain=example.com;",
        "session=abc123; path=/",
        "user=johndoe; path=/profile",
        "lang=en-US; expires=Thu, 28 Oct 2099 14:30:00 GMT",
        "theme=dark; domain=example.com",
        "remember=true; path=/; secure",
        "deleted=; expires=Thu, 01 Jan 1970 00:00:00 GMT", // expired
        "httpOnly=true; path=/; HttpOnly",
        "maxAgeCookie=test; path=/; max-age=3600",
        "sameSiteCookie=test; path=/; SameSite=Strict",
    ];

    #[test]
    fn test_domain_matching() {
        // Match condition
        assert!(ErgoCookieContainer::is_domain_match(
            "www.google.com",
            "www.google.com"
        ));
        assert!(ErgoCookieContainer::is_domain_match(
            ".google.com",
            "www.google.com"
        ));
        assert!(ErgoCookieContainer::is_domain_match(
            "google.com",
            "www.google.com"
        ));
        assert!(ErgoCookieContainer::is_domain_match(
            "static.google.com",
            "img.static.google.com"
        ));
    }

    #[test]
    fn test_domain_non_matching() {
        // Unmatch condition
        assert!(!ErgoCookieContainer::is_domain_match(
            "www.google.com",
            "google.com"
        ));
        assert!(!ErgoCookieContainer::is_domain_match(
            "c.google.com",
            "abc.google.com"
        ));
    }

    #[test]
    fn test_cookie_container_store() {
        let parsed_cookies =
            ErgoCookieParser::parse_set_cookie_header(SET_COOKIE_HEADERS.into_iter());

        let container = ErgoCookieContainer::new(false, false, false);
        container.store_from_response(
            parsed_cookies,
            &reqwest::Url::parse("http://crates.io").unwrap(),
        );
        let cookie_count = container
            .store
            .iter()
            .map(|v| v.value().iter().map(|v| v.value().len()).sum::<usize>())
            .sum::<usize>();
        println!("Cookie parsed and valid: {}", cookie_count);
        println!("Stored cookies: {:#?}", container.store);
        assert_eq!(cookie_count, 12);
    }

    #[test]
    fn test_cookie_container_restore() {
        let parsed_cookies =
            ErgoCookieParser::parse_set_cookie_header(SET_COOKIE_HEADERS.into_iter());

        let container = ErgoCookieContainer::new(false, false, false);
        container.store_from_response(
            parsed_cookies,
            &reqwest::Url::parse("https://crates.io").unwrap(),
        );
        let result = container.to_header_value(&reqwest::Url::parse("http://crates.io").unwrap());
        println!("Result: {:#?}", result);
        assert_eq!(result.len(), 6);
        let result = container.to_header_value(&reqwest::Url::parse("https://crates.io").unwrap());
        println!("Result secure: {:#?}", result);
        assert_eq!(result.len(), 7);
        let result =
            container.to_header_value(&reqwest::Url::parse("https://crates.io/profile").unwrap());
        println!("Result path: {:#?}", result);
        assert_eq!(result.len(), 1);
        let result =
            container.to_header_value(&reqwest::Url::parse("https://abc.example.com").unwrap());
        println!("Result subdomain: {:#?}", result);
        assert_eq!(result.len(), 4);
        let result = container.to_header_value(&reqwest::Url::parse("https://xample.com").unwrap());
        println!("Result nodomain: {:#?}", result);
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_serialize() {
        let parsed_cookies =
            ErgoCookieParser::parse_set_cookie_header(SET_COOKIE_HEADERS.into_iter());

        let container = ErgoCookieContainer::new(false, false, false);
        container.store_from_response(
            parsed_cookies,
            &reqwest::Url::parse("http://crates.io").unwrap(),
        );
        let result = container.serialize_cookies();
        println!("Serialize result: {:#?}", result);
        assert_eq!(result.len(), 12);
    }
}
