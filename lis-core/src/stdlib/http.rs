//! HTTP Client Operations for LIS
//!
//! Functions for making HTTP requests and handling responses.
//! This module provides a simple, synchronous HTTP client for LIS programs.

use crate::error::{Error, Result};

/// HTTP response containing status code, headers, and body
#[derive(Debug, Clone)]
pub struct HttpResponse {
    /// HTTP status code (e.g., 200, 404, 500)
    pub status: u16,
    /// Response body as a string
    pub body: String,
    /// Content-Type header value if present
    pub content_type: Option<String>,
}

/// @stdlib_http fn http_get(url: String) -> String
///
/// Makes an HTTP GET request and returns the response body.
/// Returns an error if the request fails or returns a non-2xx status.
///
/// # Example
/// ```lis
/// let response = http_get("https://api.example.com/data");
/// println(response);
/// ```
pub fn http_get(url: &str) -> Result<String> {
    let response = ureq::get(url)
        .call()
        .map_err(|e| Error::IoError {
            message: format!("HTTP GET failed for '{}': {}", url, e),
        })?;

    let body = response.into_string().map_err(|e| Error::IoError {
        message: format!("Failed to read response body: {}", e),
    })?;

    Ok(body)
}

/// @stdlib_http fn http_get_with_status(url: String) -> (Int, String)
///
/// Makes an HTTP GET request and returns both status code and body.
/// Unlike http_get, this does not error on non-2xx status codes.
///
/// # Example
/// ```lis
/// let (status, body) = http_get_with_status("https://api.example.com/data");
/// if status == 200 {
///     println(body);
/// }
/// ```
pub fn http_get_with_status(url: &str) -> Result<(i64, String)> {
    match ureq::get(url).call() {
        Ok(response) => {
            let status = response.status() as i64;
            let body = response.into_string().map_err(|e| Error::IoError {
                message: format!("Failed to read response body: {}", e),
            })?;
            Ok((status, body))
        }
        Err(ureq::Error::Status(code, response)) => {
            let body = response.into_string().unwrap_or_default();
            Ok((code as i64, body))
        }
        Err(e) => Err(Error::IoError {
            message: format!("HTTP GET failed for '{}': {}", url, e),
        }),
    }
}

/// @stdlib_http fn http_post(url: String, body: String) -> String
///
/// Makes an HTTP POST request with a text body and returns the response.
/// Content-Type defaults to text/plain.
///
/// # Example
/// ```lis
/// let response = http_post("https://api.example.com/submit", "Hello, World!");
/// println(response);
/// ```
pub fn http_post(url: &str, body: &str) -> Result<String> {
    let response = ureq::post(url)
        .set("Content-Type", "text/plain")
        .send_string(body)
        .map_err(|e| Error::IoError {
            message: format!("HTTP POST failed for '{}': {}", url, e),
        })?;

    let response_body = response.into_string().map_err(|e| Error::IoError {
        message: format!("Failed to read response body: {}", e),
    })?;

    Ok(response_body)
}

/// @stdlib_http fn http_post_json(url: String, json: String) -> String
///
/// Makes an HTTP POST request with a JSON body.
/// Automatically sets Content-Type to application/json.
///
/// # Example
/// ```lis
/// let json = "{\"name\": \"Alice\", \"value\": 42}";
/// let response = http_post_json("https://api.example.com/data", json);
/// println(response);
/// ```
pub fn http_post_json(url: &str, json: &str) -> Result<String> {
    let response = ureq::post(url)
        .set("Content-Type", "application/json")
        .send_string(json)
        .map_err(|e| Error::IoError {
            message: format!("HTTP POST JSON failed for '{}': {}", url, e),
        })?;

    let response_body = response.into_string().map_err(|e| Error::IoError {
        message: format!("Failed to read response body: {}", e),
    })?;

    Ok(response_body)
}

/// @stdlib_http fn http_put(url: String, body: String) -> String
///
/// Makes an HTTP PUT request with a text body.
///
/// # Example
/// ```lis
/// let response = http_put("https://api.example.com/resource/1", "Updated content");
/// println(response);
/// ```
pub fn http_put(url: &str, body: &str) -> Result<String> {
    let response = ureq::put(url)
        .set("Content-Type", "text/plain")
        .send_string(body)
        .map_err(|e| Error::IoError {
            message: format!("HTTP PUT failed for '{}': {}", url, e),
        })?;

    let response_body = response.into_string().map_err(|e| Error::IoError {
        message: format!("Failed to read response body: {}", e),
    })?;

    Ok(response_body)
}

/// @stdlib_http fn http_put_json(url: String, json: String) -> String
///
/// Makes an HTTP PUT request with a JSON body.
///
/// # Example
/// ```lis
/// let json = "{\"name\": \"Updated\"}";
/// let response = http_put_json("https://api.example.com/resource/1", json);
/// println(response);
/// ```
pub fn http_put_json(url: &str, json: &str) -> Result<String> {
    let response = ureq::put(url)
        .set("Content-Type", "application/json")
        .send_string(json)
        .map_err(|e| Error::IoError {
            message: format!("HTTP PUT JSON failed for '{}': {}", url, e),
        })?;

    let response_body = response.into_string().map_err(|e| Error::IoError {
        message: format!("Failed to read response body: {}", e),
    })?;

    Ok(response_body)
}

