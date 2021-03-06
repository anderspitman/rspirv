// Copyright 2017 Google Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//      http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use mr;

use utils::num::{bytes_to_u32_le, f32_to_u32};

/// Trait for assembling functionalities.
pub trait Assemble {
    /// Assembles the current object and returns the binary code.
    fn assemble(&self) -> Vec<u32>;
}

impl Assemble for mr::ModuleHeader {
    fn assemble(&self) -> Vec<u32> {
        vec![self.magic_number, self.version, self.generator, self.bound, self.reserved_word]
    }
}

fn assemble_str(s: &str) -> Vec<u32> {
    let bytes = s.as_bytes();
    let len = (bytes.len() + 3) >> 2;
    let mut words: Vec<u32> = (0..len).map(|i| bytes_to_u32_le(&bytes[(i << 2)..])).collect();
    if bytes.len() % 4 == 0 {
        words.push(0)
    }
    words
}

impl Assemble for mr::Operand {
    fn assemble(&self) -> Vec<u32> {
        match *self {
            mr::Operand::ImageOperands(v) => vec![v.bits()],
            mr::Operand::FPFastMathMode(v) => vec![v.bits()],
            mr::Operand::SelectionControl(v) => vec![v.bits()],
            mr::Operand::LoopControl(v) => vec![v.bits()],
            mr::Operand::FunctionControl(v) => vec![v.bits()],
            mr::Operand::MemorySemantics(v) => vec![v.bits()],
            mr::Operand::MemoryAccess(v) => vec![v.bits()],
            mr::Operand::KernelProfilingInfo(v) => vec![v.bits()],
            mr::Operand::SourceLanguage(v) => vec![v as u32],
            mr::Operand::ExecutionModel(v) => vec![v as u32],
            mr::Operand::AddressingModel(v) => vec![v as u32],
            mr::Operand::MemoryModel(v) => vec![v as u32],
            mr::Operand::ExecutionMode(v) => vec![v as u32],
            mr::Operand::StorageClass(v) => vec![v as u32],
            mr::Operand::Dim(v) => vec![v as u32],
            mr::Operand::SamplerAddressingMode(v) => vec![v as u32],
            mr::Operand::SamplerFilterMode(v) => vec![v as u32],
            mr::Operand::ImageFormat(v) => vec![v as u32],
            mr::Operand::ImageChannelOrder(v) => vec![v as u32],
            mr::Operand::ImageChannelDataType(v) => vec![v as u32],
            mr::Operand::FPRoundingMode(v) => vec![v as u32],
            mr::Operand::LinkageType(v) => vec![v as u32],
            mr::Operand::AccessQualifier(v) => vec![v as u32],
            mr::Operand::FunctionParameterAttribute(v) => vec![v as u32],
            mr::Operand::Decoration(v) => vec![v as u32],
            mr::Operand::BuiltIn(v) => vec![v as u32],
            mr::Operand::Scope(v) => vec![v as u32],
            mr::Operand::GroupOperation(v) => vec![v as u32],
            mr::Operand::KernelEnqueueFlags(v) => vec![v as u32],
            mr::Operand::Capability(v) => vec![v as u32],
            mr::Operand::IdMemorySemantics(v) |
            mr::Operand::IdScope(v) |
            mr::Operand::IdRef(v) |
            mr::Operand::LiteralInt32(v) |
            mr::Operand::LiteralExtInstInteger(v) => vec![v],
            mr::Operand::LiteralInt64(_) => unimplemented!(),
            mr::Operand::LiteralFloat32(v) => vec![f32_to_u32(v)],
            mr::Operand::LiteralFloat64(_) => unimplemented!(),
            mr::Operand::LiteralSpecConstantOpInteger(v) => vec![v as u32],
            mr::Operand::LiteralString(ref v) => assemble_str(v),
        }
    }
}

impl Assemble for mr::Instruction {
    fn assemble(&self) -> Vec<u32> {
        let mut code = vec![self.class.opcode as u32];
        if let Some(r) = self.result_type {
            code.push(r);
        }
        if let Some(r) = self.result_id {
            code.push(r);
        }
        for operand in &self.operands {
            code.append(&mut operand.assemble());
        }
        code[0] |= (code.len() as u32) << 16;
        code
    }
}

impl Assemble for mr::BasicBlock {
    fn assemble(&self) -> Vec<u32> {
        let mut code = vec![];
        if let Some(ref l) = self.label {
            code.append(&mut l.assemble());
        }
        for inst in &self.instructions {
            code.append(&mut inst.assemble());
        }
        code
    }
}

impl Assemble for mr::Function {
    fn assemble(&self) -> Vec<u32> {
        let mut code = vec![];
        if let Some(ref d) = self.def {
            code.append(&mut d.assemble());
        }
        for bb in &self.basic_blocks {
            code.append(&mut bb.assemble());
        }
        if let Some(ref e) = self.end {
            code.append(&mut e.assemble());
        }
        code
    }
}

impl Assemble for mr::Module {
    fn assemble(&self) -> Vec<u32> {
        let mut code = match self.header {
            Some(ref h) => h.assemble(),
            None => vec![],
        };
        for inst in self.global_inst_iter() {
            code.append(&mut inst.assemble());
        }
        for f in &self.functions {
            code.append(&mut f.assemble());
        }
        code
    }
}

#[cfg(test)]
mod tests {
    use mr;
    use spirv;

