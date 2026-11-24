//! `TransformCondition` evaluation (ADR-0094 §3).
//!
//! Conditions are pure predicates over the *input* node (pre-remap) and the
//! input arena. `KindAt` resolves the child position by named role via
//! [`AstShapeDef::role_index`](handl_language::AstShapeDef::role_index)
//! (ADR-0081), falling back to the positional `index` for shapes that declare
//! no roles.

use handl_language::{Ast, AstNode, AstRef, Language, TransformCondition};

/// Evaluate `cond` against `node` in the input arena `ast`.
///
/// Returns `true` when the condition holds — i.e. the guarded rule may fire.
/// A condition that cannot be resolved (missing child, unknown role, absent
/// payload path) evaluates to `false`, never panics.
pub fn matches_condition(
    lang: &Language,
    ast: &Ast,
    node: &AstNode,
    cond: &TransformCondition,
) -> bool {
    match cond {
        TransformCondition::KindAt { index, role, kind } => {
            match resolve_child_index(lang, node, *index, role.as_deref()) {
                Some(old_idx) => ast
                    .get(AstRef::new(old_idx))
                    .map(|child| child.kind == *kind)
                    .unwrap_or(false),
                None => false,
            }
        }
        TransformCondition::AnyChildKind { kind } => node
            .payload
            .get("children")
            .and_then(|c| c.as_array())
            .map(|arr| {
                arr.iter().any(|v| {
                    v.as_u64()
                        .and_then(|idx| ast.get(AstRef::new(idx as usize)))
                        .map(|child| child.kind == *kind)
                        .unwrap_or(false)
                })
            })
            .unwrap_or(false),
        TransformCondition::PayloadEq { path, value } => payload_at(&node.payload, path)
            .map(|v| payload_eq(lang, node, v, value))
            .unwrap_or(false),
        TransformCondition::ChildPayloadEq {
            index,
            role,
            path,
            value,
        } => match resolve_child_index(lang, node, *index, role.as_deref()) {
            Some(old_idx) => ast
                .get(AstRef::new(old_idx))
                .and_then(|child| {
                    payload_at(&child.payload, path).map(|v| payload_eq(lang, child, v, value))
                })
                .unwrap_or(false),
            None => false,
        },
        TransformCondition::ChildCount(n) => node
            .payload
            .get("children")
            .and_then(|c| c.as_array())
            .map(|arr| arr.len() == *n)
            .unwrap_or(*n == 0),
        TransformCondition::AttrEq { path, attr, value } => attr_at(ast, node, path, attr)
            .map(|v| match value {
                Some(want) => &v == want,
                None => true,
            })
            .unwrap_or(false),
        TransformCondition::SameAttr {
            path_a,
            attr_a,
            path_b,
            attr_b,
        } => match (
            attr_at(ast, node, path_a, attr_a),
            attr_at(ast, node, path_b, attr_b),
        ) {
            (Some(a), Some(b)) => a == b,
            _ => false,
        },
        TransformCondition::And(conds) => {
            conds.iter().all(|c| matches_condition(lang, ast, node, c))
        }
        TransformCondition::Or(conds) => {
            conds.iter().any(|c| matches_condition(lang, ast, node, c))
        }
        TransformCondition::Not(cond) => !matches_condition(lang, ast, node, cond),
    }
}

/// Resolve the child position to test for [`TransformCondition::KindAt`].
///
/// A declared `role` is resolved through the node's [`AstShapeDef`] first
/// (ADR-0081); the positional `index` is the role-less fallback.
fn resolve_child_index(
    lang: &Language,
    node: &AstNode,
    index: Option<usize>,
    role: Option<&str>,
) -> Option<usize> {
    let pos = role
        .and_then(|r| {
            lang.ast
                .get_by_identity(&node.kind)
                .and_then(|shape| shape.role_index(r))
        })
        .or(index)?;
    node.payload
        .get("children")
        .and_then(|c| c.as_array())
        .and_then(|arr| arr.get(pos))
        .and_then(|v| v.as_u64())
        .map(|v| v as usize)
}

/// Payload equality for the `*PayloadEq` conditions.
///
/// Direct JSON equality, plus two documented extensions:
///
/// 1. **String-pattern operators.** When `value` is a *single-key*
///    object `{"$prefix"|"$suffix"|"$contains": "<lit>"}` and the
///    payload under test is a string, the string is compared by the
///    named affix/substring relation instead of JSON equality — the
///    `match:` escape hatch for name patterns (e.g. every
///    `deprecated_*` callee). This is deliberately *not* a regex
///    engine: no new dependency, no ReDoS surface, and the spelling
///    says what it does. A multi-key object, an unknown `$`-key, or a
///    non-string operand falls through to ordinary equality. Literal
///    equality is checked first, so a payload that genuinely holds a
///    `{"$prefix": …}`-shaped record still compares by value.
/// 2. **Name-leaf globals.** When the tested node is a *name leaf*
///    (its [`AstShapeDef`] declares `pattern_role: "leaf"` — e.g.
///    canonical `var`) and the payload under test is a string naming
///    an ambient global in [`Language::globals`], the name compares
///    equal to the global's bound value. Constants spelled as
///    references — the imported Bool globals `true`/`false`, which lex
///    as `var` nodes rather than `value` literals — therefore satisfy
///    `value: true` / `value: false` guards without the condition
///    vocabulary hardcoding global spellings.
fn payload_eq(
    lang: &Language,
    node: &AstNode,
    v: &serde_json::Value,
    value: &serde_json::Value,
) -> bool {
    if v == value {
        return true;
    }
    if let Some(s) = v.as_str()
        && let Some(op) = string_pattern_op(value)
    {
        return match op {
            ("$prefix", pat) => s.starts_with(pat),
            ("$suffix", pat) => s.ends_with(pat),
            ("$contains", pat) => s.contains(pat),
            _ => unreachable!("string_pattern_op only yields known keys"),
        };
    }
    let is_name_leaf = lang
        .ast
        .get_by_identity(&node.kind)
        .and_then(|shape| shape.pattern_role.as_deref())
        == Some("leaf");
    if !is_name_leaf {
        return false;
    }
    v.as_str()
        .and_then(|name| lang.globals.get(name))
        .map(|global| global == value)
        .unwrap_or(false)
}

