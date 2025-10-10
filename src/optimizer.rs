use std::collections::{HashMap, HashSet};

use crate::intermediate_code_generator::{IRInstr, IRValue};

/// Optimize a vector of IR instructions.
/// Runs several passes until no more changes:
///  - constant folding
///  - constant / copy propagation
///  - dead code elimination
pub fn optimize_ir(mut code: Vec<IRInstr>) -> Vec<IRInstr> {
    loop {
        let before = code.len();

        // 1) Constant folding & propagation pass
        code = constant_fold_and_propagate(&code);

        // 2) Copy propagation pass (replace assigned temps/vars with their sources)
        code = copy_propagation(&code);

        // 3) Dead code elimination
        code = dead_code_elimination(&code);

        // stop when stable (no change in instruction count)
        if code.len() == before {
            break;
        }
    }

    code
}

// -----------------------------
// Pass: Constant folding + propagation
// -----------------------------
//
// Walks instructions in order and attempts to evaluate BinaryOp when operands are
// known constants (either literal or previously folded temps). It also tracks
// simple constant assignments (e.g., t1 = 5 or x = t1 where t1 is a constant).
fn constant_fold_and_propagate(code: &[IRInstr]) -> Vec<IRInstr> {
    let mut new_code = Vec::with_capacity(code.len());
    // map from name (var or temp string) to constant IRValue
    let mut consts: HashMap<String, IRValue> = HashMap::new();

    // helper: try to get a constant for a name (temp or var)
    let get_const = |name: &str, consts: &HashMap<String, IRValue>| -> Option<IRValue> {
        consts.get(name).cloned()
    };

    for instr in code {
        match instr {
            IRInstr::Assign(target, value) => {
                // If value is literal, record it as constant.
                // If value is a Temp or Var that maps to a constant, propagate.
                let resolved_value = match value {
                    IRValue::Int(_) | IRValue::Bool(_) | IRValue::Str(_) => Some(value.clone()),
                    IRValue::Temp(t) | IRValue::Var(t) => get_const(t, &consts),
                };

                if let Some(cv) = resolved_value {
                    // We can replace the assign with a direct constant assign
                    new_code.push(IRInstr::Assign(target.clone(), cv.clone()));
                    consts.insert(target.clone(), cv);
                } else {
                    // Not a known constant; keep the original assign and remove any const entry
                    new_code.push(IRInstr::Assign(target.clone(), value.clone()));
                    consts.remove(target);
                }
            }

            IRInstr::BinaryOp(result, left, op, right) => {
                // Try to resolve left/right into constants (either var/temp or literal already recorded)
                let left_const = get_const(left, &consts);
                let right_const = get_const(right, &consts);

                match (&left_const, &right_const) {
                    (Some(IRValue::Int(a)), Some(IRValue::Int(b))) => {
                        // integer arithmetic folding
                        let folded = match op.as_str() {
                            "+" => IRValue::Int(a + b),
                            "-" => IRValue::Int(a - b),
                            "*" => IRValue::Int(a * b),
                            "/" => {
                                if *b == 0 {
                                    new_code.push(IRInstr::BinaryOp(
                                        result.clone(),
                                        left.clone(),
                                        op.clone(),
                                        right.clone(),
                                    ));
                                    continue;
                                } else {
                                    IRValue::Int(a / b)
                                }
                            }
                            _ => {
                                new_code.push(IRInstr::BinaryOp(
                                    result.clone(),
                                    left.clone(),
                                    op.clone(),
                                    right.clone(),
                                ));
                                continue;
                            }
                        };
                        new_code.push(IRInstr::Assign(result.clone(), folded.clone()));
                        consts.insert(result.clone(), folded);
                    }

                    (Some(IRValue::Str(a)), Some(IRValue::Str(b))) if op == "+" => {
                        let folded = IRValue::Str(format!("{}{}", a, b));
                        new_code.push(IRInstr::Assign(result.clone(), folded.clone()));
                        consts.insert(result.clone(), folded);
                    }

                    (Some(_), None) | (None, Some(_)) | (None, None) => {
                        new_code.push(IRInstr::BinaryOp(
                            result.clone(),
                            left.clone(),
                            op.clone(),
                            right.clone(),
                        ));
                        consts.remove(result);
                    }

                    //catch-all for Bool, Temp, Var, etc.
                    _ => {
                        new_code.push(IRInstr::BinaryOp(
                            result.clone(),
                            left.clone(),
                            op.clone(),
                            right.clone(),
                        ));
                        consts.remove(result);
                    }

                    (Some(IRValue::Str(a)), Some(IRValue::Str(b))) if op == "+" => {
                        // string concatenation folding
                        let folded = IRValue::Str(format!("{}{}", a, b));
                        new_code.push(IRInstr::Assign(result.clone(), folded.clone()));
                        consts.insert(result.clone(), folded);
                    }

                    (Some(lc), None) | (None, Some(lc)) => {
                        // One side constant, other not. Can't fold fully, but we can push a BinaryOp
                        // If left or right are literals, we could store them into temps earlier, but
                        // leave for other passes.
                        new_code.push(IRInstr::BinaryOp(
                            result.clone(),
                            left.clone(),
                            op.clone(),
                            right.clone(),
                        ));
                        // It's not a constant result
                        consts.remove(result);
                    }

                    (None, None) => {
                        // no folding possible
                        new_code.push(IRInstr::BinaryOp(
                            result.clone(),
                            left.clone(),
                            op.clone(),
                            right.clone(),
                        ));
                        consts.remove(result);
                    }
                }
            }

            IRInstr::Return(name) => {
                // If the returned name maps to a constant, replace return with that constant assigned to a temp
                // or leave as-is if you prefer. Simpler: keep Return(name) unchanged, but we could fold.
                if let Some(cv) = consts.get(name) {
                    // turn into Assign(temp, const); Return(temp)
                    let tmp = format!("t_fold_return_{}", name);
                    new_code.push(IRInstr::Assign(tmp.clone(), cv.clone()));
                    new_code.push(IRInstr::Return(tmp));
                } else {
                    new_code.push(IRInstr::Return(name.clone()));
                }
            }
        }
    }

    new_code
}

