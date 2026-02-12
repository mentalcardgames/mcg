import * as fs from 'fs';
import { 
    controlKeywords, 
    helperKeywords, 
    actionKeywords, 
    creationKeywords, 
    runtimeKeywords, 
    filterKeywords,
    operationKeywords,
    positionalKeywords,
    extremaKeywords,
    statusKeywords,
    quantifierKeywords
} from './keywords';

function wordRegex(words: string[]) {
  // Escaping backslashes for JSON string compatibility
  return `\\b(${words.join("|")})\\b`;
}

const grammar = {
  $schema: "https://json.schemastore.org/tmlanguage.json",
  name: "cgdsl",
  scopeName: "source.cgdsl",
  patterns: [
    { include: "#punctuation" },
    { include: "#control" },
    { include: "#actions" },
    { include: "#helper" },
    { include: "#filter" },
    { include: "#quantifier" },
    { include: "#operation" },
    { include: "#positional" },
    { include: "#extrema" },
    { include: "#status" },
    { include: "#creation" },
    { include: "#runtime" },
    { include: "#ints" },
    { include: "#strings" }
  ],
  repository: {

    control: {
      patterns: [
        {
          name: "keyword.control.flow.cgdsl",
          match: wordRegex(controlKeywords)
        }
      ]
    },

    actions: {
      patterns: [
        {
          name: "keyword.control.action.cgdsl",
          match: wordRegex(actionKeywords)
        }
      ]
    },

    status: {
      patterns: [
        {
          name: "keyword.control.state.cgdsl",
          match: wordRegex(statusKeywords)
        }
      ]
    },

    creation: {
      patterns: [
        {
          name: "keyword.control.create.cgdsl",
          match: wordRegex(creationKeywords)
        }
      ]
    },

    helper: {
      patterns: [
        {
          name: "keyword.other.helper.cgdsl",
          match: wordRegex(helperKeywords)
        }
      ]
    },

    filter: {
      patterns: [
        {
          name: "entity.name.function.filter.cgdsl",
          match: wordRegex(filterKeywords)
        }
      ]
    },

    quantifier: {
      patterns: [
        {
          name: "keyword.operator.quantifier.cgdsl",
          match: wordRegex(quantifierKeywords)
        }
      ]
    },

    operation: {
      patterns: [
        {
          name: "keyword.operator.arithmetic.cgdsl",
          match: wordRegex(operationKeywords)
        }
      ]
    },

    positional: {
      patterns: [
        {
          name: "support.function.query.cgdsl",
          match: wordRegex(positionalKeywords)
        }
      ]
    },

    extrema: {
      patterns: [
        {
          name: "support.function.query.cgdsl",
          match: wordRegex(extremaKeywords)
        }
      ]
    },

    runtime: {
      patterns: [
        {
          name: "support.type.runtime.cgdsl",
          match: wordRegex(runtimeKeywords)
        }
      ]
    },

    ints: {
      patterns: [{
        name: "constant.numeric.integer.cgdsl",
        match: "\\b[0-9]+\\b"
      }]
    },

    strings: {
      name: "string.quoted.double.cgdsl",
      begin: "\"",
      end: "\"",
      patterns: [
        {
          name: "constant.character.escape.cgdsl",
          match: "\\\\."
        }
      ]
    }
  }
};

fs.writeFileSync('./syntaxes/cgdsl.tmLanguage.json', JSON.stringify(grammar, null, 2));
console.log("Grammar generated successfully!");