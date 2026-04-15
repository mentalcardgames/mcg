export const controlKeywords = [
  "stage",
  "if",
  "optional",
  "choose",
  "case",
  "conditional",
  "else",
  "trigger",
];

export const helperKeywords = [
  "for", "is", "of", "in", "on", "as",
  "to", "using", "with", "where",
  "until", "from", "at", "cards", "out",
  "owner", "size", "fail", "successful",
  "table", "times"
];

export const actionKeywords = [
  "cycle", "move", "deal", "demand",
  "exchange", "place", "flip", "token",
  "reset", "set", "shuffle", "bid",
  "end stage", "end game", "end turn",
  "score", "winner"
];

export const filterKeywords = [
  "adjacent", "distinct", "empty", "higher",
  "lower", "same",
];

export const quantifierKeywords = [
  "all", "any"
];

export const operationKeywords = [
  "and", "or", "not", "random", "sum"
];

export const positionalKeywords = [
  "bottom", "top"
];

export const extremaKeywords = [
  "highest", "lowest", "max", "min"
];

export const creationKeywords = [
  "card", "combo", "create", "location",
  "memory", "player", "points", "precedence",
  "team", "turnorder"
];

export const runtimeKeywords = [
  "competitor", "current", "next",
  "other", "others", "playersin", "playersout",
  "playroundcounter", "position", "previous",
  "stageroundcounter", "teams",
];

export const statusKeywords = [
  "face down", "face up", "private"
];

function wordRegex(words: [&String]) {
  return `\\b(${words.join("|")})\\b`;
}