// -----------------------------
// Pass: Copy propagation
// -----------------------------
//
// Replace uses of variables/temps that are simple copies of other temps/vars.
// e.g. Assign("d", Temp("t1")) followed by uses of "d" -> replace with "t1".
fn copy_propagation(code: &[IRInstr]) -> Vec<IRInstr> {
    // Build a map of direct copies: name -> source_name
    let mut copy_map: HashMap<String, String> = HashMap::new();

    // First pass: collect direct copy assignments: Assign(a, Temp(t)) or Assign(a, Var(t)) where t is not a literal
    for instr in code {
        if let IRInstr::Assign(target, value) = instr {
            match value {
                IRValue::Temp(src) | IRValue::Var(src) => {
                    // record copy target -> src (overwrite previous if any)
                    copy_map.insert(target.clone(), src.clone());
                }
                _ => {
                    // assignment of literal or non-copy; remove any previous mapping
                    copy_map.remove(target);
                }
            }
        } else {
            // other instr: no target mapping change here
        }
    }

    // Second pass: rewrite instructions replacing targets that map to copies transitively.
    // We must resolve transitively (a -> b, b -> c => a -> c)
    let resolve_copy = |mut name: String, map: &HashMap<String, String>| -> String {
        let mut seen = HashSet::new();
        while let Some(next) = map.get(&name) {
            if !seen.insert(name.clone()) {
                break; // cycle; stop
            }
            name = next.clone();
        }
        name
    };

    let mut new_code = Vec::with_capacity(code.len());
    for instr in code {
        match instr {
            IRInstr::Assign(target, value) => {
                // If value is a name and that name maps to something, resolve it.
                let new_val = match value {
                    IRValue::Temp(t) | IRValue::Var(t) => {
                        let resolved = resolve_copy(t.clone(), &copy_map);
                        // If resolved equals target, keep as original to avoid self-copy.
                        if &resolved == target {
                            value.clone()
                        } else {
                            // produce Var(resolved) — keep using Var/Temp indistinctly in IRValue
                            IRValue::Var(resolved)
                        }
                    }
                    _ => value.clone(),
                };
                new_code.push(IRInstr::Assign(target.clone(), new_val));
            }

            IRInstr::BinaryOp(res, l, op, r) => {
                let new_l = resolve_copy(l.clone(), &copy_map);
                let new_r = resolve_copy(r.clone(), &copy_map);
                new_code.push(IRInstr::BinaryOp(
                    res.clone(),
                    new_l,
                    op.clone(),
                    new_r,
                ));
            }

            IRInstr::Return(name) => {
                let new_name = resolve_copy(name.clone(), &copy_map);
                new_code.push(IRInstr::Return(new_name));
            }
        }
    }

    new_code
}

// -----------------------------
// Pass: Dead Code Elimination (DCE)
// -----------------------------
//
// Remove assignments to temps/vars that are never used later. This is conservative:
// - We don't remove assignments to names used by Return or used as left-hand of BinaryOp.
// - We iterate until no more removals happen.
fn dead_code_elimination(code: &[IRInstr]) -> Vec<IRInstr> {
    let mut code_vec: Vec<IRInstr> = code.to_vec();

    loop {
        // compute usage counts of names (temps/vars)
        let mut uses: HashMap<String, usize> = HashMap::new();
        for instr in &code_vec {
            match instr {
                IRInstr::Assign(_, value) => {
                    match value {
                        IRValue::Var(name) | IRValue::Temp(name) => {
                            *uses.entry(name.clone()).or_default() += 1;
                        }
                        _ => {}
                    }
                }
                IRInstr::BinaryOp(_, l, _, r) => {
                    *uses.entry(l.clone()).or_default() += 1;
                    *uses.entry(r.clone()).or_default() += 1;
                }
                IRInstr::Return(name) => {
                    *uses.entry(name.clone()).or_default() += 1;
                }
            }
        }

        // find instructions that write to a name that has zero uses
        let mut removed_any = false;
        let mut new_code = Vec::with_capacity(code_vec.len());
        for instr in &code_vec {
            match instr {
                IRInstr::Assign(target, _) => {
                    let count = uses.get(target).copied().unwrap_or(0);
                    if count == 0 && is_temporary_name(target) {
                        // remove assignment to unused temporary
                        removed_any = true;
                        continue;
                    } else {
                        // keep assignment (either used or it's a non-temp var)
                        new_code.push(instr.clone());
                    }
                }
                _ => new_code.push(instr.clone()),
            }
        }

        code_vec = new_code;

        if !removed_any {
            break;
        }
    }

    code_vec
}

// Heuristic: treat names that start with 't' followed by digits as temporaries.
// Adjust if your temp naming scheme differs.
fn is_temporary_name(name: &str) -> bool {
    name.starts_with('t') && name[1..].chars().all(|c| c.is_ascii_digit())
}
