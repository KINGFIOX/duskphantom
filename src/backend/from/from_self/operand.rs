use std::collections::HashMap;

use anyhow::{anyhow, Context, Result};

use crate::backend::var::FloatVar;
use crate::backend::{Fmm, Inst, LiInst, LlaInst, LwInst, Operand, Reg, RegGenerator, StackSlot};

use crate::context;

use crate::middle;
use crate::middle::ir::Instruction;
use crate::utils::mem::ObjPtr;

use super::*;
use builder::IRBuilder;

impl IRBuilder {
    /// 用于生成虚拟寄存器
    pub fn new_var(ty: &middle::ir::ValueType, reg_gener: &mut RegGenerator) -> Result<Reg> {
        let dst_reg = match ty {
            middle::ir::ValueType::Int
            | middle::ir::ValueType::Bool
            | middle::ir::ValueType::Pointer(_) => reg_gener.gen_virtual_usual_reg(),
            middle::ir::ValueType::Float => reg_gener.gen_virtual_float_reg(),
            _ => return Err(anyhow!("phi can't be void/array")).with_context(|| context!()),
        };
        Ok(dst_reg)
    }

    pub fn stack_slot_from(
        operand: &middle::ir::Operand,
        stack_slots: &HashMap<Address, StackSlot>,
    ) -> Result<Operand> {
        Ok(match operand {
            middle::ir::Operand::Instruction(instr) => stack_slots
                .get(&(instr.as_ref().as_ref() as *const dyn Instruction as *const () as Address))
                .ok_or(anyhow!(
                    "stack slot not found {}",
                    (instr.as_ref().as_ref() as *const dyn Instruction as *const () as Address)
                ))
                .with_context(|| context!())?
                .into(), // 这个 into 将 stackslot -> operand
            _ => {
                /* Constant, Global */
                /* Parameter 只有 void, int, float 三种类型 */
                return Err(anyhow!("operand is not local var:{}", operand))
                    .with_context(|| context!());
            }
        })
    }

    /// 这个不包含有 arr
    #[inline]
    pub fn const_except_arr_from(con: &middle::ir::Constant) -> Result<Operand> {
        Ok(match con {
            middle::ir::Constant::Int(val) => Operand::Imm((*val as i64).into()),
            middle::ir::Constant::Float(fla) => Operand::Fmm((*fla as f64).into()),
            middle::ir::Constant::Bool(boo) => Operand::Imm((*boo as i64).into()),
            middle::ir::Constant::SignedChar(sig) => Operand::Imm((*sig as i64).into()),
            middle::ir::Constant::Array(_) => {
                return Err(anyhow!("const_from operand cann't not be array:{}", con))
                    .with_context(|| context!())
            }
        })
    }

    /// 这里不包含有 函数的形参。local_var_from 返回 Reg
    #[inline]
    pub fn local_var_except_param_from(
        instr: &ObjPtr<Box<dyn Instruction>>,
        regs: &HashMap<Address, Reg>,
    ) -> Result<Reg> {
        let addr = instr.as_ref().as_ref() as *const dyn Instruction as *const () as Address;
        let reg = regs
            .get(&addr)
            .ok_or(anyhow!("local var not found {}", addr))
            .with_context(|| context!())?;
        Ok(*reg)
    }

    /// 因为 build_entry 的时候, 就已经把参数 mv <虚拟寄存器>, <param> 了
    #[inline]
    pub fn param_from(param: &middle::ir::Parameter, regs: &HashMap<Address, Reg>) -> Result<Reg> {
        let addr = param as *const _ as Address;
        let reg = regs
            .get(&addr)
            .ok_or(anyhow!(
                "local var not found {}",
                param as *const _ as Address
            ))
            .with_context(|| context!())?;
        Ok(*reg)
    }

    /// 获取 basic block 的 label
    #[inline]
    pub fn label_name_from(bb: &ObjPtr<middle::ir::BasicBlock>) -> String {
        format!(".LBB{}", bb.as_ref() as *const _ as Address)
    }

