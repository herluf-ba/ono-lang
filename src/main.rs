mod vm;

use vm::*;

fn main() {
    let mut test_chunk = Chunk::new();
    test_chunk.push_constant(1, 1);
    test_chunk.push(OpCode::RETURN, 2);
    println!("{:?}", test_chunk);

    let mut vm = VirtualMachine::new(test_chunk);
    println!("{:?}", vm.run());
}
