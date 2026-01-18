pub mod asm {
    use crate::datatypes::ast_statements::VariableType;

    pub enum StackDestination {
        StackPointer,
        StackFramePointer
    }

    pub enum TempRegisters {
        T0,
        T1,
        T2
    }

    pub fn temp_reg_for_type(var_type : VariableType, load_instruction : bool, reg : TempRegisters) -> String {
        let temp_registers : [&str; 2] = {
            match reg {
                TempRegisters::T0 => ["x10", "w10"],
                TempRegisters::T1 => ["x11", "w11"],
                TempRegisters::T2 => ["x12", "w12"]
            }
        };

        let res : &str = match var_type {
            VariableType::U8 | VariableType::U16 | VariableType::U32 => temp_registers[1],
            VariableType::I64 | VariableType::U64 => temp_registers[0],
            VariableType::I8 | VariableType::I16 | VariableType::I32 => {
                if load_instruction { temp_registers[0] } else { temp_registers[1] }
            }

            _ => unreachable!(),
        };

        return String::from(res);
    }

    pub fn store_literal_to_stack(var_type : VariableType, num : i64, offset : usize) -> String {
        let temp_reg = temp_reg_for_type(var_type.clone(), false, TempRegisters::T0);

        return format!("mov {}, #{}\n{} {}, [sp, #{}]\n", temp_reg, num, store_instruction_for_type(var_type), temp_reg, offset);
    }

    pub fn store_reg_to_stack(reg : &str, offset : usize, var_type : VariableType) -> String {
        return format!("{} {}, [sp, #{}]\n", store_instruction_for_type(var_type), reg, offset);
    }

    pub fn load_instruction_for_type(var_type : VariableType) -> String {
        let res : &str = match var_type {
            VariableType::I8 => "ldrsb",
            VariableType::I16 => "ldrsh",
            VariableType::I32 => "ldrsw",
            VariableType::I64 => "ldr",
            VariableType::U8 => "ldrb",
            VariableType::U16 => "ldrh",
            VariableType::U32 => "ldr",
            VariableType::U64 => "ldr",
            _ => unreachable!()
        };

        return String::from(res);
    }

    pub fn store_instruction_for_type(var_type : VariableType) -> String {
        let res : &str = match var_type {
            VariableType::I8 => "strb",
            VariableType::I16 => "strh",
            VariableType::I32 => "str",
            VariableType::I64 => "str",
            VariableType::U8 => "strb",
            VariableType::U16 => "strh",
            VariableType::U32 => "str",
            VariableType::U64 => "str",
            _ => unreachable!()
        };

        return String::from(res);
    }

    pub fn allocate_stack_memory(bytes : usize) -> String {
        return format!("sub sp, sp, #{}\n", bytes);
    }

    pub fn deallocate_stack_memory(bytes : usize) -> String {
        return format!("add sp, sp, #{}\n", bytes);
    }

    pub fn mov_num_to_reg(reg : &str, num : i64) -> String {
        return format!("mov {}, #{}\n", reg, num);
    }

    pub fn jump_to_function(function_name : &str) -> String {
        return format!("bl _{}\n", function_name);
    }

    pub fn goto(label : &str) -> String {
        return format!("b _{}\n", label);
    }

    pub fn create_scope(stack_memory_allocate : usize) -> String {
        if stack_memory_allocate == 0 {
            return format!("str x30, [sp, #-16]!\n");
        } else {
            return format!("str x30, [sp, #-16]!\nsub sp, sp, #{}\n", stack_memory_allocate);
        }
    }

    pub fn destroy_scope(stack_memory_allocated : usize) -> String {
        if stack_memory_allocated == 0 {
            return format!("ldr x30, [sp], #16\nret\n");
        } else {
            return format!("add sp, sp, #{}\nldr x30, [sp], #16\nret\n", stack_memory_allocated);
        }
    }

    pub fn variable_to_reg(reg : &str, offset : usize, var_type : VariableType) -> String {
        return format!("{} {}, [sp, #{}]\n", load_instruction_for_type(var_type), reg, offset);
    }
}
