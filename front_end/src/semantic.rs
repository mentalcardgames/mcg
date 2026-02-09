// use std::{collections::HashMap};

// use crate::{spanned_ast::*, spans::SID, symbols::Var, walker::{AstPass, NodeKind}};

// pub enum TypeError {
//   KeyNotInPrecedence { var: Var },
//   KeyNotInPointMap { var: Var },
//   KeyAndStringDontAllign { var: Var },
// }

// pub enum SemanticOp {
//   CreateCard {
//     k_vs: (SID, Vec<SID>),
//   },
//   CreatePrecedence {
//     k_p: (SID, SID),
//   },
//   CreatePointMap {
//     k_p: (SID, SID),
//   },
//   UseKeyValue {
//     k_v: (SID, SID),
//   },
//   UseKeyPrecedence {
//     k_p: (SID, SID),
//   },
//   UseKeyPointMap {
//     k_p: (SID, SID),
//   },
//   UseKeyString {
//     k_s: (SID, SID),
//   },
// }

// pub struct SemanticVisitor {
//   ops: Vec<SemanticOp>
// }

// impl SemanticVisitor {
//   pub fn new() -> Self {
//     SemanticVisitor { 
//       ops: Vec::new()
//     }
//   }

//   // pub fn type_check(&self) -> Option<Vec<TypeError>> {


//   // }
// }

// impl AstPass for SemanticVisitor {
//     fn enter_node<T: crate::walker::Walker>(&mut self, node: &T)
//     where
//         Self: Sized {
//         match node.kind() {
//           NodeKind::SetUpRule(s) => {
//             match s {
//               SetUpRule::CreatePrecedence(precedence, key_value_pairs) => {
//                 self.ops.push(
//                   SemanticOp::CreatePrecedence { 
//                     k_p:  
//                       (key_value_pairs.first().unwrap().0.clone(), precedence.clone())
//                   }
//                 );
//                 for (k, v) in key_value_pairs.iter() {
//                   self.ops.push(
//                   SemanticOp::UseKeyValue { 
//                     k_v:  
//                       (k.clone(), v.clone())
//                     }
//                   );
//                 }
//               },
//               SetUpRule::CreatePointMap(pointmap, key_value_int_triples) => {
//                 self.ops.push(
//                   SemanticOp::CreatePointMap { 
//                     k_p:  
//                       (key_value_int_triples.first().unwrap().0.clone(), pointmap.clone())
//                   }
//                 );
//                 for (k, v, _) in key_value_int_triples.iter() {
//                   self.ops.push(
//                   SemanticOp::UseKeyValue { 
//                     k_v:  
//                       (k.clone(), v.clone())
//                     }
//                   );
//                 }
//               },
//               SetUpRule::CreateCardOnLocation(_, types) => {
//                 for (k, vs) in types.node.types.iter() {
//                   self.ops.push(
//                     SemanticOp::CreateCard { 
//                       k_vs:
//                         (k.clone(), vs.clone()) 
//                     }
//                   );
//                 }
//               },
//               _ => {},
//             }
//           },
//           NodeKind::AggregateFilter(a) => {
//             match a {
//                 AggregateFilter::Adjacent(spanned, spanned1) => {
//                   self.ops.push(
//                     SemanticOp::CreatePrecedence { 
//                       k_p:  
//                         (spanned.clone(), spanned1.clone())
//                     }
//                   );
//                 },
//                 AggregateFilter::Higher(spanned, spanned1) => {
//                   self.ops.push(
//                     SemanticOp::CreatePrecedence { 
//                       k_p:  
//                         (spanned.clone(), spanned1.clone())
//                     }
//                   );
//                 },
//                 AggregateFilter::Lower(spanned, spanned1) => {
//                   self.ops.push(
//                     SemanticOp::CreatePrecedence { 
//                       k_p:  
//                         (spanned.clone(), spanned1.clone())
//                     }
//                   );
//                 },
//                 AggregateFilter::KeyString(key, _, spanned2) => {
//                   match &spanned2.node {
//                     StringExpr::Query(spanned) => {
//                       match &spanned.node {
//                         QueryString::KeyOf(key_of_string, _) => {
//                           self.ops.push(
//                             SemanticOp::UseKeyString { 
//                               k_s: 
//                                 (key.clone(), key_of_string.clone())
//                             }
//                           );
//                         },
//                         _ => {}
//                       }
//                     },
//                     _ => {}
//                   }
//                 },
//                 _ => {}
//             }
//           },
//           _ => {},
//         }
//     }

//     fn exit_node<T: crate::walker::Walker>(&mut self, _: &T)
//     where
//         Self: Sized {
//     }
// }