// Copyright 2024 Duskphantom Authors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//
// SPDX-License-Identifier: Apache-2.0

use core::fmt;
use std::collections::HashSet;
use std::fmt::{Debug, Formatter};

use super::*;
use super::{block::Block, gen_asm::GenTool};

use crate::config::CONFIG;
use rayon::prelude::*;

#[allow(unused)]
#[derive(Default, Debug, Clone)]
#[non_exhaustive]
pub struct Func {
    name: String,
    args: Vec<String>,
    ret: Option<Reg>,
    reg_gener: Option<RegGenerator>,
    stack_allocator: Option<StackAllocator>,
    // entry block
    entry: Block,
    // basic blocks
    other_bbs: Vec<Block>,
    pub max_callee_regs_stack: u32,
}

impl Func {
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn entry(&self) -> &Block {
        &self.entry
    }
    pub fn entry_mut(&mut self) -> &mut Block {
        &mut self.entry
    }

    pub fn gen_asm(&self) -> String {
        if CONFIG.num_parallel_for_func_gen_asm == 1 {
            println!("num_parallel_for_func_gen_asm == 1");
            let mut bbs_asm = String::with_capacity(256);
            for bb in self.iter_bbs() {
                bbs_asm.push_str(bb.gen_asm().as_str());
                bbs_asm.push('\n');
            }
            return GenTool::gen_func(self.name.as_str(), bbs_asm.as_str());
        }
        let thread_pool = rayon::ThreadPoolBuilder::new()
            .num_threads(CONFIG.num_parallel_for_block_gen_asm)
            .build()
            .unwrap();
        let bbs: Vec<&Block> = self.iter_bbs().collect();
        let bbs_asm = thread_pool.install(|| {
            bbs.par_iter()
                .map(|bb| bb.gen_asm())
                .collect::<Vec<String>>()
                .join("\n")
        });
        GenTool::gen_func(self.name.as_str(), &bbs_asm)
    }
}

impl Func {
    /// create a new function, default return type is void
    pub fn new(name: String, args: Vec<String>, entry: Block) -> Func {
        Func {
            name,
            args,
            ret: None,
            reg_gener: None,
            stack_allocator: None,
            other_bbs: Vec::new(),
            max_callee_regs_stack: 0,
            entry,
        }
    }

    pub fn ret(&self) -> Option<&Reg> {
        self.ret.as_ref()
    }
    pub fn ret_mut(&mut self) -> &mut Option<Reg> {
        &mut self.ret
    }

    pub fn stack_allocator(&self) -> Option<&StackAllocator> {
        self.stack_allocator.as_ref()
    }

    pub fn stack_allocator_mut(&mut self) -> &mut Option<StackAllocator> {
        &mut self.stack_allocator
    }

    pub fn reg_gener(&self) -> Option<&RegGenerator> {
        self.reg_gener.as_ref()
    }

    pub fn reg_gener_mut(&mut self) -> &mut Option<RegGenerator> {
        &mut self.reg_gener
    }

    pub fn push_bb(&mut self, bb: Block) {
        self.other_bbs.push(bb);
    }

    pub fn extend_bbs(&mut self, bbs: Vec<Block>) {
        self.other_bbs.extend(bbs);
    }

    /// check if there is a call instruction in the function
    pub fn is_caller(&self) -> bool {
        for bb in self.iter_bbs() {
            for inst in bb.insts() {
                if let Inst::Call { .. } = inst {
                    return true;
                }
            }
        }
        false
    }

    /// get all virtual regs in the function
    pub fn v_regs(&self) -> HashSet<Reg> {
        self.regs().into_iter().filter(|r| r.is_virtual()).collect()
    }

    pub fn regs(&self) -> HashSet<Reg> {
        let mut regs = HashSet::new();
        for bb in self.iter_bbs() {
            for inst in bb.insts() {
                regs.extend(inst.uses().iter().cloned());
                regs.extend(inst.defs().iter().cloned());
            }
        }
        regs
    }

    /// iter bbs in a special order,in which entry is the first one
    pub fn iter_bbs(&self) -> BBIter {
        let other_bbs: Vec<&Block> = self.other_bbs.iter().collect();
        BBIter {
            entry: &self.entry,
            other_bbs,
            idx: 0,
        }
    }

    /// iter bbs in a special order mutably, in which entry is the first one,
    pub fn iter_bbs_mut(&mut self) -> impl Iterator<Item = &mut Block> {
        let mut bbs = vec![&mut self.entry];
        bbs.extend(self.other_bbs.iter_mut());
        bbs.into_iter()
    }

