use actix_web::{HttpRequest, error::ErrorInternalServerError};
use html5ever::{QualName, local_name, namespace_url};
use kuchikiki::traits::TendrilSink;
use kuchikiki::{parse_fragment, parse_html};

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
    let canonical = format!("{}://{}{}", scheme, host, uri);
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

    // 2) Parse the blob as a <head> fragment:
    let fragment = parse_fragment(
        QualName::new(
            None,
            namespace_url!("http://www.w3.org/1999/xhtml"),
            local_name!("head"),
        ),
        Vec::new(),
    )
    .one(blob);

    let document = parse_html().one(html);
    if let Ok(head) = document.select_first("head") {
        // 4) Move each child from the parsed fragment into the document's head:
        for child in fragment.children() {
            head.as_node().append(child.clone());
        }
    }

    // head.append(NodeRef::new_text(blob));

    let mut out = Vec::new();
    document
        .serialize(&mut out)
        .map_err(|_| ErrorInternalServerError("Serialization failed"))?;

    String::from_utf8(out).map_err(|_| ErrorInternalServerError("UTF-8 conversion failed"))
}