/// @stdlib_http fn http_delete(url: String) -> String
///
/// Makes an HTTP DELETE request.
///
/// # Example
/// ```lis
/// let response = http_delete("https://api.example.com/resource/1");
/// println(response);
/// ```
pub fn http_delete(url: &str) -> Result<String> {
    let response = ureq::delete(url)
        .call()
        .map_err(|e| Error::IoError {
            message: format!("HTTP DELETE failed for '{}': {}", url, e),
        })?;

    let response_body = response.into_string().map_err(|e| Error::IoError {
        message: format!("Failed to read response body: {}", e),
    })?;

    Ok(response_body)
}

/// @stdlib_http fn http_patch(url: String, body: String) -> String
///
/// Makes an HTTP PATCH request with a text body.
///
/// # Example
/// ```lis
/// let response = http_patch("https://api.example.com/resource/1", "partial update");
/// println(response);
/// ```
pub fn http_patch(url: &str, body: &str) -> Result<String> {
    let response = ureq::patch(url)
        .set("Content-Type", "text/plain")
        .send_string(body)
        .map_err(|e| Error::IoError {
            message: format!("HTTP PATCH failed for '{}': {}", url, e),
        })?;

    let response_body = response.into_string().map_err(|e| Error::IoError {
        message: format!("Failed to read response body: {}", e),
    })?;

    Ok(response_body)
}

/// @stdlib_http fn http_patch_json(url: String, json: String) -> String
///
/// Makes an HTTP PATCH request with a JSON body.
///
/// # Example
/// ```lis
/// let json = "{\"field\": \"new_value\"}";
/// let response = http_patch_json("https://api.example.com/resource/1", json);
/// println(response);
/// ```
pub fn http_patch_json(url: &str, json: &str) -> Result<String> {
    let response = ureq::patch(url)
        .set("Content-Type", "application/json")
        .send_string(json)
        .map_err(|e| Error::IoError {
            message: format!("HTTP PATCH JSON failed for '{}': {}", url, e),
        })?;

    let response_body = response.into_string().map_err(|e| Error::IoError {
        message: format!("Failed to read response body: {}", e),
    })?;

    Ok(response_body)
}

/// @stdlib_http fn http_head(url: String) -> Int
///
/// Makes an HTTP HEAD request and returns the status code.
/// Useful for checking if a resource exists without downloading it.
///
/// # Example
/// ```lis
/// let status = http_head("https://example.com/file.txt");
/// if status == 200 {
///     println("Resource exists");
/// }
/// ```
pub fn http_head(url: &str) -> Result<i64> {
    match ureq::head(url).call() {
        Ok(response) => Ok(response.status() as i64),
        Err(ureq::Error::Status(code, _)) => Ok(code as i64),
        Err(e) => Err(Error::IoError {
            message: format!("HTTP HEAD failed for '{}': {}", url, e),
        }),
    }
}

/// @stdlib_http fn http_get_header(url: String, header: String) -> String
///
/// Makes an HTTP GET request with a custom header.
///
/// # Example
/// ```lis
/// let response = http_get_header(
///     "https://api.example.com/data",
///     "Authorization: Bearer token123"
/// );
/// println(response);
/// ```
pub fn http_get_with_header(url: &str, header: &str) -> Result<String> {
    // Parse header in "Name: Value" format
    let parts: Vec<&str> = header.splitn(2, ':').collect();
    if parts.len() != 2 {
        return Err(Error::SemanticError {
            message: format!("Invalid header format. Expected 'Name: Value', got: {}", header),
        });
    }

    let name = parts[0].trim();
    let value = parts[1].trim();

    let response = ureq::get(url)
        .set(name, value)
        .call()
        .map_err(|e| Error::IoError {
            message: format!("HTTP GET failed for '{}': {}", url, e),
        })?;

    let body = response.into_string().map_err(|e| Error::IoError {
        message: format!("Failed to read response body: {}", e),
    })?;

    Ok(body)
}

/// @stdlib_http fn http_get_auth(url: String, token: String) -> String
///
/// Makes an HTTP GET request with Bearer token authentication.
///
/// # Example
/// ```lis
/// let response = http_get_auth("https://api.example.com/private", "my_token");
/// println(response);
/// ```
pub fn http_get_auth(url: &str, token: &str) -> Result<String> {
    let auth_value = format!("Bearer {}", token);

    let response = ureq::get(url)
        .set("Authorization", &auth_value)
        .call()
        .map_err(|e| Error::IoError {
            message: format!("HTTP GET failed for '{}': {}", url, e),
        })?;

    let body = response.into_string().map_err(|e| Error::IoError {
        message: format!("Failed to read response body: {}", e),
    })?;

    Ok(body)
}

