use std::process::exit;

use log::{error, info};

const GOOD: &[&str] = &[
    "ヽ('ー`)ノ",
    "ヽ(^‿^)ノ",
    "＼(＾▽＾)／",
    "(´▽`)/",
    "(*^▽^*)",
    "ヽ(•‿•)ノ",
    "(^_^)/",
    "٩(◕‿◕)۶",
    "ヾ(^∇^)",
    "(^o^)/",
];

const BAD: &[&str] = &[
    "(ﾉಥ益ಥ）ﾉ ┻━┻",
    "ヽ(ಠ_ಠ)ﾉ",
    "(╯°□°）╯︵ ┻━┻",
    "ಠ╭╮ಠ",
    "щ(ಠ益ಠщ)",
    "ヽ(`Д´)ﾉ",
    "凸(ಠ_ಠ)凸",
    "ಠ_ಠ",
    "(；一_一)",
    "o(TヘTo)",
];

fn good() -> &'static str {
    GOOD[rand::random::<u64>() as usize % GOOD.len()]
}

fn bad() -> &'static str {
    BAD[rand::random::<u64>() as usize % BAD.len()]
}

pub(super) fn looks_good() {
    info!("Looks good! {}", good());
}

pub(super) fn deadlock(reason: &str) {
    error!(
        "DEADLOCK! {}, Reason: {reason}\nTry using simple runner with RUST_LOG=debug",
        bad()
    );
    exit(1)
}
