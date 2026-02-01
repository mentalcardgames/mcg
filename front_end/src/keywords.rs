// // ------------------------
// // Keywords declarations
// // ------------------------

// pub mod kw {
//   use std::collections::HashSet;

//   kwposition = { "position" }
//   kwscore = { "score" }
//   kwchoose = { "choose" }
//   kwoptional = { "optional" }
//   kwnext = { "next" }
//   kwturn = { "turn" }
//   kwwinner = { "winner" }
//   kwdemand = { "demand" }
//   kwcycle = { "cycle" }
//   kwbid = { "bid" }
//   kwsuccessful = { "successful" }
//   kwfail = { "fail" }
//   kwset = { "set" }
//   kwshuffle = { "shuffle" }
//   kwflip = { "flip" }
//   kwcombo = { "combo" }
//   kwmemory = { "memory" }
//   kwpointmap = { "pointmap" }
//   kwprecedence = { "precedence" }
//   kwtoken = { "token" }
//   kwrandom = { "random" }
//   kwlocation = { "location" }
//   kwtable = { "table" }
//   kwon = { "on" }
//   kwcard = { "card" }
//   kwwith = { "with" }
//   kwplace = { "place" }
//   kwexchange = { "exchange" }
//   kwdeal = { "deal" }
//   kwrange = { "range" }
//   kwfrom = { "from" }
//   kwto = { "to" }
//   kwuntil = { "until" }
//   kwend = { "end" }
//   kwtimes = { "times" }
//   kwcards = { "cards" }
//   kwface = { "face" }
//   kwdown = { "down" }
//   kwup = { "up" }
//   kwprivate = { "private" }
//   kwall = { "all" }
//   kwany = { "any" }
//   kwcurrent = { "current" }
//   kwprevious = { "previous" }
//   kwowner = { "owner" }
//   kwof = { "of" }
//   kwhighest = { "highest" }
//   kwlowest = { "lowest" }
//   kwcompetitor = { "competitor" }
//   kwturnorder = { "turnorder" }
//   kwtop = { "top" }
//   kwbottom = { "bottom" }
//   kwteam = { "team" }
//   kwat = { "at" }
//   kwusing = { "using" }
//   kwprec = { "prec" }
//   kwpoint = { "point" }
//   kwmin = { "min" }
//   kwmax = { "max" }
//   kwstageroundcounter = { "stageroundcounter" }
//   kwsize = { "size" }
//   kwsum = { "sum" }
//   kwor = { "or" }
//   kwand = { "and" }
//   kwstage = { "stage" }
//   kwgame = { "game" }
//   kwnot = { "not" }
//   kwis = { "is" }
//   kwempty = { "empty" }
//   kwout = { "out" }
//   kwplayers = { "players" }
//   kwplayersin = { "playersin" }
//   kwplayersout = { "playersout" }
//   kwothers = { "others" }
//   kwlower = { "lower" }
//   kwhigher = { "higher" }
//   kwadjacent = { "adjacent" }
//   kwdistinct = { "distinct" }
//   kwsame = { "same" }
//   kwkey = { "key" }
//   kwother = { "other" }
//   kwteams = { "teams" }
//   kwplayer = { "player" }
//   kwlocations = { "locations" }
//   kwints = { "ints" }

//   pub fn in_custom_key_words<T: ToString>(value: &T) -> bool {
//     let custom_keywords: HashSet<String> = vec![
//         "position", "score", "choose", "optional", "next", "turn",
//         "winner", "demand", "cycle", "bid", "successful", "fail",
//         "set", "shuffle", "flip", "combo", "memory", "pointmap",
//         "precedence", "token", "random", "location", "table", "on",
//         "card", "with", "place", "exchange", "deal", "range", "from",
//         "to", "until", "end", "times", "cards", "face", "down", "up",
//         "private", "all", "any", "current", "previous", "owner", "of",
//         "highest", "lowest", "competitor", "turnorder", "top", "bottom",
//         "team", "at", "using", "prec", "point", "min", "max",
//         "stageroundcounter", "size", "sum", "or", "and", "stage", "game",
//         "not", "is", "empty", "out", "players", "playersin", "playersout",
//         "others", "lower", "higher", "adjacent", "distinct", "same", "key",
//         "other", "teams", "player", "locations", "ints",
//     ].iter().map(|x| x.to_string()).collect();

//     if custom_keywords.contains(&value.to_string()) {
//       return true
//     }

//     return false
//   }
// }
