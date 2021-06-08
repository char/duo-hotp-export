use reqwest::{blocking::Client, header::USER_AGENT};
use serde_json::Value;

use clap::{App, Arg};

fn decode_qr(qr: &str) -> (String, String) {
    let img = image::open(qr).expect("Failed to open QR image");
    let decoder = bardecoder::default_decoder();

    let result = decoder
        .decode(&img)
        .remove(0)
        .expect("Failed to decode QR image");

    let components = result.split('-').collect::<Vec<_>>();
    if let [code, authority] = components.as_slice() {
        let authority = base64::decode(authority).expect("Could not decode API authority");
        let authority = String::from_utf8(authority).expect("API authority was malformed!");

        (code.to_string(), authority)
    } else {
        panic!("Failed to decode QR data")
    }
}

fn fetch_api_details(authority: &str, code: &str, counter: u32) -> String {
    let url = format!(
        "https://{}/push/v2/activation/{}?customer_protocol=1",
        authority, code
    );

    let client = Client::new();
    let request = client
        .post(url)
        .header(USER_AGENT, "okhttp/2.7.5" /* Emulate an Android App */)
        .form(&[
            // Data from github.com/chris4795/duo-cli; might want to throw a proxy at a UK request, though.
            ("jailbroken", "false"),
            ("architecture", "arm64"),
            ("region", "US"),
            ("app_id", "com.duosecurity.duomobile"),
            ("full_disk_encryption", "true"),
            ("passcode_status", "true"),
            ("platform", "Android"),
            ("app_version", "3.49.0"),
            ("app_build_number", "323001"),
            ("version", "11"),
            ("manufacturer", "unknown"),
            ("language", "en"),
            ("model", "Pixel 3a"),
            ("security_patch_level", "2021-02-01"),
        ])
        .build()
        .unwrap();

    let response = client
        .execute(request)
        .expect("Could not make request to duosecurity.com");
    let response = response
        .text()
        .expect("Could not decode duosecurity response as text");
    let response: Value =
        serde_json::from_str(&response).expect("Could not decode duosecurity response as JSON");

    let response = dbg!(response);

    let secret = response["response"]["hotp_secret"].as_str().unwrap();
    let secret = base32::encode(
        base32::Alphabet::RFC4648 { padding: false },
        secret.as_bytes(),
    );

    format!(
        "otpauth://hotp/Duo?secret={}&issuer=Duo&counter={}",
        secret, counter
    )
}

fn main() {
    let matches = App::new("duo-hotp-export")
        .version("1.0.0")
        .author("Charlotte Som <half-kh-hacker@hackery.site>")
        .about("HOTP URL exporter for Duo Security")
        .arg(
            Arg::with_name("qr-code")
                .value_name("FILE")
                .help("Sets the QR code to use in device activation")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::with_name("counter")
                .short("c")
                .long("counter")
                .value_name("N"),
        )
        .get_matches();

    let qr_code = matches.value_of("qr-code").unwrap_or("qr.png");
    let counter = matches
        .value_of("counter")
        .map(|s| s.parse::<u32>().ok())
        .flatten()
        .unwrap_or(1);

    let (code, authority) = decode_qr(qr_code);
    dbg!(&code, &authority);

    let otp_url = fetch_api_details(&authority, &code, counter);
    dbg!(&otp_url);

    let qr_code = qrcode::QrCode::new(&otp_url).unwrap();
    let qr_code = qr_code.render().light_color(' ').dark_color('#').build();
    println!("\n\n{}\n\n", qr_code);
}