    use binary::Assemble;
    use super::{assemble_str, bytes_to_u32_le};

    #[test]
    fn test_assemble_str() {
        assert_eq!(vec![0u32], assemble_str(""));
        assert_eq!(vec![bytes_to_u32_le(b"h\0\0\0")], assemble_str("h"));
        assert_eq!(vec![bytes_to_u32_le(b"hell"), 0u32], assemble_str("hell"));
        assert_eq!(vec![bytes_to_u32_le(b"hell"), bytes_to_u32_le(b"o\0\0\0")],
                   assemble_str("hello"));
    }

    #[test]
    fn test_assemble_operand_bitmask() {
        let v = spirv::FUNCTION_CONTROL_DONT_INLINE;
        assert_eq!(vec![v.bits()], mr::Operand::FunctionControl(v).assemble());
        let v = spirv::FUNCTION_CONTROL_PURE;
        assert_eq!(vec![v.bits()], mr::Operand::FunctionControl(v).assemble());
        let v = spirv::FUNCTION_CONTROL_CONST;
        assert_eq!(vec![v.bits()], mr::Operand::FunctionControl(v).assemble());
        let v = spirv::FUNCTION_CONTROL_DONT_INLINE | spirv::FUNCTION_CONTROL_CONST;
        assert_eq!(vec![v.bits()], mr::Operand::FunctionControl(v).assemble());
        let v = spirv::FUNCTION_CONTROL_DONT_INLINE | spirv::FUNCTION_CONTROL_PURE |
                spirv::FUNCTION_CONTROL_CONST;
        assert_eq!(vec![v.bits()], mr::Operand::FunctionControl(v).assemble());
    }

    #[test]
    fn test_assemble_operand_enum() {
        assert_eq!(vec![spirv::BuiltIn::Position as u32],
                   mr::Operand::BuiltIn(spirv::BuiltIn::Position).assemble());
        assert_eq!(vec![spirv::BuiltIn::PointSize as u32],
                   mr::Operand::BuiltIn(spirv::BuiltIn::PointSize).assemble());
        assert_eq!(vec![spirv::BuiltIn::InstanceId as u32],
                   mr::Operand::BuiltIn(spirv::BuiltIn::InstanceId).assemble());
    }

    fn wc_op(wc: u32, op: spirv::Op) -> u32 {
        (wc << 16) | op as u32
    }

    // No operands
    #[test]
    fn test_assemble_inst_nop() {
        assert_eq!(vec![wc_op(1, spirv::Op::Nop)],
                   mr::Instruction::new(spirv::Op::Nop, None, None, vec![]).assemble());
    }

    // No result type and result id
    #[test]
    fn test_assemble_inst_memory_model() {
        let operands = vec![mr::Operand::AddressingModel(spirv::AddressingModel::Physical32),
                            mr::Operand::MemoryModel(spirv::MemoryModel::OpenCL)];
        assert_eq!(vec![wc_op(3, spirv::Op::MemoryModel),
                        spirv::AddressingModel::Physical32 as u32,
                        spirv::MemoryModel::OpenCL as u32],
                   mr::Instruction::new(spirv::Op::MemoryModel, None, None, operands).assemble());
    }

    // No result type, having result id
    #[test]
    fn test_assemble_inst_type_int() {
        let operands = vec![mr::Operand::LiteralInt32(32), mr::Operand::LiteralInt32(1)];
        assert_eq!(vec![wc_op(4, spirv::Op::TypeInt), 42, 32, 1],
                   mr::Instruction::new(spirv::Op::TypeInt, None, Some(42), operands).assemble());
    }

    // Having result type and id
    #[test]
    fn test_assemble_inst_iadd() {
        let operands = vec![mr::Operand::IdRef(0xef), mr::Operand::IdRef(0x78)];
        assert_eq!(vec![wc_op(5, spirv::Op::IAdd), 0xab, 0xcd, 0xef, 0x78],
                   mr::Instruction::new(spirv::Op::IAdd, Some(0xab), Some(0xcd), operands)
                       .assemble());
    }

    #[test]
    fn test_assemble_function() {
        let mut b = mr::Builder::new();
        b.memory_model(spirv::AddressingModel::Logical, spirv::MemoryModel::Simple);
        let void = b.type_void();
        let voidfvoid = b.type_function(void, vec![void]);
        b.begin_function(void, None, spirv::FUNCTION_CONTROL_CONST, voidfvoid).unwrap();
        b.begin_basic_block(None).unwrap();
        b.ret().unwrap();
        b.end_function().unwrap();

        assert_eq!(vec![0x07230203,
                        (spirv::MAJOR_VERSION << 16) | (spirv::MINOR_VERSION << 8),
                        0xffffffff,
                        5,
                        0,
                        wc_op(3, spirv::Op::MemoryModel),
                        spirv::AddressingModel::Logical as u32,
                        spirv::MemoryModel::Simple as u32,
                        wc_op(2, spirv::Op::TypeVoid),
                        1,
                        wc_op(4, spirv::Op::TypeFunction),
                        2,
                        1,
                        1,
                        wc_op(5, spirv::Op::Function),
                        1,
                        3,
                        spirv::FUNCTION_CONTROL_CONST.bits(),
                        2,
                        wc_op(2, spirv::Op::Label),
                        4,
                        wc_op(1, spirv::Op::Return),
                        wc_op(1, spirv::Op::FunctionEnd)],
                   b.module().assemble());
    }
}
