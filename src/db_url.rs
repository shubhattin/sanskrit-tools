//! Database URL normalization shared by the server and the importer.

/// sqlx reads `sslrootcert` as a file path, but libpq ≥ 16 (and PlanetScale's
/// connection strings) use `sslrootcert=system` for "the OS CA store". Drop
/// that value: `sslmode=verify-full` stays, and rustls verifies against its
/// bundled (webpki) roots. Any other value (a real path) is kept.
pub fn normalize(url: &str) -> String {
  let Some((base, query)) = url.split_once('?') else {
    return url.to_string();
  };
  let kept: Vec<&str> = query
    .split('&')
    .filter(|kv| !kv.eq_ignore_ascii_case("sslrootcert=system"))
    .collect();
  if kept.is_empty() {
    base.to_string()
  } else {
    format!("{base}?{}", kept.join("&"))
  }
}

#[cfg(test)]
mod tests {
  use super::normalize;
  #[test]
  fn drops_system_root_cert_only() {
    assert_eq!(
      normalize("postgresql://u:p@h:5432/db?sslmode=verify-full&sslrootcert=system"),
      "postgresql://u:p@h:5432/db?sslmode=verify-full"
    );
    assert_eq!(
      normalize("postgres://h/db?sslrootcert=system"),
      "postgres://h/db"
    );
    assert_eq!(
      normalize("postgres://h/db?sslrootcert=/etc/ca.pem"),
      "postgres://h/db?sslrootcert=/etc/ca.pem"
    );
    assert_eq!(normalize("postgres://h/db"), "postgres://h/db");
  }
}
