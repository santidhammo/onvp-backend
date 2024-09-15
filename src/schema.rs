#[cfg(not(test))]
include!("schema_build.rs");

#[cfg(test)]
include!("schema_test.rs");
