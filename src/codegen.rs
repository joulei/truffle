use std::{any::Any, collections::HashMap};

use crate::{
    delta::EngineDelta,
    parser::{AstNode, NodeId},
    typechecker::{
        FnRecord, Function, FunctionId, TypeChecker, TypeId, BOOL_TYPE, I64_TYPE, VOID_TYPE,
    },
    F64_TYPE,
};

#[derive(Clone, Copy, Debug)]
pub struct InstructionId(usize);

#[derive(Clone, Copy, Debug)]
pub struct RegisterId(usize);

#[derive(Debug, PartialEq)]
pub struct Value {
    ty: TypeId,
    val: i64,
}

impl Value {
    // pub fn new_i64(ty: ValueType, val: i64) -> Value {
    //     Value { ty, val }
    // }

    // pub fn unknown() -> Value {
    //     Value {
    //         ty: ValueType::Unknown,
    //         val: 0,
    //     }
    // }
}

#[derive(Debug)]
pub enum Instruction {
    IADD {
        lhs: RegisterId,
        rhs: RegisterId,
        target: RegisterId,
    },
    ISUB {
        lhs: RegisterId,
        rhs: RegisterId,
        target: RegisterId,
    },
    IMUL {
        lhs: RegisterId,
        rhs: RegisterId,
        target: RegisterId,
    },
    IDIV {
        lhs: RegisterId,
        rhs: RegisterId,
        target: RegisterId,
    },

    // Integer comparisons (e.g., ILT = Integer + LessThan)
    ILT {
        lhs: RegisterId,
        rhs: RegisterId,
        target: RegisterId,
    },
    ILTE {
        lhs: RegisterId,
        rhs: RegisterId,
        target: RegisterId,
    },
    IGT {
        lhs: RegisterId,
        rhs: RegisterId,
        target: RegisterId,
    },
    IGTE {
        lhs: RegisterId,
        rhs: RegisterId,
        target: RegisterId,
    },

    // float math
    FADD {
        lhs: RegisterId,
        rhs: RegisterId,
        target: RegisterId,
    },
    FSUB {
        lhs: RegisterId,
        rhs: RegisterId,
        target: RegisterId,
    },
    FMUL {
        lhs: RegisterId,
        rhs: RegisterId,
        target: RegisterId,
    },
    FDIV {
        lhs: RegisterId,
        rhs: RegisterId,
        target: RegisterId,
    },

    // float comparisons (e.g., ILT = Integer + LessThan)
    FLT {
        lhs: RegisterId,
        rhs: RegisterId,
        target: RegisterId,
    },
    FLTE {
        lhs: RegisterId,
        rhs: RegisterId,
        target: RegisterId,
    },
    FGT {
        lhs: RegisterId,
        rhs: RegisterId,
        target: RegisterId,
    },
    FGTE {
        lhs: RegisterId,
        rhs: RegisterId,
        target: RegisterId,
    },

    MOV {
        target: RegisterId,
        source: RegisterId,
    },

    BRIF {
        condition: RegisterId,
        then_branch: InstructionId,
        else_branch: InstructionId,
    },

    JMP(InstructionId),

    EXTERNALCALL {
        head: FunctionId,
        args: Vec<RegisterId>,
        target: RegisterId,
    },
}

pub struct FunctionCodegen {
    pub instructions: Vec<Instruction>,
    pub register_values: Vec<i64>,
    pub register_types: Vec<TypeId>,
}

impl FunctionCodegen {
    pub fn new_register_with_value(&mut self, value: i64, value_type: TypeId) -> RegisterId {
        self.register_values.push(value);
        self.register_types.push(value_type);

        RegisterId(self.register_values.len() - 1)
    }

    pub fn new_register(&mut self, ty: TypeId) -> RegisterId {
        self.new_register_with_value(0, ty)
    }

    pub fn i64_const(&mut self, value: i64) -> RegisterId {
        self.new_register_with_value(value, I64_TYPE)
    }

    pub fn f64_const(&mut self, value: f64) -> RegisterId {
        self.new_register_with_value(unsafe { std::mem::transmute::<f64, i64>(value) }, F64_TYPE)
    }

    pub fn bool_const(&mut self, value: bool) -> RegisterId {
        if value {
            self.new_register_with_value(1, BOOL_TYPE)
        } else {
            self.new_register_with_value(0, BOOL_TYPE)
        }
    }

