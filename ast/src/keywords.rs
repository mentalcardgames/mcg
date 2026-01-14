// ------------------------
// Keywords declarations
// ------------------------

pub mod kw {
  use std::collections::HashSet;

  syn::custom_keyword!(position);
  syn::custom_keyword!(score);
  syn::custom_keyword!(choose);
  syn::custom_keyword!(optional);
  syn::custom_keyword!(next);
  syn::custom_keyword!(turn);
  syn::custom_keyword!(winner);
  syn::custom_keyword!(demand);
  syn::custom_keyword!(cycle);
  syn::custom_keyword!(bid);
  syn::custom_keyword!(successful);
  syn::custom_keyword!(fail);
  syn::custom_keyword!(set);
  syn::custom_keyword!(shuffle);
  syn::custom_keyword!(flip);
  syn::custom_keyword!(combo);
  syn::custom_keyword!(memory);
  syn::custom_keyword!(pointmap);
  syn::custom_keyword!(precedence);
  syn::custom_keyword!(token);
  syn::custom_keyword!(random);
  syn::custom_keyword!(location);
  syn::custom_keyword!(table);
  syn::custom_keyword!(on);
  syn::custom_keyword!(card);
  syn::custom_keyword!(with);
  syn::custom_keyword!(place);
  syn::custom_keyword!(exchange);
  syn::custom_keyword!(deal);
  syn::custom_keyword!(range);
  syn::custom_keyword!(from);
  syn::custom_keyword!(to);
  syn::custom_keyword!(until);
  syn::custom_keyword!(end);
  syn::custom_keyword!(times);
  syn::custom_keyword!(cards);
  syn::custom_keyword!(face);
  syn::custom_keyword!(down);
  syn::custom_keyword!(up);
  syn::custom_keyword!(private);
  syn::custom_keyword!(all);
  syn::custom_keyword!(any);
  syn::custom_keyword!(current);
  syn::custom_keyword!(previous);
  syn::custom_keyword!(owner);
  syn::custom_keyword!(of);
  syn::custom_keyword!(highest);
  syn::custom_keyword!(lowest);
  syn::custom_keyword!(competitor);
  syn::custom_keyword!(turnorder);
  syn::custom_keyword!(top);
  syn::custom_keyword!(bottom);
  syn::custom_keyword!(team);
  syn::custom_keyword!(at);
  syn::custom_keyword!(using);
  syn::custom_keyword!(prec);
  syn::custom_keyword!(point);
  syn::custom_keyword!(min);
  syn::custom_keyword!(max);
  syn::custom_keyword!(stageroundcounter);
  syn::custom_keyword!(size);
  syn::custom_keyword!(sum);
  syn::custom_keyword!(or);
  syn::custom_keyword!(and);
  syn::custom_keyword!(stage);
  syn::custom_keyword!(game);
  syn::custom_keyword!(not);
  syn::custom_keyword!(is);
  syn::custom_keyword!(empty);
  syn::custom_keyword!(out);
  syn::custom_keyword!(players);
  syn::custom_keyword!(playersin);
  syn::custom_keyword!(playersout);
  syn::custom_keyword!(others);
  syn::custom_keyword!(lower);
  syn::custom_keyword!(higher);
  syn::custom_keyword!(adjacent);
  syn::custom_keyword!(distinct);
  syn::custom_keyword!(same);
  syn::custom_keyword!(key);
  syn::custom_keyword!(other);
  syn::custom_keyword!(teams);
  syn::custom_keyword!(player);
  syn::custom_keyword!(locations);
  syn::custom_keyword!(ints);

  pub fn in_custom_key_words<T: ToString>(value: &T) -> bool {
    let custom_keywords: HashSet<String> = vec![
        "position", "score", "choose", "optional", "next", "turn",
        "winner", "demand", "cycle", "bid", "successful", "fail",
        "set", "shuffle", "flip", "combo", "memory", "pointmap",
        "precedence", "token", "random", "location", "table", "on",
        "card", "with", "place", "exchange", "deal", "range", "from",
        "to", "until", "end", "times", "cards", "face", "down", "up",
        "private", "all", "any", "current", "previous", "owner", "of",
        "highest", "lowest", "competitor", "turnorder", "top", "bottom",
        "team", "at", "using", "prec", "point", "min", "max",
        "stageroundcounter", "size", "sum", "or", "and", "stage", "game",
        "not", "is", "empty", "out", "players", "playersin", "playersout",
        "others", "lower", "higher", "adjacent", "distinct", "same", "key",
        "other", "teams", "player", "locations", "ints",
    ].iter().map(|x| x.to_string()).collect();

    if custom_keywords.contains(&value.to_string()) {
      return true
    }

    return false
  }
}
