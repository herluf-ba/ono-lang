mod vm;

use vm::*;

fn main() {
    let mut test_chunk = Chunk::new();
    test_chunk.push_constant(1.2, 123);
    test_chunk.push_constant(3.4, 123);
    test_chunk.push(OpCode::ADD, 123);
    test_chunk.push_constant(5.6, 123);
    test_chunk.push(OpCode::DIVIDE, 123);
    test_chunk.push(OpCode::NEGATE, 123);
    test_chunk.push(OpCode::RETURN, 123);

    let mut vm = VirtualMachine::new(test_chunk);
    println!("{:?}", vm.run());
}