    pub fn add(&mut self, lhs: RegisterId, rhs: RegisterId) -> RegisterId {
        if self.register_types[lhs.0] == F64_TYPE {
            let target = self.new_register(F64_TYPE);

            self.instructions
                .push(Instruction::FADD { lhs, rhs, target });

            target
        } else if self.register_types[lhs.0] == I64_TYPE {
            let target = self.new_register(I64_TYPE);

            self.instructions
                .push(Instruction::IADD { lhs, rhs, target });

            target
        } else {
            panic!("unsupport add operation")
        }
    }

    pub fn sub(&mut self, lhs: RegisterId, rhs: RegisterId) -> RegisterId {
        if self.register_types[lhs.0] == F64_TYPE {
            let target = self.new_register(F64_TYPE);

            self.instructions
                .push(Instruction::FSUB { lhs, rhs, target });

            target
        } else if self.register_types[lhs.0] == I64_TYPE {
            let target = self.new_register(I64_TYPE);

            self.instructions
                .push(Instruction::ISUB { lhs, rhs, target });

            target
        } else {
            panic!("unsupport sub operation")
        }
    }

    pub fn mul(&mut self, lhs: RegisterId, rhs: RegisterId) -> RegisterId {
        if self.register_types[lhs.0] == F64_TYPE {
            let target = self.new_register(F64_TYPE);

            self.instructions
                .push(Instruction::FMUL { lhs, rhs, target });

            target
        } else if self.register_types[lhs.0] == I64_TYPE {
            let target = self.new_register(I64_TYPE);

            self.instructions
                .push(Instruction::IMUL { lhs, rhs, target });

            target
        } else {
            panic!("unsupport mul operation")
        }
    }

    pub fn div(&mut self, lhs: RegisterId, rhs: RegisterId) -> RegisterId {
        if self.register_types[lhs.0] == F64_TYPE {
            let target = self.new_register(F64_TYPE);

            self.instructions
                .push(Instruction::FDIV { lhs, rhs, target });

            target
        } else if self.register_types[lhs.0] == I64_TYPE {
            let target = self.new_register(I64_TYPE);

            self.instructions
                .push(Instruction::IDIV { lhs, rhs, target });

            target
        } else {
            panic!("unsupport div operation")
        }
    }

    pub fn lt(&mut self, lhs: RegisterId, rhs: RegisterId) -> RegisterId {
        let target = self.new_register(BOOL_TYPE);

        if self.register_types[lhs.0] == F64_TYPE {
            self.instructions
                .push(Instruction::FLT { lhs, rhs, target });

            target
        } else if self.register_types[lhs.0] == I64_TYPE {
            self.instructions
                .push(Instruction::ILT { lhs, rhs, target });

            target
        } else {
            panic!("unsupport lt operation")
        }
    }

    pub fn lte(&mut self, lhs: RegisterId, rhs: RegisterId) -> RegisterId {
        let target = self.new_register(BOOL_TYPE);

        if self.register_types[lhs.0] == F64_TYPE {
            self.instructions
                .push(Instruction::FLTE { lhs, rhs, target });

            target
        } else if self.register_types[lhs.0] == I64_TYPE {
            self.instructions
                .push(Instruction::ILTE { lhs, rhs, target });

            target
        } else {
            panic!("unsupport lte operation")
        }
    }

    pub fn gt(&mut self, lhs: RegisterId, rhs: RegisterId) -> RegisterId {
        let target = self.new_register(BOOL_TYPE);

        if self.register_types[lhs.0] == F64_TYPE {
            self.instructions
                .push(Instruction::FGT { lhs, rhs, target });

            target
        } else if self.register_types[lhs.0] == I64_TYPE {
            self.instructions
                .push(Instruction::IGT { lhs, rhs, target });

            target
        } else {
            panic!("unsupport gt operation")
        }
    }

    pub fn gte(&mut self, lhs: RegisterId, rhs: RegisterId) -> RegisterId {
        let target = self.new_register(BOOL_TYPE);

        if self.register_types[lhs.0] == F64_TYPE {
            self.instructions
                .push(Instruction::FGTE { lhs, rhs, target });

            target
        } else if self.register_types[lhs.0] == I64_TYPE {
            self.instructions
                .push(Instruction::IGTE { lhs, rhs, target });

            target
        } else {
            panic!("unsupport gte operation")
        }
    }

    pub fn mov(&mut self, target: RegisterId, source: RegisterId) -> RegisterId {
        if target.0 == source.0 {
            source
        } else {
            self.instructions.push(Instruction::MOV { target, source });

            target
        }
    }

