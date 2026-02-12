export const controlKeywords = [
  "stage",
  "if",
  "optional",
  "choose",
  "case",
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


// Old Code
// "semanticTokenScopes": [
//   {
//     "language": "cgdsl",
//     "scopes": {
//       "player":     ["entity.name.type"],
//       "team":       ["entity.name.class"],
//       "location":   ["entity.name.class"],
//       "precedence": ["keyword.other.unit"],
//       "pointmap":   ["keyword.other.unit"],
//       "combo":      ["entity.name.type.enum"],
//       "key":        ["entity.name.type.enum"],
//       "value":      ["entity.name.type.enum"],
//       "memory":     ["entity.name.type.enum"],
//       "token":      ["entity.name.type.enum"],
//       "stage":      ["entity.name.type.enum"]
//     }
//   }
// ]  

// Updated Change:
// "configurationDefaults": {
//       "editor.tokenColorCustomizations": {
//         "semanticTokenColors": {
//           "player": "#388828",
//           "team": "#0a869c",
//           "location": "#7a0a9c",
//           "precedence": "#9c8d0a",
//           "pointmap": "#b9830e",
//           "combo": "#410eb9",
//           "key": "#0d5f0a",
//           "value": "#0a5f5b",
//           "memory": "#db8d17",
//           "token": "#171ad1",
//           "stage": "#d11746",
//           "notype": "#FB7185"
//         }
//       }
//     }


// Custom color
// "contributes": {
//   "configurationDefaults": {
//     "editor.tokenColorCustomizations": {
//       "semanticTokenColors": {
//         "player": {
//           "foreground": "#d11746",
//           "fontStyle": "bold"
//         },
//         "team": "#60A5FA",
//         "location": "#34D399",
//         "precedence": "#FB7185",
//         "memory": {
//           "foreground": "#A78BFA",
//           "fontStyle": "italic"
//         }
//       }
//     }
//   }
// }

// "editor.tokenColorCustomizations": {
//   "[*Dark*]": {
//     "semanticTokenColors": {
//       "player": "#FF8888"
//     }
//   },
//   "[*Light*]": {
//     "semanticTokenColors": {
//       "player": "#AA0000"
//     }
//   }
// }



// Semantic Toke Scopes ordered:
// "semanticTokenScopes": [
//     {
//       "language": "cgdsl",
//       "scopes": {
//         "player":     ["entity.name.class"],
//         "team":       ["entity.name.class"],

//         "precedence": ["entity.name.namespace"],
//         "pointmap":   ["entity.name.namespace"],
//         "combo":      ["entity.name.namespace"],

//         "key":        ["variable.parameter"],
//         "value":      ["variable.other"],

//         "memory":     ["support.variable"],
//         "token":      ["support.variable"],
//         "location":   ["support.variable"],

//         "stage":      ["entity.name.enum"],

//         "notype":     ["entity.name.type"]
//       }
//     }
//   ]