const GOOD: &[&str] = &[
    "ヽ('ー`)ノ",
    "ヽ(^‿^)ノ",
    "(ﾉ◕ヮ◕)ﾉ*:･ﾟ✧",
    "＼(＾▽＾)／",
    "(づ｡◕‿‿◕｡)づ",
    "✧ヽ(°▽°)ﾉ✧",
];

const BAD: &[&str] = &[
    "(ﾉಥ益ಥ）ﾉ ┻━┻",
    "ヽ(ಠ_ಠ)ﾉ",
    "(╯°□°）╯︵ ┻━┻",
    "ಠ╭╮ಠ",
    "(҂◡_◡) ᕤ",
    "щ(ಠ益ಠщ)",
];

pub fn good() -> &'static str {
    GOOD[rand::random::<usize>() % GOOD.len()]
}

pub fn bad() -> &'static str {
    BAD[rand::random::<usize>() % BAD.len()]
}
