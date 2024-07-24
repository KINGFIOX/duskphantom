use super::*;

pub trait CheckValidInst {
    fn check_valid(&self) -> bool {
        true
    }
}

pub mod riscv {
    use super::*;
    impl CheckValidInst for Inst {
        fn check_valid(&self) -> bool {
            match self {
                Inst::Add(inst) => inst.check_valid(),
                Inst::Sub(inst) => inst.check_valid(),
                Inst::Mul(inst) => inst.check_valid(),
                Inst::Rem(inst) => inst.check_valid(),
                Inst::Neg(inst) => inst.check_valid(),
                Inst::Div(inst) => inst.check_valid(),
                Inst::Sll(inst) => inst.check_valid(),
                Inst::Srl(inst) => inst.check_valid(),
                Inst::Slt(inst) => inst.check_valid(),
                Inst::Mv(inst) => inst.check_valid(),
                Inst::Ld(inst) => inst.check_valid(),
                Inst::Sd(inst) => inst.check_valid(),
                Inst::Sw(inst) => inst.check_valid(),
                Inst::Lw(inst) => inst.check_valid(),
                Inst::Lla(inst) => inst.check_valid(),
                Inst::Load(inst) => inst.check_valid(),
                Inst::Store(inst) => inst.check_valid(),
                Inst::Jmp(inst) => inst.check_valid(),
                Inst::Beq(inst) => inst.check_valid(),
                Inst::Bne(inst) => inst.check_valid(),
                Inst::Bge(inst) => inst.check_valid(),
                Inst::Blt(inst) => inst.check_valid(),
                Inst::Bgt(inst) => inst.check_valid(),
                Inst::Ble(inst) => inst.check_valid(),

                Inst::F2i(inst) => inst.check_valid(),
                Inst::I2f(inst) => inst.check_valid(),

                Inst::Call(inst) => inst.check_valid(),
                Inst::SRA(inst) => inst.check_valid(),
                Inst::Ret => true,
                Inst::And(inst) => inst.check_valid(),
                Inst::Or(inst) => inst.check_valid(),
                Inst::Xor(inst) => inst.check_valid(),
                Inst::Tail(inst) => inst.check_valid(),
                Inst::Li(inst) => inst.check_valid(),
                Inst::Seqz(inst) => inst.check_valid(),
                Inst::Snez(snez) => snez.check_valid(),
                Inst::Not(not) => not.check_valid(),
            }
        }
    }
    impl CheckValidInst for SubInst {
        fn check_valid(&self) -> bool {
            matches!(self.dst(), Operand::Reg(_))
                && matches!(self.lhs(), Operand::Reg(_))
                && matches!(self.rhs(), Operand::Reg(_))
        }
    }

    impl CheckValidInst for RemInst {
        fn check_valid(&self) -> bool {
            matches!(self.dst(), Operand::Reg(_))
                && matches!(self.lhs(), Operand::Reg(_))
                && matches!(self.rhs(), Operand::Reg(_))
        }
    }
    impl CheckValidInst for DivInst {
        fn check_valid(&self) -> bool {
            matches!(self.dst(), Operand::Reg(_))
                && matches!(self.lhs(), Operand::Reg(_))
                && matches!(self.rhs(), Operand::Reg(_))
        }
    }
    impl CheckValidInst for SllInst {}
    impl CheckValidInst for SrlInst {}
    impl CheckValidInst for SltInst {}
    impl CheckValidInst for MvInst {}
    impl CheckValidInst for LdInst {}
    impl CheckValidInst for SdInst {}
    impl CheckValidInst for SwInst {}
    impl CheckValidInst for LwInst {}
    impl CheckValidInst for LlaInst {}
    impl CheckValidInst for LoadInst {
        /// 在riscv 阶段，不应该存在load指令
        fn check_valid(&self) -> bool {
            false
        }
    }
    impl CheckValidInst for StoreInst {
        /// 在riscv 阶段，不应该存在store指令
        fn check_valid(&self) -> bool {
            false
        }
    }
    impl CheckValidInst for JmpInst {}
    impl CheckValidInst for BeqInst {}
    impl CheckValidInst for BneInst {}
    impl CheckValidInst for BgeInst {}
    impl CheckValidInst for BltInst {}
    impl CheckValidInst for BgtInst {}
    impl CheckValidInst for BleInst {}
    impl CheckValidInst for CallInst {}
    impl CheckValidInst for AndInst {}
    impl CheckValidInst for OrInst {}
    impl CheckValidInst for SraInst {}
    impl CheckValidInst for XorInst {}
    impl CheckValidInst for TailInst {}
    impl CheckValidInst for AddInst {}
    impl CheckValidInst for MulInst {
        fn check_valid(&self) -> bool {
            matches!(self.dst(), Operand::Reg(_))
                && matches!(self.lhs(), Operand::Reg(_))
                && matches!(self.rhs(), Operand::Reg(_))
        }
    }
    impl CheckValidInst for NegInst {}
    impl CheckValidInst for LiInst {
        fn check_valid(&self) -> bool {
            matches!(self.dst(), Operand::Reg(_)) && matches!(self.src(), Operand::Imm(_))
        }
    }
    impl CheckValidInst for SeqzInst {
        fn check_valid(&self) -> bool {
            matches!(self.dst(), Operand::Reg(_)) && matches!(self.src(), Operand::Reg(_))
        }
    }
    impl CheckValidInst for F2iInst {
        fn check_valid(&self) -> bool {
            (match self.dst() {
                Operand::Reg(r) => r.is_usual(),
                _ => false,
            }) && (match self.src() {
                Operand::Reg(r) => r.is_float(),
                _ => false,
            })
        }
    }
    impl CheckValidInst for I2fInst {
        fn check_valid(&self) -> bool {
            (match self.dst() {
                Operand::Reg(r) => r.is_float(),
                _ => false,
            }) && (match self.src() {
                Operand::Reg(r) => r.is_usual(),
                _ => false,
            })
        }
    }
    impl CheckValidInst for SnezInst {
        fn check_valid(&self) -> bool {
            matches!(self.dst(), Operand::Reg(_)) && matches!(self.src(), Operand::Reg(_))
        }
    }
    impl CheckValidInst for NotInst {
        fn check_valid(&self) -> bool {
            matches!(self.dst(), Operand::Reg(_)) && matches!(self.src(), Operand::Reg(_))
        }
    }
}
