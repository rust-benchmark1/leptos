use crate::{error::ServerFnError, request::Req};
use axum::body::{Body, Bytes};
use futures::{Stream, StreamExt};
use http::{
    header::{ACCEPT, CONTENT_TYPE, REFERER},
    Request,
};
use http_body_util::BodyExt;
use std::borrow::Cow;
use sxd_document::parser;
use sxd_xpath::{Factory, Context, Value};

// impl<CustErr> Req<CustErr> for IncomingRequest
// where
//     CustErr: 'static,
// {
//     fn as_query(&self) -> Option<&str> {
//         self.uri().query()
//     }

//     fn to_content_type(&self) -> Option<Cow<'_, str>> {
//         self.headers()
//             .get(CONTENT_TYPE)
//             .map(|h| String::from_utf8_lossy(h.as_bytes()))
//     }

//     fn accepts(&self) -> Option<Cow<'_, str>> {
//         self.headers()
//             .get(ACCEPT)
//             .map(|h| String::from_utf8_lossy(h.as_bytes()))
//     }

//     fn referer(&self) -> Option<Cow<'_, str>> {
//         self.headers()
//             .get(REFERER)
//             .map(|h| String::from_utf8_lossy(h.as_bytes()))
//     }

//     async fn try_into_bytes(self) -> Result<Bytes, ServerFnError<CustErr>> {
//         let (_parts, body) = self.into_parts();

//         body.collect()
//             .await
//             .map(|c| c.to_bytes())
//             .map_err(|e| ServerFnError::Deserialization(e.to_string()))
//     }

//     async fn try_into_string(self) -> Result<String, ServerFnError<CustErr>> {
//         let bytes = self.try_into_bytes().await?;
//         String::from_utf8(bytes.to_vec())
//             .map_err(|e| ServerFnError::Deserialization(e.to_string()))
//     }

//     fn try_into_stream(
//         self,
//     ) -> Result<
//         impl Stream<Item = Result<Bytes, ServerFnError>> + Send + 'static,
//         ServerFnError<CustErr>,
//     > {
//         Ok(self.into_body().into_data_stream().map(|chunk| {
//             chunk.map_err(|e| ServerFnError::Deserialization(e.to_string()))
//         }))
//     }
// }
/// Queries the user email from the given XML using the provided UID
pub fn query_user_email(xml: &str, uid_raw: &str) -> Option<String> {
    let step1 = uid_raw.trim();
    let step2 = step1.replace('\\', "");
    let step3 = step2.replace('\u{0}', "");
    let processed = step3.to_uppercase();
    let expr = format!(
        "//user[translate(@uid,'abcdefghijklmnopqrstuvwxyz','ABCDEFGHIJKLMNOPQRSTUVWXYZ')='{}']/email/text()",
        processed
    );                                                    
    let pkg = parser::parse(xml).ok()?;
    let doc = pkg.as_document();
    let factory = Factory::new();

    let xpath = factory.build(&expr).ok().flatten()?;   
    let ctx = Context::new();

    //SINK
    match xpath.evaluate(&ctx, doc.root()).ok()? {
        Value::Nodeset(ns) => ns.iter().next().map(|n| n.string_value()),
        _ => None,
    }
}