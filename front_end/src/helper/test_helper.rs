use crate::{diagnostic::OwnedSpan, symbols::Var, typed_ast::*};

// pub fn ctx_max_cardpos() -> LoweringCtx {
//   LoweringCtx::new(vec![
//     (
//       Var {
//         id: "Aces".to_string(),
//         span: OwnedSpan {
//           start: 0,
//           end: 0,
//         },
//       },
//       GameType::PointMap
//     )
//   ])
// }

pub fn id(id: &str) -> TypedID {
  TypedID {
    id: id.to_string(),
    ty: GameType::NoType,
  }
}

pub fn stage(id: &str) -> TypedID {
  TypedID {
    id: id.to_string(),
    ty: GameType::Stage,
  }
}

pub fn playername(id: &str) -> TypedID {
  TypedID {
    id: id.to_string(),
    ty: GameType::Player,
  }
}

pub fn teamname(id: &str) -> TypedID {
  TypedID {
    id: id.to_string(),
    ty: GameType::Team,
  }
}

pub fn location(id: &str) -> TypedID {
  TypedID {
    id: id.to_string(),
    ty: GameType::Location,
  }
}

pub fn token(id: &str) -> TypedID {
  TypedID {
    id: id.to_string(),
    ty: GameType::Token,
  }
}

pub fn precedence(id: &str) -> TypedID {
  TypedID {
    id: id.to_string(),
    ty: GameType::Precedence,
  }
}

pub fn pointmap(id: &str) -> TypedID {
  TypedID {
    id: id.to_string(),
    ty: GameType::PointMap,
  }
}

pub fn combo(id: &str) -> TypedID {
  TypedID {
    id: id.to_string(),
    ty: GameType::Combo,
  }
}

pub fn memory(id: &str) -> TypedID {
  TypedID {
    id: id.to_string(),
    ty: GameType::Memory,
  }
}

pub fn key(id: &str) -> TypedID {
  TypedID {
    id: id.to_string(),
    ty: GameType::Key,
  }
}

pub fn value(id: &str) -> TypedID {
  TypedID {
    id: id.to_string(),
    ty: GameType::Value,
  }
}