    pub fn other_bbs(&self) -> &Vec<Block> {
        &self.other_bbs
    }

    pub fn other_bbs_mut(&mut self) -> &mut Vec<Block> {
        &mut self.other_bbs
    }

    // get exit bbs, which are the bbs that end with ret or tail call
    pub fn exit_bbs(&self) -> Vec<&Block> {
        let mut ret = vec![];
        for bb in self.iter_bbs() {
            let insts = bb.insts();
            if let Some(last_inst) = insts.last() {
                match last_inst {
                    Inst::Ret { .. } => {
                        ret.push(bb);
                    }
                    Inst::Tail { .. } => {
                        ret.push(bb);
                    }
                    _ => {}
                }
            }
        }
        ret
    }

    // get all the bbs that end with ret or tail call in mutable way
    pub fn exit_bbs_mut(&mut self) -> impl Iterator<Item = &mut Block> {
        let mut ret = vec![];
        for bb in self.iter_bbs_mut() {
            let insts = bb.insts();
            if let Some(last_inst) = insts.last() {
                match last_inst {
                    Inst::Ret { .. } => {
                        ret.push(bb);
                    }
                    Inst::Tail { .. } => {
                        ret.push(bb);
                    }
                    _ => {}
                }
            }
        }
        ret.into_iter()
    }

    pub fn merge_bb(&mut self, from: &str, to: &str) -> Result<()> {
        fn remove_to(vec: &mut Vec<Block>, to: &str) -> Result<Block> {
            let idx = vec.iter().position(|bb| bb.label() == to);
            if let Some(idx) = idx {
                Ok(vec.remove(idx))
            } else {
                Err(anyhow!(""))
            }
        }
        let to = remove_to(&mut self.other_bbs, to)?;
        let from = if self.entry().label() == from {
            self.entry_mut()
        } else {
            self.iter_bbs_mut()
                .find(|bb| bb.label() == from)
                .ok_or(anyhow!(""))
                .with_context(|| context!())?
        };
        from.merge(to)?;
        Ok(())
    }
}

impl Func {
    pub fn find_bb(&self, label: &str) -> Option<&Block> {
        if self.entry.label() == label {
            return Some(&self.entry);
        }
        self.other_bbs.iter().find(|bb| bb.label() == label)
    }

    /// 化简块结尾的jmp指令,如果第i块的最后一条指令是jmp,且jmp的目标就是第i+1块,则删除jmp指令
    /// 如果有发生化简,返回true
    /// 否则返回false
    pub fn simplify_term(&mut self) -> Result<bool> {
        let mut bbs_mut: Vec<&mut Block> = self.iter_bbs_mut().collect();
        let mut to_simplify = vec![];
        for (idx, bb) in bbs_mut.iter().enumerate() {
            let Some(inst) = bb.insts().last() else {
                continue;
            };
            let Inst::Jmp(jmp) = inst else {
                continue;
            };
            let jmp_to: String = jmp.dst().label().ok_or(anyhow!(""))?.into();
            let Some(next_bb) = bbs_mut.get(idx + 1) else {
                continue;
            };
            if jmp_to != next_bb.label() {
                continue;
            }
            to_simplify.push(idx);
        }
        let if_simplify = !to_simplify.is_empty();
        for idx in to_simplify {
            let bb = bbs_mut.get_mut(idx).ok_or(anyhow!(""))?;
            bb.insts_mut().pop();
        }
        Ok(if_simplify)
    }

    /// 移除空块: 通过将空块与后继块合并的方式
    /// Note: 如果entry块为空,则不会被移除
    /// 如果成功移除至少空块,返回true
    /// 否则返回false
    #[allow(unused)]
    pub fn elim_empty_bb(&mut self) -> Result<bool> {
        let mut if_elim = false;
        loop {
            let pos = self
                .other_bbs
                .iter()
                .enumerate()
                .find(|(_, bb)| bb.insts().is_empty());
            let Some((idx, empty_bb)) = pos else {
                break;
            };
            if idx + 1 >= self.other_bbs.len() {
                break;
            }
            let next_bb = self.other_bbs[idx + 1].label().to_string();
            let from = empty_bb.label().to_string();
            self.other_bbs.remove(idx);

            self.rename_bb_label(&from, &next_bb)?;
            if_elim = true;
        }
        Ok(if_elim)
    }