    pub fn brif(
        &mut self,
        target: RegisterId,
        condition: RegisterId,
        then_branch: InstructionId,
        else_branch: InstructionId,
    ) -> RegisterId {
        self.instructions.push(Instruction::BRIF {
            condition,
            then_branch,
            else_branch,
        });

        target
    }

    pub fn jmp(&mut self, location: InstructionId) {
        self.instructions.push(Instruction::JMP(location))
    }

    pub fn external_call(&mut self, head: FunctionId, args: Vec<RegisterId>, target: RegisterId) {
        self.instructions
            .push(Instruction::EXTERNALCALL { head, args, target })
    }

    pub fn eval(&mut self, functions: &[FnRecord]) -> (i64, TypeId) {
        let mut instruction_pointer = 0;
        let length = self.instructions.len();

        while instruction_pointer < length {
            match &self.instructions[instruction_pointer] {
                Instruction::IADD { lhs, rhs, target } => {
                    self.register_values[target.0] =
                        self.register_values[lhs.0] + self.register_values[rhs.0];

                    instruction_pointer += 1;
                }
                Instruction::ISUB { lhs, rhs, target } => {
                    self.register_values[target.0] =
                        self.register_values[lhs.0] - self.register_values[rhs.0];

                    instruction_pointer += 1;
                }
                Instruction::IMUL { lhs, rhs, target } => {
                    self.register_values[target.0] =
                        self.register_values[lhs.0] * self.register_values[rhs.0];

                    instruction_pointer += 1;
                }
                Instruction::IDIV { lhs, rhs, target } => {
                    self.register_values[target.0] =
                        self.register_values[lhs.0] / self.register_values[rhs.0];

                    instruction_pointer += 1
                }
                Instruction::ILT { lhs, rhs, target } => {
                    let lhs = self.register_values[lhs.0];
                    let rhs = self.register_values[rhs.0];

                    self.register_values[target.0] = (lhs < rhs) as i64;

                    instruction_pointer += 1;
                }
                Instruction::ILTE { lhs, rhs, target } => {
                    let lhs = self.register_values[lhs.0];
                    let rhs = self.register_values[rhs.0];

                    self.register_values[target.0] = (lhs <= rhs) as i64;

                    instruction_pointer += 1;
                }
                Instruction::IGT { lhs, rhs, target } => {
                    let lhs = self.register_values[lhs.0];
                    let rhs = self.register_values[rhs.0];

                    self.register_values[target.0] = (lhs > rhs) as i64;

                    instruction_pointer += 1;
                }
                Instruction::IGTE { lhs, rhs, target } => {
                    let lhs = self.register_values[lhs.0];
                    let rhs = self.register_values[rhs.0];

                    self.register_values[target.0] = (lhs >= rhs) as i64;

                    instruction_pointer += 1;
                }
                Instruction::FADD { lhs, rhs, target } => {
                    let lhs =
                        unsafe { std::mem::transmute::<i64, f64>(self.register_values[lhs.0]) };
                    let rhs =
                        unsafe { std::mem::transmute::<i64, f64>(self.register_values[rhs.0]) };

                    self.register_values[target.0] =
                        unsafe { std::mem::transmute::<f64, i64>(lhs + rhs) };

                    instruction_pointer += 1;
                }
                Instruction::FSUB { lhs, rhs, target } => {
                    let lhs =
                        unsafe { std::mem::transmute::<i64, f64>(self.register_values[lhs.0]) };
                    let rhs =
                        unsafe { std::mem::transmute::<i64, f64>(self.register_values[rhs.0]) };

                    self.register_values[target.0] =
                        unsafe { std::mem::transmute::<f64, i64>(lhs - rhs) };

                    instruction_pointer += 1;
                }
                Instruction::FMUL { lhs, rhs, target } => {
                    let lhs =
                        unsafe { std::mem::transmute::<i64, f64>(self.register_values[lhs.0]) };
                    let rhs =
                        unsafe { std::mem::transmute::<i64, f64>(self.register_values[rhs.0]) };

                    self.register_values[target.0] =
                        unsafe { std::mem::transmute::<f64, i64>(lhs * rhs) };

                    instruction_pointer += 1;
                }
                Instruction::FDIV { lhs, rhs, target } => {
                    let lhs =
                        unsafe { std::mem::transmute::<i64, f64>(self.register_values[lhs.0]) };
                    let rhs =
                        unsafe { std::mem::transmute::<i64, f64>(self.register_values[rhs.0]) };

                    self.register_values[target.0] =
                        unsafe { std::mem::transmute::<f64, i64>(lhs / rhs) };

                    instruction_pointer += 1;
                }
                Instruction::FLT { lhs, rhs, target } => {
                    let lhs =
                        unsafe { std::mem::transmute::<i64, f64>(self.register_values[lhs.0]) };
                    let rhs =
                        unsafe { std::mem::transmute::<i64, f64>(self.register_values[rhs.0]) };

                    self.register_values[target.0] = (lhs < rhs) as i64;

                    instruction_pointer += 1;
                }
                Instruction::FLTE { lhs, rhs, target } => {
                    let lhs =
                        unsafe { std::mem::transmute::<i64, f64>(self.register_values[lhs.0]) };
                    let rhs =
                        unsafe { std::mem::transmute::<i64, f64>(self.register_values[rhs.0]) };

                    self.register_values[target.0] = (lhs <= rhs) as i64;

                    instruction_pointer += 1;
                }
                Instruction::FGT { lhs, rhs, target } => {
                    let lhs =
                        unsafe { std::mem::transmute::<i64, f64>(self.register_values[lhs.0]) };
                    let rhs =
                        unsafe { std::mem::transmute::<i64, f64>(self.register_values[rhs.0]) };

                    self.register_values[target.0] = (lhs > rhs) as i64;

                    instruction_pointer += 1;
                }
                Instruction::FGTE { lhs, rhs, target } => {
                    let lhs =
                        unsafe { std::mem::transmute::<i64, f64>(self.register_values[lhs.0]) };
                    let rhs =
                        unsafe { std::mem::transmute::<i64, f64>(self.register_values[rhs.0]) };

                    self.register_values[target.0] = (lhs >= rhs) as i64;

                    instruction_pointer += 1;
                }
                Instruction::MOV { target, source } => {
                    self.register_values[target.0] = self.register_values[source.0];
                    instruction_pointer += 1;
                }
                Instruction::BRIF {
                    condition,
                    then_branch,
                    else_branch,
                } => {
                    let condition = self.register_values[condition.0];

                    if condition == 0 {
                        instruction_pointer = else_branch.0;
                    } else {
                        instruction_pointer = then_branch.0;
                    }
                }
                Instruction::JMP(location) => {
                    instruction_pointer = location.0;
                }
                Instruction::EXTERNALCALL { head, args, target } => {
                    let output = self.eval_external_call(*head, args, functions);

                    self.unbox_to_register(output, *target);
                    instruction_pointer += 1;
                }
            }
        }

        (self.register_values[0], self.register_types[0])
    }