/// Decode a `*PayloadEq` `value` operand as a string-pattern operator:
/// a single-key object `{"$prefix"|"$suffix"|"$contains": "<lit>"}`.
/// Returns `(operator, pattern)`; `None` for anything else (literal
/// values, multi-key objects, unknown `$`-keys, non-string patterns).
fn string_pattern_op(value: &serde_json::Value) -> Option<(&'static str, &str)> {
    let obj = value.as_object()?;
    if obj.len() != 1 {
        return None;
    }
    let (key, pat) = obj.iter().next()?;
    let op = match key.as_str() {
        "$prefix" => "$prefix",
        "$suffix" => "$suffix",
        "$contains" => "$contains",
        _ => return None,
    };
    Some((op, pat.as_str()?))
}

/// Resolve the attr `attr` on the node reached by child-index `path`
/// below `node` — the [`TransformCondition::AttrEq`] /
/// [`TransformCondition::SameAttr`] lookup. An empty `path` names
/// `node` itself. Returns the attr's payload value, or `None` when the
/// path does not resolve, the node has no `attrs` map, or the attr is
/// absent.
fn attr_at<'a>(
    ast: &'a Ast,
    node: &'a AstNode,
    path: &[usize],
    attr: &str,
) -> Option<serde_json::Value> {
    let mut current = node;
    for &step in path {
        let idx = current
            .payload
            .get("children")
            .and_then(|c| c.as_array())
            .and_then(|arr| arr.get(step))
            .and_then(|v| v.as_u64())
            .map(|v| v as usize)?;
        current = ast.get(AstRef::new(idx))?;
    }
    current
        .payload
        .get("attrs")
        .and_then(|a| a.get(attr))
        .cloned()
}

/// Navigate a dotted `path` (`.`-separated object keys; numeric segments
/// index into arrays) into `payload`.
fn payload_at<'a>(payload: &'a serde_json::Value, path: &str) -> Option<&'a serde_json::Value> {
    if path.is_empty() {
        return Some(payload);
    }
    let mut cur = payload;
    for seg in path.split('.') {
        cur = if let Some(idx) = seg.parse::<usize>().ok().filter(|_| cur.is_array()) {
            cur.get(idx)?
        } else {
            cur.get(seg)?
        };
    }
    Some(cur)
}

#[cfg(test)]
mod tests {
    use super::*;
    use handl_language::{Ast, AstNode, TransformCondition};
    use serde_json::json;

    fn node_with(payload: serde_json::Value) -> AstNode {
        let mut node = AstNode::new("call");
        node.payload = payload;
        node
    }

    fn payload_eq_fires(node: &AstNode, value: serde_json::Value) -> bool {
        matches_condition(
            &Language::new(),
            &Ast::new(),
            node,
            &TransformCondition::PayloadEq {
                path: "name".into(),
                value,
            },
        )
    }

    #[test]
    fn string_pattern_operators_match() {
        let node = node_with(json!({"name": "deprecated_foo"}));
        assert!(payload_eq_fires(&node, json!({"$prefix": "deprecated_"})));
        assert!(payload_eq_fires(&node, json!({"$suffix": "_foo"})));
        assert!(payload_eq_fires(&node, json!({"$contains": "cated_fo"})));
        assert!(!payload_eq_fires(&node, json!({"$prefix": "legacy_"})));
        assert!(!payload_eq_fires(&node, json!({"$suffix": "_bar"})));
        assert!(!payload_eq_fires(&node, json!({"$contains": "zzz"})));
    }

    #[test]
    fn pattern_ops_fall_back_to_literal_equality() {
        // Multi-key objects, unknown `$` keys, and non-string patterns
        // are ordinary JSON values, not operators.
        let node = node_with(json!({"name": "x"}));
        assert!(!payload_eq_fires(
            &node,
            json!({"$prefix": "x", "$suffix": "x"})
        ));
        assert!(!payload_eq_fires(&node, json!({"$regex": "x.*"})));
        assert!(!payload_eq_fires(&node, json!({"$prefix": 3})));
        // A payload that literally holds the operator-shaped record
        // still compares by value (literal equality runs first).
        let record = node_with(json!({"name": {"$prefix": "x"}}));
        assert!(payload_eq_fires(&record, json!({"$prefix": "x"})));
        // Non-string payloads never take the operator path.
        let num = node_with(json!({"name": 7}));
        assert!(!payload_eq_fires(&num, json!({"$prefix": "7"})));
    }
}
