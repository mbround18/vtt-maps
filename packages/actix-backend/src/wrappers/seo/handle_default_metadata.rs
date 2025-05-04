use crate::wrappers::seo::inject_seo_metadata::{SeoData, inject_seo_metadata};
use actix_web::Error;

pub(crate) async fn handle_default_metadata(
    html: String,
    request: &actix_web::HttpRequest,
) -> Result<String, Error> {
    let seo = SeoData {
        title: "D&D VTT Maps – Free Virtual Tabletop Battle Maps".to_string(),
        description: "Discover a vast collection of free virtual tabletop maps for Dungeons & Dragons, FoundryVTT, and other VTT software. Elevate your campaigns with stunning, functional battle maps for RPG sessions.".to_string(),
        keywords: Some("Dungeons & Dragons, VTT Maps, FoundryVTT, Battle Maps, RPG, Free Maps".to_string()),
        image_url: "/assets/vtt-maps-logo.png".to_string(),
    };

    inject_seo_metadata(html, request, seo)
}