    pub fn eval_external_call(
        &self,
        head: FunctionId,
        args: &[RegisterId],
        functions: &[FnRecord],
    ) -> Box<dyn Any> {
        match &functions[head.0].fun {
            // Function::ExternalFn0(fun) => fun().unwrap(),
            Function::ExternalFn1(fun) => {
                let mut val = self.box_register(args[0]);

                fun(&mut val).unwrap()
            }
            Function::ExternalFn2(fun) => {
                let mut arg0 = self.box_register(args[0]);
                let mut arg1 = self.box_register(args[1]);

                fun(&mut arg0, &mut arg1).unwrap()
            } // _ => Box::new(0),
        }
    }

    pub fn box_register(&self, register_id: RegisterId) -> Box<dyn Any> {
        Box::new(self.register_values[register_id.0])
    }

    pub fn unbox_to_register(&mut self, value: Box<dyn Any>, target: RegisterId) {
        if let Ok(value) = value.downcast::<i64>() {
            self.register_values[target.0] = *value;
        }
    }

    pub fn debug_print(&self, typechecker: &TypeChecker) {
        println!("virtual machine:");
        println!("  instructions:");
        for instr in self.instructions.iter().enumerate() {
            println!("    {:?}", instr);
        }
        println!("  registers:");
        for (idx, value) in self.register_values.iter().enumerate() {
            if self.register_types[idx] == F64_TYPE {
                println!(
                    "    {}: {} ({})",
                    idx,
                    unsafe { std::mem::transmute::<i64, f64>(*value) },
                    typechecker.stringify_type(self.register_types[idx])
                );
            } else {
                println!(
                    "    {}: {} ({})",
                    idx,
                    *value,
                    typechecker.stringify_type(self.register_types[idx])
                );
            }
        }
    }

