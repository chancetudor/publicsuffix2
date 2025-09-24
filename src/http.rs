use crate::errors::{Error, Result};

pub fn get(url: &str) -> Result<String> {
    let agent = ureq::agent();
    agent
        .get(url)
        .call()
        .map_err(|e| Error::Fetch(Box::new(e)))?
        .into_string()
        .map_err(Error::Io)
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::Server;

    #[test]
    fn test_get_success() {
        let mut server = Server::new();
        let mock = server
            .mock("GET", "/dat")
            .with_status(200)
            .with_header("content-type", "text/plain")
            .with_body("test data")
            .create();

        let url = server.url();
        let result = get(&format!("{}/dat", url));

        mock.assert();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "test data");
    }

    #[test]
    fn test_get_failure() {
        let mut server = Server::new();
        let mock = server
            .mock("GET", "/dat")
            .with_status(500)
            .with_body("server error")
            .create();

        let url = server.url();
        let result = get(&format!("{}/dat", url));

        mock.assert();
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::Fetch(_) => {} // Expected error
            e => panic!("Expected Error::Fetch, but got {:?}", e),
        }
    }
}