    /// 需要注意的是 指令的 lvalue 只能是寄存器,所以如果value是个常数,则需要用一个寄存器来存储,并且需要生成一条指令
    /// so this function promise that the return value is a (reg,pre_insts) tuple
    /// pre_insts is the insts that generate the reg,which should be inserted before the insts that use the reg
    pub fn prepare_rs1_i(
        value: &middle::ir::Operand,
        reg_gener: &mut RegGenerator,
        regs: &HashMap<Address, Reg>,
    ) -> Result<(Operand, Vec<Inst>)> {
        let mut insts = Vec::new();
        let value = IRBuilder::no_load_from(value, regs)?;
        match &value {
            Operand::Imm(imm) => {
                let dst = reg_gener.gen_virtual_usual_reg();
                let li = LiInst::new(dst.into(), imm.into());
                insts.push(li.into());
                Ok((dst.into(), insts))
            }
            Operand::Reg(_) => Ok((value, insts)),
            _ => unimplemented!(),
        }
    }

    /// 如果value是个寄存器,直接返回,
    /// 如果是个常数,如果超出范围,则需要用一个寄存器来存储,并且需要生成一条指令
    /// 如果是不超出范围的常数,则直接返回
    pub fn prepare_rs2_i(
        value: &middle::ir::Operand,
        reg_gener: &mut RegGenerator,
        regs: &HashMap<Address, Reg>,
    ) -> Result<(Operand, Vec<Inst>)> {
        let mut insts: Vec<Inst> = Vec::new();
        let value = IRBuilder::no_load_from(value, regs)?;
        match &value {
            Operand::Imm(imm) => {
                if imm.in_limit(12) {
                    Ok((value, insts))
                } else {
                    let dst = reg_gener.gen_virtual_usual_reg();
                    let li = LiInst::new(dst.into(), imm.into());
                    insts.push(li.into());
                    Ok((dst.into(), insts))
                }
            }
            Operand::Reg(_) => Ok((value, insts)),
            _ => unimplemented!(),
        }
    }

    #[inline]
    pub fn prepare_f(
        value: &middle::ir::Operand,
        reg_gener: &mut RegGenerator,
        regs: &HashMap<Address, Reg>,
        fmms: &mut HashMap<Fmm, FloatVar>,
    ) -> Result<(Operand, Vec<Inst>)> {
        let mut insts: Vec<Inst> = Vec::new();
        let value = IRBuilder::no_load_from(value, regs)?;
        match value {
            Operand::Reg(_) => Ok((value, insts)),
            Operand::Fmm(fmm) => {
                let lit = if let Some(f_var) = fmms.get(&fmm) {
                    f_var.name.clone()
                } else {
                    let name = Self::fmm_lit_label_from(&fmm);
                    fmms.insert(
                        fmm.clone(),
                        FloatVar {
                            name: name.clone(),
                            init: Some(fmm.clone().try_into()?),
                            is_const: true,
                        },
                    );
                    name
                };
                let addr = reg_gener.gen_virtual_usual_reg();
                let lla = LlaInst::new(addr, lit.into());
                insts.push(lla.into());
                let dst = reg_gener.gen_virtual_float_reg();
                let loadf = LwInst::new(dst, 0.into(), addr);
                insts.push(loadf.into());
                Ok((dst.into(), insts))
            }
            _ => unimplemented!(),
        }
    }

    #[inline]
    pub fn fmm_lit_label_from(fmm: &Fmm) -> String {
        format!("_fc_{:x}", fmm.to_bits())
    }

    #[inline]
    pub fn prepare_cond(
        cond: &middle::ir::Operand,
        regs: &HashMap<Address, Reg>,
    ) -> Result<(Reg, Vec<Inst>)> {
        match cond {
            middle::ir::Operand::Constant(_) => todo!(),
            middle::ir::Operand::Global(_) => todo!(),
            middle::ir::Operand::Parameter(_) => todo!(),
            middle::ir::Operand::Instruction(instr) => {
                let ope = Self::local_var_except_param_from(instr, regs)?;
                Ok((ope, Vec::new()))
            }
        }
    }

