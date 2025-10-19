use poem_openapi::Tags;

/// Root of messaging HTTP API.
///
/// Used with `poem` HTTP server.
pub struct Endpoints;

/// Enum of API sections (tags)
#[derive(Tags)]
pub enum EndpointsTags {
    Health,
}