/// @stdlib_http fn http_post_auth(url: String, body: String, token: String) -> String
///
/// Makes an HTTP POST request with Bearer token authentication.
///
/// # Example
/// ```lis
/// let response = http_post_auth(
///     "https://api.example.com/submit",
///     "{\"data\": 123}",
///     "my_token"
/// );
/// println(response);
/// ```
pub fn http_post_auth(url: &str, body: &str, token: &str) -> Result<String> {
    let auth_value = format!("Bearer {}", token);

    let response = ureq::post(url)
        .set("Authorization", &auth_value)
        .set("Content-Type", "application/json")
        .send_string(body)
        .map_err(|e| Error::IoError {
            message: format!("HTTP POST failed for '{}': {}", url, e),
        })?;

    let response_body = response.into_string().map_err(|e| Error::IoError {
        message: format!("Failed to read response body: {}", e),
    })?;

    Ok(response_body)
}

/// @stdlib_http fn url_encode(s: String) -> String
///
/// URL-encodes a string for safe use in query parameters.
///
/// # Example
/// ```lis
/// let encoded = url_encode("hello world");
/// // Returns "hello%20world"
/// ```
pub fn url_encode(s: &str) -> Result<String> {
    Ok(urlencoding::encode(s).into_owned())
}

/// @stdlib_http fn url_decode(s: String) -> String
///
/// Decodes a URL-encoded string.
///
/// # Example
/// ```lis
/// let decoded = url_decode("hello%20world");
/// // Returns "hello world"
/// ```
pub fn url_decode(s: &str) -> Result<String> {
    urlencoding::decode(s)
        .map(|s| s.into_owned())
        .map_err(|e| Error::SemanticError {
            message: format!("Failed to URL decode '{}': {}", s, e),
        })
}

/// @stdlib_http fn http_status_ok(status: Int) -> Bool
///
/// Returns true if the HTTP status code indicates success (2xx).
///
/// # Example
/// ```lis
/// let (status, body) = http_get_with_status(url);
/// if http_status_ok(status) {
///     println("Success!");
/// }
/// ```
pub fn http_status_ok(status: i64) -> Result<bool> {
    Ok((200..300).contains(&(status as u16)))
}

/// @stdlib_http fn http_status_text(status: Int) -> String
///
/// Returns a human-readable description of an HTTP status code.
///
/// # Example
/// ```lis
/// let text = http_status_text(404);
/// // Returns "Not Found"
/// ```
pub fn http_status_text(status: i64) -> Result<String> {
    let text = match status as u16 {
        100 => "Continue",
        101 => "Switching Protocols",
        200 => "OK",
        201 => "Created",
        202 => "Accepted",
        204 => "No Content",
        301 => "Moved Permanently",
        302 => "Found",
        304 => "Not Modified",
        307 => "Temporary Redirect",
        308 => "Permanent Redirect",
        400 => "Bad Request",
        401 => "Unauthorized",
        403 => "Forbidden",
        404 => "Not Found",
        405 => "Method Not Allowed",
        408 => "Request Timeout",
        409 => "Conflict",
        410 => "Gone",
        413 => "Payload Too Large",
        415 => "Unsupported Media Type",
        422 => "Unprocessable Entity",
        429 => "Too Many Requests",
        500 => "Internal Server Error",
        501 => "Not Implemented",
        502 => "Bad Gateway",
        503 => "Service Unavailable",
        504 => "Gateway Timeout",
        _ => "Unknown Status",
    };
    Ok(text.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_encode() {
        let result = url_encode("hello world").unwrap();
        assert_eq!(result, "hello%20world");
    }

    #[test]
    fn test_url_encode_special_chars() {
        let result = url_encode("a=1&b=2").unwrap();
        assert_eq!(result, "a%3D1%26b%3D2");
    }

    #[test]
    fn test_url_decode() {
        let result = url_decode("hello%20world").unwrap();
        assert_eq!(result, "hello world");
    }

    #[test]
    fn test_url_decode_roundtrip() {
        let original = "hello world & stuff=value";
        let encoded = url_encode(original).unwrap();
        let decoded = url_decode(&encoded).unwrap();
        assert_eq!(decoded, original);
    }

    #[test]
    fn test_http_status_ok() {
        assert!(http_status_ok(200).unwrap());
        assert!(http_status_ok(201).unwrap());
        assert!(http_status_ok(204).unwrap());
        assert!(!http_status_ok(400).unwrap());
        assert!(!http_status_ok(404).unwrap());
        assert!(!http_status_ok(500).unwrap());
    }

    #[test]
    fn test_http_status_text() {
        assert_eq!(http_status_text(200).unwrap(), "OK");
        assert_eq!(http_status_text(404).unwrap(), "Not Found");
        assert_eq!(http_status_text(500).unwrap(), "Internal Server Error");
    }
}
