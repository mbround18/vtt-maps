use actix_web::{HttpRequest, error::ErrorInternalServerError};
use kuchikiki::{parse_html, traits::TendrilSink};

pub struct SeoData {
    pub title: String,
    pub description: String,
    pub keywords: Option<String>,
    pub image_url: String,
}

pub fn inject_seo_metadata(
    html: String,
    req: &HttpRequest,
    seo: SeoData,
) -> Result<String, actix_web::Error> {
    let connection_info = req.connection_info();
    let scheme = connection_info.scheme();
    let host = connection_info.host();
    let uri = req.uri();
    let canonical = format!("{scheme}://{host}{uri}");
    let keywords = seo.keywords.unwrap_or_default();

    let blob = format!(
        r#"
    <title>{title}</title>
    <meta name="description" content="{desc}" />
    <meta name="keywords"    content="{kw}"  />
    <link rel="canonical"     href="{can}" />
    
    <!-- Open Graph -->
    <meta property="og:type"        content="website" />
    <meta property="og:site_name"   content="D&amp;D VTT Maps" />
    <meta property="og:title"       content="{title}" />
    <meta property="og:description" content="{desc}"  />
    <meta property="og:url"         content="{can}"   />
    <meta property="og:image"       content="{img}"   />
    
    <!-- Twitter Card -->
    <meta name="twitter:card"        content="summary_large_image" />
    <meta name="twitter:title"       content="{title}" />
    <meta name="twitter:description" content="{desc}"  />
    <meta name="twitter:image"       content="{img}"   />
    "#,
        title = seo.title,
        desc = seo.description,
        kw = keywords,
        can = canonical,
        img = seo.image_url,
    );

    let fragment_document = parse_html().one(format!("<head>{blob}</head>"));
    let document = parse_html().one(html);
    if let (Ok(head), Ok(fragment_head)) = (
        document.document_node.select_first("head"),
        fragment_document.document_node.select_first("head"),
    ) {
        for child in fragment_head.as_node().children() {
            head.as_node().append(child.clone());
        }
    }

    let mut out = Vec::new();
    document
        .document_node
        .serialize(&mut out)
        .map_err(|_| ErrorInternalServerError("Serialization failed"))?;

    String::from_utf8(out).map_err(|_| ErrorInternalServerError("UTF-8 conversion failed"))
}