    pub fn next_position(&self) -> usize {
        self.instructions.len()
    }
}

pub struct Translater {
    var_lookup: HashMap<NodeId, RegisterId>,
}

impl Translater {
    pub fn new() -> Self {
        Translater {
            var_lookup: HashMap::new(),
        }
    }

    pub fn translate<'source>(
        &mut self,
        delta: &'source EngineDelta,
        typechecker: &TypeChecker,
    ) -> FunctionCodegen {
        let mut builder = FunctionCodegen {
            instructions: vec![],
            register_values: vec![],
            register_types: vec![],
        };
        if !delta.ast_nodes.is_empty() {
            let last = delta.ast_nodes.len() - 1;
            let result = self.translate_node(&mut builder, NodeId(last), delta, typechecker);
            builder.mov(RegisterId(0), result);
        }

        builder
    }

    pub fn translate_node<'source>(
        &mut self,
        builder: &mut FunctionCodegen,
        node_id: NodeId,
        delta: &'source EngineDelta,
        typechecker: &TypeChecker,
    ) -> RegisterId {
        match &delta.ast_nodes[node_id.0] {
            AstNode::Int => self.translate_int(builder, node_id, delta),
            AstNode::Float => self.translate_float(builder, node_id, delta),
            AstNode::BinaryOp { lhs, op, rhs } => {
                self.translate_binop(builder, *lhs, *op, *rhs, delta, typechecker)
            }
            AstNode::Block(nodes) => self.translate_block(builder, nodes, delta, typechecker),
            AstNode::True => builder.bool_const(true),
            AstNode::False => builder.bool_const(false),
            AstNode::Let {
                variable_name,
                initializer,
                ..
            } => self.translate_let(builder, *variable_name, *initializer, delta, typechecker),
            AstNode::Variable => self.translate_variable(node_id, typechecker),
            AstNode::Statement(node_id) => {
                self.translate_node(builder, *node_id, delta, typechecker)
            }
            AstNode::If {
                condition,
                then_block,
                else_expression,
            } => self.translate_if(
                builder,
                node_id,
                *condition,
                *then_block,
                *else_expression,
                delta,
                typechecker,
            ),
            AstNode::While { condition, block } => {
                self.translate_while(builder, *condition, *block, delta, typechecker)
            }
            AstNode::Call { head, args } => {
                self.translate_call(builder, *head, args, delta, typechecker)
            }
            x => panic!("unsupported translation: {:?}", x),
        }
    }

    pub fn translate_int<'source>(
        &mut self,
        builder: &mut FunctionCodegen,
        node_id: NodeId,
        delta: &'source EngineDelta,
    ) -> RegisterId {
        let contents = &delta.contents[delta.span_start[node_id.0]..delta.span_end[node_id.0]];

        let constant = i64::from_str_radix(&String::from_utf8_lossy(contents), 10)
            .expect("internal error: int constant could not be parsed");

        builder.i64_const(constant)
    }

    pub fn translate_float<'source>(
        &mut self,
        builder: &mut FunctionCodegen,
        node_id: NodeId,
        delta: &'source EngineDelta,
    ) -> RegisterId {
        let contents = &delta.contents[delta.span_start[node_id.0]..delta.span_end[node_id.0]];

        let constant = String::from_utf8_lossy(contents)
            .parse::<f64>()
            .expect("internal error: float constant could not be parsed");

        builder.f64_const(constant)
    }

    pub fn translate_binop<'source>(
        &mut self,
        builder: &mut FunctionCodegen,
        lhs: NodeId,
        op: NodeId,
        rhs: NodeId,
        delta: &'source EngineDelta,
        typechecker: &TypeChecker,
    ) -> RegisterId {
        let lhs = self.translate_node(builder, lhs, delta, typechecker);
        let rhs = self.translate_node(builder, rhs, delta, typechecker);

        match delta.ast_nodes[op.0] {
            AstNode::Plus => builder.add(lhs, rhs),
            AstNode::Minus => builder.sub(lhs, rhs),
            AstNode::Multiply => builder.mul(lhs, rhs),
            AstNode::Divide => builder.div(lhs, rhs),
            AstNode::LessThan => builder.lt(lhs, rhs),
            AstNode::LessThanOrEqual => builder.lte(lhs, rhs),
            AstNode::GreaterThan => builder.gt(lhs, rhs),
            AstNode::GreaterThanOrEqual => builder.gte(lhs, rhs),
            AstNode::Assignment => builder.mov(lhs, rhs),
            _ => panic!("unsupported operation"),
        }
    }

    pub fn translate_let<'source>(
        &mut self,
        builder: &mut FunctionCodegen,
        variable_name: NodeId,
        initializer: NodeId,
        delta: &'source EngineDelta,
        typechecker: &TypeChecker,
    ) -> RegisterId {
        let initializer = self.translate_node(builder, initializer, delta, typechecker);

        self.var_lookup.insert(variable_name, initializer);

        initializer
    }

    pub fn translate_variable<'source>(
        &mut self,
        variable_name: NodeId,
        typechecker: &TypeChecker,
    ) -> RegisterId {
        let def_site = typechecker
            .variable_def
            .get(&variable_name)
            .expect("internal error: resolved variable not found");

        let register_id = self
            .var_lookup
            .get(def_site)
            .expect("internal error: resolved variable missing definition");

        *register_id
    }

    pub fn translate_if<'source>(
        &mut self,
        builder: &mut FunctionCodegen,
        node_id: NodeId,
        condition: NodeId,
        then_block: NodeId,
        else_expression: Option<NodeId>,
        delta: &'source EngineDelta,
        typechecker: &TypeChecker,
    ) -> RegisterId {
        let output = builder.new_register(typechecker.node_types[node_id.0]);
        let condition = self.translate_node(builder, condition, delta, typechecker);

        let brif_location = builder.next_position();
        builder.brif(output, condition, InstructionId(0), InstructionId(0));

        let then_branch = InstructionId(builder.next_position());
        let then_output = self.translate_node(builder, then_block, delta, typechecker);
        builder.mov(output, then_output);

        let else_branch = if let Some(else_expression) = else_expression {
            // Create a jump with a temporary location we'll replace when we know the correct one
            // Remember where it is so we can replace it later
            let jmp_location = builder.next_position();
            builder.jmp(InstructionId(0));

            let else_location = builder.next_position();
            let else_output = self.translate_node(builder, else_expression, delta, typechecker);
            builder.mov(output, else_output);

            let after_if = builder.next_position();

            builder.instructions[jmp_location] = Instruction::JMP(InstructionId(after_if));
            InstructionId(else_location)
        } else {
            InstructionId(builder.next_position())
        };

        builder.instructions[brif_location] = Instruction::BRIF {
            condition,
            then_branch,
            else_branch,
        };
        output
    }

    pub fn translate_while<'source>(
        &mut self,
        builder: &mut FunctionCodegen,
        condition: NodeId,
        block: NodeId,
        delta: &'source EngineDelta,
        typechecker: &TypeChecker,
    ) -> RegisterId {
        let output = builder.new_register(VOID_TYPE);

        let top = builder.next_position();
        let condition = self.translate_node(builder, condition, delta, typechecker);

        let brif_location = builder.next_position();
        builder.brif(output, condition, InstructionId(0), InstructionId(0));

        let block_begin = InstructionId(builder.next_position());
        self.translate_node(builder, block, delta, typechecker);
        builder.jmp(InstructionId(top));

        let block_end = InstructionId(builder.next_position());

        builder.instructions[brif_location] = Instruction::BRIF {
            condition,
            then_branch: block_begin,
            else_branch: block_end,
        };

        output
    }

    pub fn translate_block<'source>(
        &mut self,
        builder: &mut FunctionCodegen,
        nodes: &[NodeId],
        delta: &'source EngineDelta,
        typechecker: &TypeChecker,
    ) -> RegisterId {
        if nodes.is_empty() {
            return builder.new_register(VOID_TYPE);
        } else {
            let mut idx = 0;

            loop {
                let output = self.translate_node(builder, nodes[idx], delta, typechecker);
                if idx == (nodes.len() - 1) {
                    return output;
                }
                idx += 1;
            }
        }
    }

    pub fn translate_call<'source>(
        &mut self,
        builder: &mut FunctionCodegen,
        head: NodeId,
        args: &[NodeId],
        delta: &'source EngineDelta,
        typechecker: &TypeChecker,
    ) -> RegisterId {
        let output = builder.new_register(VOID_TYPE);

        let head = typechecker
            .call_resolution
            .get(&head)
            .expect("internal error: call should be resolved");

        let mut translated_args = vec![];

        for node_id in args {
            translated_args.push(self.translate_node(builder, *node_id, delta, typechecker));
        }

        builder.external_call(*head, translated_args, output);

        output
    }
}