    /// no_load_from 的特点就是, 可以直接作为 operand, 不需要经过一次 load
    pub fn no_load_from(
        operand: &middle::ir::Operand,
        regs: &HashMap<Address, Reg>,
    ) -> Result<Operand> {
        match operand {
            middle::ir::Operand::Constant(con) => Self::const_except_arr_from(con),
            middle::ir::Operand::Parameter(param) => {
                let param = Self::param_from(param, regs).with_context(|| context!())?;
                Ok(param.into())
            } // 参数实际上都是 Reg
            middle::ir::Operand::Instruction(instr) => {
                let reg =
                    Self::local_var_except_param_from(instr, regs).with_context(|| context!())?;
                Ok(reg.into())
            }
            middle::ir::Operand::Global(glo) => Err(anyhow!(
                "local_operand_from operand cann't not be global:{}",
                glo
            ))
            .with_context(|| context!()),
        }
    }

    #[inline]
    pub fn pointer_from(
        operand: &middle::ir::Operand,
        regs: &mut HashMap<Address, Reg>,
    ) -> Result<Reg> {
        match operand {
            middle::ir::Operand::Parameter(param) => {
                let param = param.as_ref();
                match param.value_type {
                    middle::ir::ValueType::Array(_, _) | middle::ir::ValueType::Pointer(_) => {
                        Self::param_from(param, regs)
                    }
                    _ => Err(anyhow!(
                        "it is impossible to load from a void/bool/int/float paramter: {}",
                        operand
                    ))
                    .with_context(|| context!()),
                }
            }
            middle::ir::Operand::Instruction(instr) => {
                Self::local_var_except_param_from(instr, regs)
            }
            middle::ir::Operand::Constant(_) => Err(anyhow!(
                "it is impossible to load from a constant: {}",
                operand
            ))
            .with_context(|| context!()),
            middle::ir::Operand::Global(_) => Err(anyhow!(
                "global have been processed in global_from: {}",
                operand
            ))
            .with_context(|| context!()),
        }
    }

    /// 我们的 global/函数名 都是来自于中端的 name 的, 其他的 id 来自于中端的地址
    #[inline]
    pub fn global_from(operand: &middle::ir::Operand) -> Result<String> {
        // TODO
        match operand {
            middle::ir::Operand::Global(glo) => {
                let glo = glo.as_ref();
                let label = glo.name.clone();
                Ok(label)
            }
            middle::ir::Operand::Constant(con) => {
                match con {
                    middle::ir::Constant::Array(_) => unimplemented!(), // 这个不太可能
                    _ => Err(anyhow!("not a global var:{}", operand)).with_context(|| context!()), /* SignedChar(_) | Bool(_) | Float(_) | Int(_) */
                }
            }
            _ => Err(anyhow!("not a global var:{}", operand)).with_context(|| context!()), // Instruction(_) | Parameter(_)
        }
    }

    /// int arr[][5][6] ==> [ 1, 6, 5 ]
    fn _cal_dims_rev(ty: &middle::ir::ValueType) -> Vec<usize> {
        match ty {
            middle::ir::ValueType::Void => todo!(),
            middle::ir::ValueType::Int
            | middle::ir::ValueType::SignedChar
            | middle::ir::ValueType::Float
            | middle::ir::ValueType::Bool => {
                vec![1]
            }
            middle::ir::ValueType::Pointer(_) => todo!(),
            middle::ir::ValueType::Array(ty, sz) => {
                let mut ret = Self::_cal_dims_rev(ty);
                ret.push(*sz);
                ret
            }
        }
    }

    #[allow(unused)]
    pub fn _cal_offset(
        ptr: &middle::ir::Operand,
        idxes: &[middle::ir::Operand],
        reg_gener: &mut RegGenerator,
        regs: &mut HashMap<Address, Reg>,
    ) -> Result<(Operand, Vec<Inst>)> {
        match ptr {
            middle::ir::Operand::Constant(_) => unimplemented!(),
            middle::ir::Operand::Global(glo) => unimplemented!(),
            middle::ir::Operand::Parameter(param) => match param.value_type {
                middle::ir::ValueType::Array(_, _) => todo!(),
                middle::ir::ValueType::Pointer(_) => todo!(),
                _ => Err(anyhow!("can't gep from a non arr/pointer: {:?}", param))
                    .with_context(|| context!()),
            },
            middle::ir::Operand::Instruction(instr) => match instr.get_type() {
                middle::ir::instruction::InstType::Alloca => todo!(),
                _ => todo!(),
            },
        }
    }
}
