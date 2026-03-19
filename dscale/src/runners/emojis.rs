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

pub fn good() -> &'static str {
    GOOD[rand::random::<u64>() as usize % GOOD.len()]
}

pub fn bad() -> &'static str {
    BAD[rand::random::<u64>() as usize % BAD.len()]
}
