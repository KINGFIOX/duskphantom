use super::*;

/// 该检查器用于检查是否每个块的几位的1-2条指令为term类型指令，且块内无其他term类型指令
/// 这要求块必须结尾是 {b,j},{ret},{j} 这三种指令组合中的一种
pub struct TightTerm;

impl IRChecker for TightTerm {}
impl ProgramChecker for TightTerm {}
impl ModuleChecker for TightTerm {}
impl FuncChecker for TightTerm {}

impl VarChecker for TightTerm {
    #[allow(unused_variables)]
    fn check_var(&self, var: &Var) -> bool {
        true
    }
}

impl BBChecker for TightTerm {
    fn check_bb(&self, bb: &Block) -> bool {
        let insts = bb.insts();
        let terms: Vec<(usize, &Inst)> = insts
            .iter()
            .enumerate()
            .filter(|(_, inst)| inst.is_term())
            .collect();
        if terms.len() > 2 || terms.is_empty() {
            return false;
        }
        // 最后一条指令是ret/jmp
        if terms.len() == 1 {
            if let Some((last, inst)) = terms.last() {
                if (*last == insts.len() - 1 && matches!(inst, Inst::Ret))
                    || matches!(inst, Inst::Jmp(_))
                {
                    return true;
                }
            }
            return false;
        }

        if let Some((last, inst)) = terms.last() {
            if *last != insts.len() - 1 || !matches!(inst, Inst::Jmp(_)) {
                return false;
            }
        } else {
            unreachable!();
        }

        if let Some((sec, inst)) = terms.get(terms.len() - 2) {
            *sec == insts.len() - 2 && inst.is_branch()
        } else {
            unreachable!();
        }
    }
}
impl InstChecker for TightTerm {
    #[allow(unused)]
    fn check_inst(&self, inst: &Inst) -> bool {
        true
    }
}
