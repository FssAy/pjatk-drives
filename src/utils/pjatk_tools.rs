use crate::handler::endpoints::login::LoginData;
use reqwest::redirect::Policy;
use std::collections::HashMap;

/// Collection of tools for working with pjatk related entities
pub struct PjatkTools;

impl PjatkTools {
    // todo: Implement error handling
    /// Checks if provided credentials are valid by sending a request
    /// to the external authorization api
    ///
    /// * Redirections have to be blocked.
    /// * Any response status than 302 means failure
    /// * Around 10 requests can be sent before timeout will occur
    ///     and the account will be blocked for a while
    pub async fn are_credentials_valid(login_data: &LoginData) -> bool {
        let client = reqwest::ClientBuilder::new()
            .redirect(Policy::none())
            .build()
            .unwrap();

        let email = format!("{}@pjwstk.edu.pl", login_data.user);

        let mut params = HashMap::new();
        params.insert("UserName", email.as_str());
        params.insert("Password", login_data.password.as_str());
        params.insert("AuthMethod", "FormsAuthentication");

        // persistence of this url is unknown
        let auth_url = "https://adfs.pjwstk.edu.pl/adfs/oauth2/authorize/?client_id=dfbccf57-9\
        d86-4eac-aff7-1485aee6206e&redirect_uri=https://gakko.pjwstk.edu.pl/signin-oidc&response_typ\
        e=id_token&scope=openid profile&response_mode=form_post&nonce=637909981011482670.Zjc4ZDY2MDA\
        tYWYwNC00NjU1LTlmZDctNzNjYTFkYTBlZjUzYjdkYTQxZjgtODMyYy00Y2YyLWFmZDEtOWY1NzY5YjVlOTUx&state=\
        CfDJ8P2KDrn5KHVFg2kxlZyZgf_wN7RBqj0wm7rlA8NYfVhzrgzMSCyBT56jssrUmMrQxNAp-ci9Bw-Ql6SXsXBpl8yP\
        rTFr8S8Q0hP_lkYOKdj30l3YnXNsZ-JO2CUkFyJlOYyuJ4XvEuW_N09blixhq95Jbm-L5NV2sKivfdXF-uFShR2USaLE\
        oTxPJbRkjUj4LbjtWbAjwwUkp3SFpxW3WE8Lb-W3ofBcGyLJMFLCyM0KKa2xcgx10kRFyuItvCnnaSqYL0-GStOl3nt4\
        6iNTWWokcNAfQAde7-tBPQdomVlI&x-client-SKU=ID_NETSTANDARD2_0&x-client-ver=6.10.0.0&client-req\
        uest-id=74e31b04-9b58-4319-d154-0080030000f5";

        // most of the headers are non essential and can be removed
        let response = client
            .post(auth_url)
            .version(reqwest::Version::HTTP_11)
            .header("Host", "adfs.pjwstk.edu.pl")
            .header(
                "User-Agent",
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:100.0) Gecko/20100101 Firefox/100.0",
            )
            .header("Accept-Language", "en-US,en;q=0.5")
            .header("Accept-Encoding", "gzip, deflate, br")
            .header("Connection", "keep-alive")
            .header("Upgrade-Insecure-Requests", "1")
            .header("Sec-Fetch-Dest", "document")
            .header("Sec-Fetch-Mode", "navigate")
            .header("Sec-Fetch-Site", "cross-site")
            .header("Pragma", "no-cache")
            .header("Cache-Control", "no-cache")
            .header("Origin", "https://github.com")
            .header("DNT", "1")
            .form(&params)
            .send()
            .await
            .unwrap();

        response.status().as_u16() == 302
    }
}