    pub fn rename_bb_label(&mut self, from: &str, to: &str) -> Result<()> {
        macro_rules! replace_bb_label {
            ($inst:ident,$from:expr,$to:expr) => {{
                if $inst.label() == &$from.into() {
                    *$inst.label_mut() = $to.into();
                }
            }};
        }
        for bb in self.iter_bbs_mut() {
            for inst in bb.insts_mut() {
                match inst {
                    Inst::Beq(beq) => replace_bb_label!(beq, from, to),
                    Inst::Bne(bne) => replace_bb_label!(bne, from, to),
                    Inst::Blt(blt) => replace_bb_label!(blt, from, to),
                    Inst::Ble(ble) => replace_bb_label!(ble, from, to),
                    Inst::Bgt(bgt) => replace_bb_label!(bgt, from, to),
                    Inst::Bge(bge) => replace_bb_label!(bge, from, to),
                    Inst::Jmp(jmp) => {
                        if jmp.dst().label() == Some(from.into()) {
                            *jmp.dst_mut() = to.into();
                        }
                    }
                    _ => {
                        continue;
                    }
                }
            }
        }
        Ok(())
    }

    /// 解除化简块结尾的jmp指令
    /// 如果一个块结尾的term类型指令只有1个,且不是jmp,则在该块的结尾插入jmp指令到它的邻接的下一个块
    #[allow(unused)]
    pub fn desimplify_term(&mut self) -> Result<()> {
        let mut bbs_mut: Vec<&mut Block> = self.iter_bbs_mut().collect();
        let mut to_desimplify = vec![];
        for (idx, bb) in bbs_mut.iter().enumerate() {
            let terms: Vec<&Inst> = bb.insts().iter().filter(|inst| inst.is_term()).collect();
            if terms.len() >= 2 {
                continue;
            } else if let Some(inst) = terms.last() {
                if inst.is_branch() {
                    to_desimplify.push(idx);
                }
            } else {
                // this condition means the terms is empty
                to_desimplify.push(idx);
            }
        }

        for idx in to_desimplify {
            let next_bb_name = bbs_mut.get(idx + 1).ok_or(anyhow!(""))?.label().to_string();
            let bb = bbs_mut.get_mut(idx).ok_or(anyhow!(""))?;
            let jmp = JmpInst::new(next_bb_name.into());
            bb.insts_mut().push(jmp.into());
        }

        Ok(())
    }
}

pub struct BBDistanceCounter {
    num_insts: Vec<(String, usize)>,
}
impl BBDistanceCounter {
    pub fn distance_between(&self, from: &str, to: &str) -> Option<usize> {
        let from_idx = self.num_insts.iter().position(|(label, _)| label == from)?;
        let to_idx = self.num_insts.iter().position(|(label, _)| label == to)?;
        if from_idx < to_idx {
            let distance = self.num_insts[(from_idx + 1)..to_idx]
                .iter()
                .map(|(_, num_inst)| *num_inst)
                .sum();
            Some(distance)
        } else {
            let distance = self.num_insts[to_idx..=from_idx]
                .iter()
                .map(|(_, num_inst)| *num_inst)
                .sum();
            Some(distance)
        }
    }
}
/// helper method for handling long jmp
impl Func {
    pub fn bb_distances(&self) -> BBDistanceCounter {
        let num_insts: Vec<(String, usize)> = self
            .iter_bbs()
            .map(|bb| (bb.label().to_string(), bb.insts().len()))
            .collect();

        BBDistanceCounter { num_insts }
    }

    pub fn add_after(&mut self, bb: &str, add_after: Vec<Block>) -> Result<()> {
        if self.entry.label() == bb {
            self.other_bbs.splice(0..0, add_after);
        } else {
            let idx = self
                .other_bbs
                .iter()
                .position(|b| b.label() == bb)
                .ok_or_else(|| anyhow!("no such bb {}", bb))?;
            self.other_bbs.splice(idx + 1..idx + 1, add_after);
        }
        Ok(())
    }
}

#[derive(Clone)]
pub struct BBIter<'a> {
    entry: &'a Block,
    other_bbs: Vec<&'a Block>,
    idx: usize,
}
impl Debug for BBIter<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("BBIter")
            .field("entry", &self.entry.label())
            .field(
                "other_bbs",
                &self
                    .other_bbs
                    .iter()
                    .map(|bb| bb.label())
                    .collect::<Vec<_>>(),
            )
            .field("idx", &self.idx)
            .finish()
    }
}
impl<'a> Iterator for BBIter<'a> {
    type Item = &'a Block;

