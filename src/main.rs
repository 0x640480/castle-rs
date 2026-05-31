//! CLI that mints an `X-CRS-Req-Token` using the bundled Chrome 148 macOS
//! profile and prints it to stdout.

use clap::Parser;

use castle_token::fingerprint;
use castle_token::token::{mint_fresh_default, MintOptions};

/// Mint an X-CRS-Req-Token using the bundled Chrome 148 macOS profile.
#[derive(Parser)]
#[command(name = "castle-token", version, about)]
struct Cli {
    /// 32-hex __cuid cookie value (defaults to a fresh random one).
    #[arg(long)]
    cuid: Option<String>,

    /// Init timestamp in ms; 0 means now-4000ms.
    #[arg(long = "init-time-ms", default_value_t = 0)]
    init_time_ms: i64,

    /// Castle deployment public key, format pk_<32 chars>.
    #[arg(long)]
    pk: String,

    /// Castle deployment integration group ID.
    #[arg(long)]
    ig: i64,
}

fn main() {
    let cli = Cli::parse();

    let cuid = cli
        .cuid
        .unwrap_or_else(|| hex::encode(rand::random::<[u8; 16]>()));
    let init_time_ms = (cli.init_time_ms != 0).then_some(cli.init_time_ms);

    let result = mint_fresh_default(&MintOptions {
        cuid: &cuid,
        fingerprint: fingerprint::chrome_148_macos(),
        init_time_ms,
        pk: &cli.pk,
        ig: cli.ig,
        now_ms: None,
    });

    match result {
        Ok(token) => println!("{token}"),
        Err(e) => {
            eprintln!("castle-token: {e}");
            std::process::exit(1);
        }
    }
}