    fn next(&mut self) -> Option<Self::Item> {
        if self.idx == 0 {
            self.idx += 1;
            Some(self.entry)
        } else {
            let ret = self.other_bbs.get(self.idx - 1).cloned();
            self.idx += 1;
            ret
        }
    }
}

#[cfg(test)]
mod tests {
    use insta::assert_debug_snapshot;

    use super::*;
    fn new_bb(n: &str) -> Block {
        Block::new(n.to_string())
    }

    #[test]
    fn test_iter_bb_order() {
        use super::*;
        let mut func = Func::default();
        let entry = Block::new("entry".to_string());
        let bb1 = Block::new("bb1".to_string());
        let bb2 = Block::new("bb4".to_string());
        let bb3 = Block::new("bb3".to_string());
        func.entry = entry;
        func.other_bbs.push(bb1);
        func.other_bbs.push(bb2);
        func.other_bbs.push(bb3);
        let mut iter = func.iter_bbs();
        assert_eq!(iter.next().unwrap().label(), "entry");
        assert_eq!(iter.next().unwrap().label(), "bb1");
        assert_eq!(iter.next().unwrap().label(), "bb4");
        assert_eq!(iter.next().unwrap().label(), "bb3");
        assert!(iter.next().is_none());
    }

    #[test]
    fn test_func_new() {
        let func = Func::new("main".to_string(), vec![], Block::new("entry".to_string()));
        assert_eq!(func.name(), "main");
        assert_eq!(func.args.len(), 0);
        assert_eq!(func.entry().label(), "entry");
        assert_eq!(func.ret(), None); // default return type is void
    }

    #[test]
    fn test_add_after() {
        let mut func = Func::new("main".to_string(), vec![], new_bb("entry"));
        func.push_bb(new_bb("bb1"));
        func.push_bb(new_bb("bb2"));

        let new_bbs = vec![new_bb("new1"), new_bb("new2")];
        func.add_after("entry", new_bbs).unwrap();

        let iter = func.iter_bbs();
        assert_debug_snapshot!(iter,@r###"
        BBIter {
            entry: "entry",
            other_bbs: [
                "new1",
                "new2",
                "bb1",
                "bb2",
            ],
            idx: 0,
        }
        "###);
    }

    #[test]
    fn test_add_after2() {
        let mut func = Func::new("main".to_string(), vec![], new_bb("entry"));
        func.push_bb(new_bb("bb1"));
        func.push_bb(new_bb("bb2"));

        let new_bbs = vec![new_bb("new1"), new_bb("new2")];
        func.add_after("bb1", new_bbs).unwrap();

        let iter = func.iter_bbs();
        assert_debug_snapshot!(iter,@r###"
        BBIter {
            entry: "entry",
            other_bbs: [
                "bb1",
                "new1",
                "new2",
                "bb2",
            ],
            idx: 0,
        }
        "###);
    }

    #[test]
    fn test_add_after3() {
        let mut func = Func::new("main".to_string(), vec![], new_bb("entry"));
        func.push_bb(new_bb("bb1"));
        func.push_bb(new_bb("bb2"));
        let new_bbs = vec![new_bb("new1"), new_bb("new2")];
        func.add_after("bb2", new_bbs).unwrap();
        let iter = func.iter_bbs();
        assert_debug_snapshot!(iter,@r###"
        BBIter {
            entry: "entry",
            other_bbs: [
                "bb1",
                "bb2",
                "new1",
                "new2",
            ],
            idx: 0,
        }
        "###);
    }

    #[test]
    fn test_dis_counter() {
        let fill_insts = |bb: &mut Block, num: usize| {
            for _ in 0..num {
                bb.push_inst(MvInst::new(REG_A0.into(), REG_A1.into()).into());
            }
        };
        let mut func = Func::new("main".to_string(), vec![], new_bb("entry"));
        fill_insts(func.entry_mut(), 3);
        let mut bb1 = new_bb("bb1");
        fill_insts(&mut bb1, 2);
        let mut bb2 = new_bb("bb2");
        fill_insts(&mut bb2, 4);
        func.push_bb(bb1);
        func.push_bb(bb2);
        let dis_counter = func.bb_distances();
        assert_eq!(dis_counter.distance_between("entry", "bb1"), Some(0));
        assert_eq!(dis_counter.distance_between("entry", "bb2"), Some(2));
        assert_eq!(dis_counter.distance_between("bb1", "bb2"), Some(0));
        assert_eq!(dis_counter.distance_between("bb2", "bb1"), Some(6));
    }
}
